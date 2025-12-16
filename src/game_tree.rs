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
    heuristics::{Estimated, Score, estimate},
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
    score: Option<(Option<Lan>, Score)>,
}
impl GameTree {
    pub fn new(board: Board) -> Self {
        let data = if let Err(state) = board.valid_moves() {
            GameTreeData::End(state)
        } else {
            GameTreeData::Board(Box::new(board))
        };
        GameTree { data, score: None }
    }
    pub fn drop(self) {
        static DROPPER: LazyLock<Sender<GameTree>> = LazyLock::new(|| {
            let (sender, receiver) = channel();
            spawn(|| {
                for game_tree in receiver {
                    drop(game_tree);
                }
            });
            sender
        });
        DROPPER.send(self).unwrap();
    }
    pub fn move_piece(&mut self, movement: Lan) {
        let new = match &mut self.data {
            GameTreeData::Board(board) => GameTree::new(board.clone_and_move(&movement)),
            GameTreeData::Children { children, .. } => {
                let (_, _, children) = children.remove(
                    children
                        .iter()
                        .position(|(b, c, _)| movement == *b || Some(movement) == *c)
                        .unwrap(),
                );
                children
            }
            GameTreeData::End(_) => panic!("cannot move on end state"),
        };
        replace(self, new).drop();
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
        alpha: Score,
        beta: Score,
        transposition_table: &mut HashMap<HashableBoard, (Option<Lan>, Score)>,
        repetition_table: &mut HashSet<HashableBoard>,
    ) {
        if let GameTreeData::End(state) = self.data {
            let score = match state {
                EndState::Win(color) => Score::Win(color),
                EndState::Draw => Score::Estimated(Estimated::default()),
            };
            self.score = Some((None, score));
        } else {
            let board = self.board().unwrap();

            if let Some(score) = transposition_table.get(&board) {
                self.score = Some(*score);
                return;
            }
            if repetition_table.contains(&board) {
                return;
            }
            if depth == 0 {
                self.score = Some(self.estimate());
            } else {
                let current_player = self.current_player().unwrap();
                let children = self.children().unwrap();
                let mut alpha_beta = AlphaBetaState::new(current_player, alpha, beta);

                repetition_table.insert(board);
                for (movement, _, game_tree) in children.iter_mut() {
                    game_tree.alpha_beta(
                        depth - 1,
                        alpha,
                        beta,
                        transposition_table,
                        repetition_table,
                    );
                    if let Some((_, score)) = game_tree.score
                        && alpha_beta.set(*movement, score)
                    {
                        break;
                    }
                }
                repetition_table.remove(&board);
                children.sort_unstable_by(|(_, _, a), (_, _, b)| match (a.score, b.score) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => Ordering::Less,
                    (Some(_), None) => Ordering::Greater,
                    (Some((_, a)), Some((_, b))) => match current_player {
                        Color::White => Ord::cmp(&b, &a),
                        Color::Black => Ord::cmp(&a, &b),
                    },
                });
                self.score = Some((alpha_beta.best_move, alpha_beta.best_score));
            }
            transposition_table.insert(board, self.score.unwrap());
        }
    }
    fn estimate(&self) -> (Option<Lan>, Score) {
        let estimated = if let GameTreeData::Board(board) = &self.data {
            estimate(board)
        } else if let Some(score) = self.score {
            return score;
        } else if cfg!(debug_assertions) {
            panic!(concat!(
                "this node only contains board data meant for hashing alone. ",
                "the original board data is discarded to save memory space. ",
                "while it is possible to convert it back, we shouldn't resort ",
                "to this"
            ));
        } else {
            estimate(
                &self
                    .board()
                    .expect("can't estimate score on board with ended state")
                    .try_into()
                    .unwrap(),
            )
        };
        (None, Score::Estimated(estimated))
    }
    pub fn best(&mut self, depth: u32) -> (Option<Lan>, Score) {
        self.alpha_beta(
            depth,
            Score::BLACK_WINS,
            Score::WHITE_WINS,
            &mut HashMap::new(),
            &mut HashSet::new(),
        );
        self.score.unwrap()
    }
    pub fn line(&self) -> impl Iterator<Item = Lan> {
        let mut game_tree = self;
        from_fn(move || {
            let (movement, _) = game_tree.score.unwrap();
            movement.map(|movement| {
                if let GameTreeData::Children { children, .. } = &game_tree.data {
                    game_tree = &children[0].2;
                    movement
                } else {
                    unreachable!()
                }
            })
        })
    }
}
struct AlphaBetaState {
    current_player: Color,
    alpha: Score,
    beta: Score,
    best_move: Option<Lan>,
    best_score: Score,
}
impl AlphaBetaState {
    fn new(current_player: Color, alpha: Score, beta: Score) -> Self {
        AlphaBetaState {
            current_player,
            alpha,
            beta,
            best_move: None,
            best_score: match current_player {
                Color::White => Score::BLACK_WINS,
                Color::Black => Score::WHITE_WINS,
            },
        }
    }
    fn set(&mut self, movement: Lan, score: Score) -> bool {
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
