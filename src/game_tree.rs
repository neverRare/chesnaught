use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    iter::from_fn,
    mem::replace,
    sync::{
        LazyLock,
        mpsc::{Sender, channel},
    },
    thread::spawn,
};

use crate::{
    board::{Board, HashableBoard, Lan},
    color::Color,
    end_state::EndState,
    heuristics::{Advantage, Estimated, estimate},
};

type MoveTreePair = (Lan, Option<Lan>, GameTree);

#[derive(Debug, Clone)]
enum GameTreeData {
    Board(Box<Board>),
    Children {
        board: Box<HashableBoard>,
        children: Vec<MoveTreePair>,
    },
    End(EndState),
}

#[derive(Debug, Clone)]
pub struct GameTree {
    data: GameTreeData,
    advantage: Option<(Option<Lan>, Advantage)>,
}
impl GameTree {
    pub fn new(board: Board) -> Self {
        let data = if let Err(state) = board.valid_moves() {
            GameTreeData::End(state)
        } else {
            GameTreeData::Board(Box::new(board))
        };
        GameTree {
            data,
            advantage: None,
        }
    }
    pub fn move_piece(&mut self, movement: Lan) {
        static DROPPER: LazyLock<Sender<GameTree>> = LazyLock::new(|| {
            let (sender, receiver) = channel();
            spawn(|| {
                for game_tree in receiver {
                    drop(game_tree);
                }
            });
            sender
        });
        let new = match &mut self.data {
            GameTreeData::Board(board) => GameTree::new(board.clone_and_move(&movement)),
            GameTreeData::Children { children, .. } => {
                children
                    .remove(
                        children
                            .iter()
                            .position(|(b, c, _)| movement == *b || Some(movement) == *c)
                            .unwrap(),
                    )
                    .2
            }
            GameTreeData::End(_) => panic!("cannot move on end state"),
        };
        let game_tree = replace(self, new);
        DROPPER.send(game_tree).unwrap();
    }
    fn board(&self) -> Option<HashableBoard> {
        match &self.data {
            GameTreeData::Board(board) => Some(board.as_hashable()),
            GameTreeData::Children { board, .. } => Some(**board),
            GameTreeData::End(_) => None,
        }
    }
    fn children(&mut self) -> Option<&mut Vec<MoveTreePair>> {
        match &mut self.data {
            GameTreeData::Board(board) => {
                let board = Board::clone(board);
                let hashable = board.as_hashable();
                self.data = GameTreeData::Children {
                    board: Box::new(hashable),
                    children: board
                        .valid_moves()
                        .unwrap()
                        .map(|movement| {
                            let (first, second) = movement.as_lan_pair(&board);
                            (
                                first,
                                second,
                                GameTree::new(board.clone_and_move(&movement)),
                            )
                        })
                        .collect(),
                };
            }
            GameTreeData::Children { .. } => (),
            GameTreeData::End(_) => return None,
        }
        let GameTreeData::Children { children, .. } = &mut self.data else {
            unreachable!()
        };
        Some(children)
    }
    fn current_player(&self) -> Option<Color> {
        match &self.data {
            GameTreeData::Board(board) => Some(board.current_player()),
            GameTreeData::Children { board, .. } => Some(board.current_player),
            GameTreeData::End(_) => None,
        }
    }
    fn alpha_beta(
        &mut self,
        depth: u32,
        scorer: fn(&mut Self) -> (Option<Lan>, Advantage),
        alpha: Advantage,
        beta: Advantage,
        transposition_table: &mut HashMap<HashableBoard, (Option<Lan>, Advantage)>,
        repetition_table: &mut HashSet<HashableBoard>,
    ) {
        if let GameTreeData::End(state) = self.data {
            let advantage = match state {
                EndState::Win(color) => Advantage::Win(color),
                EndState::Draw => Advantage::Estimated(Estimated::default()),
            };
            self.advantage = Some((None, advantage));
        } else {
            let board = self.board().unwrap();

            if let Some(advantage) = transposition_table.get(&board) {
                self.advantage = Some(*advantage);
                return;
            }
            if repetition_table.contains(&board) {
                return;
            }
            if depth == 0 {
                self.advantage = Some(scorer(self));
            } else {
                let current_player = self.current_player().unwrap();
                let children = self.children().unwrap();
                let mut alpha_beta = AlphaBetaState::new(current_player, alpha, beta);

                repetition_table.insert(board);
                for (movement, _, game_tree) in children.iter_mut() {
                    game_tree.alpha_beta(
                        depth - 1,
                        scorer,
                        alpha,
                        beta,
                        transposition_table,
                        repetition_table,
                    );
                    if let Some((_, score)) = game_tree.advantage
                        && alpha_beta.set(*movement, score)
                    {
                        break;
                    }
                }
                repetition_table.remove(&board);
                children.sort_unstable_by(|a, b| match (a.2.advantage, b.2.advantage) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => Ordering::Less,
                    (Some(_), None) => Ordering::Greater,
                    (Some((_, a)), Some((_, b))) => match current_player {
                        Color::White => Ord::cmp(&b, &a),
                        Color::Black => Ord::cmp(&a, &b),
                    },
                });
                self.advantage = Some((alpha_beta.best_move, alpha_beta.best_score));
            }
            transposition_table.insert(board, self.advantage.unwrap());
        }
    }
    fn estimate(&self) -> (Option<Lan>, Advantage) {
        if let GameTreeData::Board(board) = &self.data {
            (None, Advantage::Estimated(estimate(board)))
        } else {
            panic!("cannot evaluate non-leaf node as board data are discarded");
        }
    }
    pub fn best(&mut self, depth: u32) -> (Option<Lan>, Advantage) {
        self.alpha_beta(
            depth,
            |game_tree| GameTree::estimate(game_tree),
            Advantage::BLACK_WINS,
            Advantage::WHITE_WINS,
            &mut HashMap::new(),
            &mut HashSet::new(),
        );
        self.advantage.unwrap()
    }
    pub fn line(&self) -> impl Iterator<Item = Lan> {
        let mut game_tree = self;
        from_fn(move || {
            game_tree.advantage.unwrap().0.map(|movement| {
                if let GameTreeData::Children { children, .. } = &game_tree.data {
                    game_tree = children
                        .iter()
                        .find_map(|(b, _, game_tree)| (movement == *b).then_some(game_tree))
                        .unwrap();
                    movement
                } else {
                    panic!()
                }
            })
        })
    }
}
struct AlphaBetaState {
    current_player: Color,
    alpha: Advantage,
    beta: Advantage,
    best_move: Option<Lan>,
    best_score: Advantage,
}
impl AlphaBetaState {
    fn new(current_player: Color, alpha: Advantage, beta: Advantage) -> Self {
        AlphaBetaState {
            current_player,
            alpha,
            beta,
            best_move: None,
            best_score: match current_player {
                Color::White => Advantage::BLACK_WINS,
                Color::Black => Advantage::WHITE_WINS,
            },
        }
    }
    fn set(&mut self, movement: Lan, score: Advantage) -> bool {
        match self.current_player {
            Color::White => {
                if score > self.best_score {
                    self.best_score = score;
                    self.best_move = Some(movement);
                }
                if self.best_score >= self.beta {
                    return true;
                }
                self.alpha = Ord::max(self.alpha, self.best_score);
                false
            }
            Color::Black => {
                if score < self.best_score {
                    self.best_score = score;
                    self.best_move = Some(movement);
                }
                if self.best_score <= self.alpha {
                    return true;
                }
                self.beta = Ord::min(self.beta, self.best_score);
                false
            }
        }
    }
}
