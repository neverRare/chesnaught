use std::collections::HashMap;

use crate::chess::{Board, EndState, Move};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameTree {
    board: Board,
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
}
