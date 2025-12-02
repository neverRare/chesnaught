use std::fmt::{self, Display, Formatter};

use crate::{color::Color, coord::Coord, piece::ColoredPieceKind};

const WHITE: &str = "\x1b[30;107m";
const BLACK: &str = "\x1b[30;47m";
const HIGHLIGHTED: &str = "\x1b[30;103m";
const RESET: &str = "\x1b[0m";

pub trait IndexableBoard {
    fn index(&self, position: Coord) -> Option<ColoredPieceKind>;
}
pub struct BoardDisplay<'board, 'highlighted, 'info, T> {
    pub board: &'board T,
    pub view: Color,
    pub show_coordinates: bool,
    pub highlighted: &'highlighted [Coord],
    pub info: &'info str,
}
impl<T> Display for BoardDisplay<'_, '_, '_, T>
where
    T: IndexableBoard,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut lines = self.info.lines().fuse();
        for y in 0..8 {
            let y = match self.view {
                Color::White => y,
                Color::Black => 7 - y,
            };
            for x in 0..8 {
                let x = match self.view {
                    Color::White => x,
                    Color::Black => 7 - x,
                };
                let position = Coord::new(x, y);
                let color = if self.highlighted.contains(&position) {
                    HIGHLIGHTED
                } else {
                    match position.color() {
                        Color::White => WHITE,
                        Color::Black => BLACK,
                    }
                };
                let figurine = self
                    .board
                    .index(position)
                    .map_or(' ', ColoredPieceKind::figurine);
                write!(f, "{color}{figurine} {RESET}")?;
            }
            if self.show_coordinates {
                write!(f, "{}", 8 - y)?;
            }
            let space = if self.show_coordinates { "  " } else { " " };
            if let Some(line) = lines.next() {
                write!(f, "{space}{line}")?;
            }
            writeln!(f)?;
        }
        if self.show_coordinates {
            match self.view {
                Color::White => write!(f, "a b c d e f g h")?,
                Color::Black => write!(f, "h g f e d c b a")?,
            }
            if let Some(line) = lines.next() {
                write!(f, "    {line}")?;
            }
            writeln!(f)?;
        }
        let spaces = if self.show_coordinates {
            "                   "
        } else {
            "                 "
        };
        for line in lines {
            writeln!(f, "{spaces}{line}")?;
        }
        Ok(())
    }
}
