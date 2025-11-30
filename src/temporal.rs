use std::collections::HashMap;

use crate::chess::{Board, EndState, HashableBoard, Move};

struct History(HashMap<HashableBoard, u8>);

impl History {
    fn record(&mut self, board: HashableBoard) {
        let count = self.0.entry(board).or_default();
        *count += 1;
    }
    fn count(&self, board: HashableBoard) -> u8 {
        self.0.get(&board).copied().unwrap_or_default()
    }
}
pub struct TemporalBoard {
    board: Board,
    hashable_board: HashableBoard,
    history: History,
    half_move: u32,
}
impl TemporalBoard {
    pub fn valid_moves(&self) -> Result<impl Iterator<Item = Move>, EndState> {
        if self.half_move > 50 || self.history.count(self.hashable_board) > 2 {
            Err(EndState::Draw)
        } else {
            self.board.valid_moves()
        }
    }
}
