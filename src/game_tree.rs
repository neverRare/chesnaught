use std::{collections::HashMap, iter::once, thread::scope};

use crate::{
    chess::{Board, Color, EndState, Move},
    heuristics::Advantage,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTree {
    pub board: Board,
    state: Option<EndState>,
    children: Option<HashMap<Move, GameTree>>,
    advantage: Option<(Option<Move>, Extended<Advantage>)>,
}
impl GameTree {
    pub fn new(board: Board) -> Self {
        GameTree {
            board,
            state: board.state(),
            children: None,
            advantage: None,
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
    fn children(&mut self) -> &mut HashMap<Move, GameTree> {
        self.children.get_or_insert_with(|| {
            self.board
                .valid_moves()
                .map(|movement| (movement, GameTree::new(self.board.into_moved(movement))))
                .collect()
        })
    }
    fn descendants_of_depth<'a>(
        &'a mut self,
        depth: u32,
    ) -> Box<dyn Iterator<Item = &'a mut Self> + 'a> {
        if depth == 0 {
            Box::new(once(self))
        } else {
            Box::new(
                self.children()
                    .values_mut()
                    .flat_map(move |game_tree| game_tree.descendants_of_depth(depth - 1)),
            )
        }
    }
    fn alpha_beta(
        &mut self,
        depth: u32,
        scorer: fn(&mut Self, u32) -> Option<(Option<Move>, Extended<Advantage>)>,
        alpha: Extended<Advantage>,
        beta: Extended<Advantage>,
    ) -> (Option<Move>, Extended<Advantage>) {
        if let Some(best_move) = scorer(self, depth) {
            best_move
        } else {
            assert_ne!(depth, 0);
            let current_player = self.board.current_player;
            let children = self.children();

            let mut alpha = alpha;
            let mut beta = beta;
            let mut best_movement = None;
            let mut best_score = match current_player {
                Color::White => Extended::NegInf,
                Color::Black => Extended::Inf,
            };
            for (movement, game_tree) in children {
                let score = game_tree.alpha_beta(depth - 1, scorer, alpha, beta).1;
                match current_player {
                    Color::White => {
                        if score > best_score {
                            best_score = score;
                            best_movement = Some(*movement);
                        }
                        if best_score >= beta {
                            break;
                        }
                        alpha = best_score;
                    }
                    Color::Black => {
                        if score < best_score {
                            best_score = score;
                            best_movement = Some(*movement);
                        }
                        if best_score <= alpha {
                            break;
                        }
                        beta = best_score;
                    }
                };
            }
            (best_movement, best_score)
        }
    }
    pub fn best(&mut self, depth: u32, multi_thread_depth: u32) -> Option<Move> {
        scope(|scope| {
            for game_tree in self.descendants_of_depth(multi_thread_depth) {
                scope.spawn(|| {
                    game_tree.advantage = Some(game_tree.alpha_beta(
                        depth - multi_thread_depth,
                        |game_tree, depth| {
                            if depth == 0 {
                                Some((
                                    None,
                                    Extended::Finite(Advantage::Estimated(game_tree.estimate())),
                                ))
                            } else {
                                game_tree
                                    .state
                                    .map(|state| (None, Extended::Finite(Advantage::End(state))))
                            }
                        },
                        Extended::NegInf,
                        Extended::Inf,
                    ));
                });
            }
        });
        self.alpha_beta(
            multi_thread_depth,
            |game_tree, depth| {
                if let Some(state) = game_tree.state {
                    Some((None, Extended::Finite(Advantage::End(state))))
                } else {
                    (depth == 0).then(|| game_tree.advantage.unwrap())
                }
            },
            Extended::NegInf,
            Extended::Inf,
        )
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
