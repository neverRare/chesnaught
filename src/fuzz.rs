use rand::{Rng, SeedableRng, rngs::SmallRng};
use rustc_hash::FxHashSet;

use crate::{
    board::{Board, Lan},
    board_display::BoardDisplay,
    coord::Coord,
    fen::Fen,
    piece::PieceKind,
};

impl From<chess::Piece> for PieceKind {
    fn from(value: chess::Piece) -> Self {
        match value {
            chess::Piece::Pawn => PieceKind::Pawn,
            chess::Piece::Knight => PieceKind::Knight,
            chess::Piece::Bishop => PieceKind::Bishop,
            chess::Piece::Rook => PieceKind::Rook,
            chess::Piece::Queen => PieceKind::Queen,
            chess::Piece::King => PieceKind::King,
        }
    }
}
impl From<chess::Square> for Coord {
    fn from(value: chess::Square) -> Self {
        Coord::new(
            value.get_file().to_index().try_into().unwrap(),
            (7 - value.get_rank().to_index()).try_into().unwrap(),
        )
    }
}
impl From<chess::ChessMove> for Lan {
    fn from(value: chess::ChessMove) -> Self {
        Lan {
            origin: value.get_source().into(),
            destination: value.get_dest().into(),
            promotion: value.get_promotion().map(Into::into),
        }
    }
}
pub fn fuzz() {
    let mut board = Board::starting_position();
    let mut rng = SmallRng::from_os_rng();
    loop {
        let moves: FxHashSet<_> = board
            .valid_moves()
            .into_iter()
            .flatten()
            .map(|movement| movement.as_lan(&board))
            .collect();
        if moves.is_empty() {
            board = Board::starting_position();
            continue;
        }
        let board2: chess::Board = Fen {
            board: board.as_hashable(),
            half_move: 0,
            full_move: 1,
        }
        .to_string()
        .parse()
        .unwrap();
        let moves2: FxHashSet<Lan> = chess::MoveGen::new_legal(&board2).map(Into::into).collect();
        if let Some(movement) = moves.difference(&moves2).next() {
            panic!(
                "found {movement} but it's not a legal move\n{}\n{}",
                BoardDisplay::new(&board),
                Fen {
                    board: board.as_hashable(),
                    half_move: 0,
                    full_move: 1,
                }
            );
        }
        if let Some(movement) = moves2.difference(&moves).next() {
            panic!(
                "{movement} not found\n{}\n{}",
                BoardDisplay::new(&board),
                Fen {
                    board: board.as_hashable(),
                    half_move: 0,
                    full_move: 1,
                }
            );
        }
        let moves: Box<[_]> = moves.into_iter().collect();
        let movement = moves[rng.random_range(0..moves.len())];
        board.move_lan(movement);
    }
}
