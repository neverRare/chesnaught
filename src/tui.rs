use std::fmt::Display;

const WHITE: &str = "\x1b[30;107m";
const BLACK: &str = "\x1b[30;47m";
const HIGHLIGHTED: &str = "\x1b[30;103m";
const RESET: &str = "\x1b[0m";

use crate::chess::{Board, Color, Coord};

pub struct Tui<T> {
    pub board: Board,
    pub highlighted: T,
}
impl<'a, T> Display for Tui<T>
where
    T: IntoIterator<Item = &'a Coord> + Copy,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (y, row) in self.board.board.iter().enumerate() {
            for (x, piece) in row.iter().enumerate() {
                let position = Coord {
                    x: x as u8,
                    y: y as u8,
                };
                let color = if self
                    .highlighted
                    .into_iter()
                    .find(|highlighted| **highlighted == position)
                    .is_some()
                {
                    HIGHLIGHTED
                } else {
                    match position.board_color() {
                        Color::White => WHITE,
                        Color::Black => BLACK,
                    }
                };
                let figurine = match piece {
                    Some(piece) => piece.figurine(),
                    None => ' ',
                };
                write!(f, "{color}{figurine} {RESET}")?;
            }
            writeln!(f, "{}", 8 - y)?;
        }
        write!(f, "a b c d e f g h")?;
        Ok(())
    }
}
