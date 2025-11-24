use std::{
    cmp::Ordering,
    iter::{empty, from_fn, once},
    mem::replace,
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
            GameTreeData::Children { children, .. } => children
                .drain(..)
                .into_iter()
                .find_map(|(b, game_tree)| {
                    if movement == b {
                        Some(game_tree)
                    } else {
                        spawn(move || drop(game_tree));
                        None
                    }
                })
                .unwrap(),
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
    fn descendants_of_depth<'a>(
        &'a mut self,
        depth: u32,
    ) -> Box<dyn Iterator<Item = &'a mut Self> + 'a> {
        if depth == 0 {
            Box::new(once(self))
        } else if let Some(children) = self.children() {
            Box::new(
                children
                    .iter_mut()
                    .flat_map(move |(_, game_tree)| game_tree.descendants_of_depth(depth - 1)),
            )
        } else {
            Box::new(empty())
        }
    }
    fn alpha_beta(
        &mut self,
        depth: u32,
        scorer: fn(&mut Self) -> (Option<Move>, Advantage),
        alpha: Advantage,
        beta: Advantage,
    ) {
        self.advantage = Some(if let GameTreeData::End(state) = self.data {
            (None, Advantage::End(state))
        } else if depth == 0 {
            scorer(self)
        } else {
            let current_player = match &self.data {
                GameTreeData::Board(board) => board.current_player,
                GameTreeData::Children { current_player, .. } => *current_player,
                GameTreeData::End(_) => unreachable!(),
            };
            let children = self.children().unwrap();

            let mut alpha = alpha;
            let mut beta = beta;
            let mut best_movement = None;
            let mut best_score = match current_player {
                Color::White => Advantage::End(EndState::Win(Color::Black)),
                Color::Black => Advantage::End(EndState::Win(Color::White)),
            };
            for (movement, game_tree) in children.iter_mut() {
                game_tree.alpha_beta(depth - 1, scorer, alpha, beta);
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
    pub fn best(&mut self, depth: u32) -> (Option<Move>, Advantage) {
        let multithread_depth = match depth {
            0..=1 => 0,
            2.. => 1,
        };
        scope(|scope| {
            for game_tree in self.descendants_of_depth(multithread_depth) {
                scope.spawn(|| {
                    game_tree.alpha_beta(
                        depth - multithread_depth,
                        |game_tree| {
                            if let GameTreeData::Board(board) = &game_tree.data {
                                (None, Advantage::Estimated(estimate(Board::clone(board))))
                            } else {
                                panic!("cannot evaluate non-leaf node as board data are discarded");
                            }
                        },
                        Advantage::End(EndState::Win(Color::Black)),
                        Advantage::End(EndState::Win(Color::White)),
                    );
                });
            }
        });
        self.alpha_beta(
            multithread_depth,
            |game_tree| game_tree.advantage.unwrap(),
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
