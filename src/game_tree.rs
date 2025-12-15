use std::{
    cmp::Ordering,
    collections::HashMap,
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

#[derive(Debug, Clone)]
enum GameTreeData {
    Board(Box<Board>),
    Children {
        current_player: Color,
        children: Vec<(Lan, Option<Lan>, GameTree)>,
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
    fn children(&mut self) -> Option<&mut Vec<(Lan, Option<Lan>, GameTree)>> {
        match &mut self.data {
            GameTreeData::Board(board) => {
                let board = Board::clone(board);
                self.data = GameTreeData::Children {
                    current_player: board.current_player(),
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
            GameTreeData::Children { current_player, .. } => Some(*current_player),
            GameTreeData::End(_) => None,
        }
    }
    fn alpha_beta(
        &mut self,
        depth: u32,
        scorer: fn(&mut Self) -> (Option<Lan>, Advantage),
        alpha: Advantage,
        beta: Advantage,
        transposition_table: &mut HashMap<HashableBoard, Advantage>,
    ) {
        self.advantage = Some(if let GameTreeData::End(state) = self.data {
            let advantage = match state {
                EndState::Win(color) => Advantage::Win(color),
                EndState::Draw => Advantage::Estimated(Estimated::default()),
            };
            (None, advantage)
        } else if depth == 0 {
            scorer(self)
        } else {
            let current_player = self.current_player().unwrap();
            let children = self.children().unwrap();

            let mut alpha = alpha;
            let mut beta = beta;
            let mut best_movement = None;
            let mut best_score = match current_player {
                Color::White => Advantage::BLACK_WINS,
                Color::Black => Advantage::WHITE_WINS,
            };
            for (movement, _, game_tree) in children.iter_mut() {
                game_tree.alpha_beta(depth - 1, scorer, alpha, beta, transposition_table);
                let score = game_tree.advantage.unwrap().1;
                match current_player {
                    Color::White => {
                        if score > best_score {
                            best_score = score;
                            best_movement = Some(*movement);
                        }
                        if best_score >= beta {
                            break;
                        }
                        alpha = Ord::max(alpha, best_score);
                    }
                    Color::Black => {
                        if score < best_score {
                            best_score = score;
                            best_movement = Some(*movement);
                        }
                        if best_score <= alpha {
                            break;
                        }
                        beta = Ord::min(beta, best_score);
                    }
                }
            }
            children.sort_unstable_by(|a, b| match (a.2.advantage, b.2.advantage) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (Some((_, a)), Some((_, b))) => match current_player {
                    Color::White => Ord::cmp(&b, &a),
                    Color::Black => Ord::cmp(&a, &b),
                },
            });
            (best_movement, best_score)
        });
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
