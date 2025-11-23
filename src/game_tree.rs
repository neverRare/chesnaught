use std::{collections::HashMap, sync::RwLock, thread::scope};

use crate::{
    chess::{Board, Color, EndState, Move},
    heuristics::Advantage,
};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTree {
    pub board: Board,
    state: Option<EndState>,
    children: Option<HashMap<Move, GameTree>>,
}
impl GameTree {
    pub fn new(board: Board) -> Self {
        GameTree {
            board,
            state: board.state(),
            children: None,
        }
    }
    pub fn move_piece(&mut self, movement: Move) {
        let new = if let Some(children) = &mut self.children {
            children.remove(&movement).unwrap()
        } else {
            GameTree::new(self.board.into_moved(movement))
        };
        *self = new;
    }
    pub fn estimate(&self) -> i32 {
        let white: i32 = self
            .board
            .into_switched_color(Color::White)
            .valid_moves()
            .count()
            .try_into()
            .unwrap();
        let black: i32 = self
            .board
            .into_switched_color(Color::Black)
            .valid_moves()
            .count()
            .try_into()
            .unwrap();
        white - black
    }
    fn alpha_beta(
        &mut self,
        depth: u32,
        multi_thread_depth: u32,

        alpha: Extended<Advantage>,
        beta: Extended<Advantage>,
    ) -> (Option<Move>, Extended<Advantage>) {
        if let Some(state) = self.state {
            (None, Extended::Finite(Advantage::End(state)))
        } else if depth == 0 {
            (
                None,
                Extended::Finite(Advantage::Estimated(self.estimate())),
            )
        } else {
            let children = self.children.get_or_insert_with(|| {
                self.board
                    .valid_moves()
                    .map(|movement| (movement, GameTree::new(self.board.into_moved(movement))))
                    .collect()
            });
            let current_player = self.board.current_player;

            if multi_thread_depth == 0 {
                let mut alpha = alpha;
                let mut beta = beta;
                let mut best_movement = None;
                let mut best_score = match current_player {
                    Color::White => Extended::NegInf,
                    Color::Black => Extended::Inf,
                };
                for (movement, game_tree) in children.iter_mut() {
                    let score = game_tree.alpha_beta(depth - 1, 0, alpha, beta).1;
                    match current_player {
                        Color::White => {
                            if score > best_score {
                                best_movement = Some(*movement);
                                best_score = score;
                            }
                            if best_score >= beta {
                                break;
                            }
                            alpha = best_score;
                        }
                        Color::Black => {
                            if score < best_score {
                                best_movement = Some(*movement);
                                best_score = score;
                            }
                            if best_score <= alpha {
                                break;
                            }
                            beta = best_score;
                        }
                    };
                }
                (best_movement, best_score)
            } else {
                struct State {
                    best_movement: Option<Move>,
                    best_score: Extended<Advantage>,
                }
                let state = RwLock::new(State {
                    best_movement: None,
                    best_score: match current_player {
                        Color::White => Extended::NegInf,
                        Color::Black => Extended::Inf,
                    },
                });
                scope(|scope| {
                    for (movement, game_tree) in children.iter_mut() {
                        let state = &state;
                        scope.spawn(|| {
                            let read = state.read().unwrap();
                            let score = game_tree
                                .alpha_beta(depth - 1, multi_thread_depth - 1, alpha, beta)
                                .1;
                            drop(read);
                            let mut write = state.write().unwrap();
                            match current_player {
                                Color::White => {
                                    if score > write.best_score {
                                        write.best_movement = Some(*movement);
                                        write.best_score = score;
                                    }
                                }
                                Color::Black => {
                                    if score < write.best_score {
                                        write.best_movement = Some(*movement);
                                        write.best_score = score;
                                    }
                                }
                            };
                            drop(write);
                        });
                    }
                });
                let state = state.into_inner().unwrap();
                (state.best_movement, state.best_score)
            }
        }
    }
    pub fn best(&mut self, depth: u32, multi_thread_depth: u32) -> Option<Move> {
        self.alpha_beta(depth, multi_thread_depth, Extended::NegInf, Extended::Inf)
            .0
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Extended<T> {
    NegInf,
    Finite(T),
    Inf,
}
impl<T> Default for Extended<T>
where
    T: Default,
{
    fn default() -> Self {
        Extended::Finite(T::default())
    }
}
