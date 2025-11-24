use std::{
    cmp::Ordering,
    iter::from_fn,
    mem::replace,
    sync::{
        LazyLock,
        mpsc::{Sender, channel},
    },
    thread::{scope, spawn},
};

use crate::{
    chess::{Board, Color, EndState, Move},
    heuristics::{Advantage, estimate},
};

#[derive(Debug, Clone)]
enum GameTreeData {
    Board(Box<Board>),
    Children {
        current_player: Color,
        children: Vec<(Move, GameTree)>,
    },
    End(EndState),
}
#[derive(Debug, Clone)]
pub struct GameTree {
    data: GameTreeData,
    advantage: Option<(Option<Move>, Advantage)>,
}
impl GameTree {
    pub fn new(board: Board) -> Self {
        let data = if let Some(state) = board.state() {
            GameTreeData::End(state)
        } else {
            GameTreeData::Board(Box::new(board))
        };
        GameTree {
            data,
            advantage: None,
        }
    }
    pub fn move_piece(&mut self, movement: Move) {
        let new = match &mut self.data {
            GameTreeData::Board(board) => GameTree::new(board.into_moved(movement)),
            GameTreeData::Children { children, .. } => {
                children
                    .remove(children.iter().position(|(b, _)| movement == *b).unwrap())
                    .1
            }
            GameTreeData::End(_) => panic!("cannot move on end state"),
        };
        *self = new;
    }
    fn children(&mut self) -> Option<&mut Vec<(Move, GameTree)>> {
        match &mut self.data {
            GameTreeData::Board(board) => {
                let board = Board::clone(board);
                self.data = GameTreeData::Children {
                    current_player: board.current_player,
                    children: board
                        .valid_moves()
                        .map(|movement| (movement, GameTree::new(board.into_moved(movement))))
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
            GameTreeData::Board(board) => Some(board.current_player),
            GameTreeData::Children { current_player, .. } => Some(*current_player),
            GameTreeData::End(_) => None,
        }
    }
    fn alpha_beta(
        &mut self,
        depth: u32,
        thread_count: Option<usize>,
        scorer: fn(&mut Self) -> (Option<Move>, Advantage),
        alpha: Advantage,
        beta: Advantage,
    ) {
        self.advantage = Some(if let GameTreeData::End(state) = self.data {
            (None, Advantage::End(state))
        } else if depth == 0 {
            scorer(self)
        } else {
            let current_player = self.current_player().unwrap();
            let children = self.children().unwrap();

            let mut alpha = alpha;
            let mut beta = beta;
            let mut best_movement = None;
            let mut best_score = match current_player {
                Color::White => Advantage::End(EndState::Win(Color::Black)),
                Color::Black => Advantage::End(EndState::Win(Color::White)),
            };
            if let Some(thread_count) = thread_count {
                for chunk in children.chunks_mut(thread_count) {
                    let (movement, score) = scope(|scope| {
                        let handles: Vec<_> = chunk
                            .iter_mut()
                            .map(|(movement, game_tree)| {
                                scope.spawn(|| {
                                    game_tree.alpha_beta(depth - 1, None, scorer, alpha, beta);
                                    (*movement, game_tree.advantage.unwrap().1)
                                })
                            })
                            .collect();
                        while !handles.iter().all(|handle| handle.is_finished()) {}
                        let iter = handles.into_iter().map(|handle| handle.join().unwrap());
                        match current_player {
                            Color::White => iter.max_by_key(|(_, advantage)| *advantage).unwrap(),
                            Color::Black => iter.min_by_key(|(_, advantage)| *advantage).unwrap(),
                        }
                    });
                    match current_player {
                        Color::White => {
                            if score > best_score {
                                best_score = score;
                                best_movement = Some(movement);
                            }
                            if best_score >= beta {
                                break;
                            }
                            alpha = Ord::max(alpha, best_score);
                        }
                        Color::Black => {
                            if score < best_score {
                                best_score = score;
                                best_movement = Some(movement);
                            }
                            if best_score <= alpha {
                                break;
                            }
                            beta = Ord::min(beta, best_score);
                        }
                    }
                }
            } else {
                for (movement, game_tree) in children.iter_mut() {
                    game_tree.alpha_beta(depth - 1, None, scorer, alpha, beta);
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
            }
            children.sort_unstable_by(|a, b| match (a.1.advantage, b.1.advantage) {
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
    fn estimate(&self) -> (Option<Move>, Advantage) {
        if let GameTreeData::Board(board) = &self.data {
            (None, Advantage::Estimated(estimate(Board::clone(board))))
        } else {
            panic!("cannot evaluate non-leaf node as board data are discarded");
        }
    }
    pub fn best(&mut self, depth: u32, thread_count: Option<usize>) -> (Option<Move>, Advantage) {
        self.alpha_beta(
            depth,
            thread_count,
            |game_tree| GameTree::estimate(game_tree),
            Advantage::End(EndState::Win(Color::Black)),
            Advantage::End(EndState::Win(Color::White)),
        );
        self.advantage.unwrap()
    }
    pub fn line(&self) -> impl Iterator<Item = Move> {
        let mut game_tree = self;
        from_fn(move || {
            game_tree.advantage.unwrap().0.map(|movement| {
                if let GameTreeData::Children { children, .. } = &game_tree.data {
                    game_tree = children
                        .iter()
                        .find_map(|(b, game_tree)| (movement == *b).then_some(game_tree))
                        .unwrap();
                    movement
                } else {
                    panic!()
                }
            })
        })
    }
}

impl Drop for GameTree {
    fn drop(&mut self) {
        static DROPPER: LazyLock<Sender<SyncDrop>> = LazyLock::new(|| {
            let (sender, receiver) = channel();
            spawn(|| {
                for game_tree in receiver {
                    drop(game_tree);
                }
            });
            sender
        });
        let game_tree = replace(
            self,
            GameTree {
                data: GameTreeData::End(EndState::Draw),
                advantage: None,
            },
        );
        DROPPER.send(SyncDrop(game_tree)).unwrap();
    }
}
struct SyncDrop(GameTree);

impl Drop for SyncDrop {
    fn drop(&mut self) {
        if let GameTreeData::Children { children, .. } = &mut self.0.data {
            for (_, game_tree) in children.drain(..) {
                drop(SyncDrop(game_tree));
            }
        }
    }
}
