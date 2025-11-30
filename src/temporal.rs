use std::collections::HashSet;

use crate::chess::{Board, HashableBoard};

pub struct TemporalBoard {
    board: Board,
    past: HashSet<HashableBoard>,
    last_pawn_moves: u16,
}
