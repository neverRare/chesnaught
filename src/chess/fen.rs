use std::{
    fmt::Display,
    iter::{Peekable, once, repeat},
};

use crate::chess::{Board, Piece};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fen(pub Board);

impl Display for Fen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let board = self.0;
        for (row, first) in board.board.into_iter().zip(once(true).chain(repeat(false))) {
            if !first {
                write!(f, "/")?;
            }
            for cell in CellIter(row.into_iter().peekable()) {
                write!(f, "{cell}")?;
            }
        }
        write!(f, " {}", board.current_player.lowercase())?;
        write!(f, " {}", board.castling_rights())?;
        if let Some(position) = board.en_passant_destination() {
            write!(f, " {position}")?;
        } else {
            write!(f, " -")?;
        }
        write!(f, " 0 1")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Cell {
    Piece(Piece),
    Space(u8),
}
impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cell::Piece(piece) => write!(f, "{}", piece.fen())?,
            Cell::Space(space) => write!(f, "{space}")?,
        }
        Ok(())
    }
}
struct CellIter<T>(Peekable<T>)
where
    T: Iterator;

impl<T> Iterator for CellIter<T>
where
    T: Iterator<Item = Option<Piece>>,
{
    type Item = Cell;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|piece| match piece {
            Some(piece) => Cell::Piece(piece),
            None => {
                let mut count = 1;
                while self.0.peek().is_some_and(Option::is_none) {
                    self.0.next();
                    count += 1;
                }
                Cell::Space(count)
            }
        })
    }
}
