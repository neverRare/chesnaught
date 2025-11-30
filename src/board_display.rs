use std::fmt::{self, Display, Formatter};

use crate::chess::{Color, ColoredPieceKind, Coord};

const WHITE: &str = "\x1b[30;107m";
const BLACK: &str = "\x1b[30;47m";
const HIGHLIGHTED: &str = "\x1b[30;103m";
const RESET: &str = "\x1b[0m";

pub trait IndexableBoard {
    fn index(&self, position: Coord) -> Option<ColoredPieceKind>;
}
pub struct BoardDisplay<'a, 'b, T> {
    pub board: T,
    pub view: Color,
    pub highlighted: &'a [Coord],
    pub info: &'b str,
}
impl<T> Display for BoardDisplay<'_, '_, T>
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
                    .map(ColoredPieceKind::figurine)
                    .unwrap_or(' ');
                write!(f, "{color}{figurine} {RESET}")?;
            }
            write!(f, "{}", 8 - y)?;
            if let Some(line) = lines.next() {
                write!(f, " {line}")?;
            }
            writeln!(f)?;
        }
        match self.view {
            Color::White => write!(f, "a b c d e f g h")?,
            Color::Black => write!(f, "h g f e d c b a")?,
        }
        if let Some(line) = lines.next() {
            write!(f, "   {line}")?;
        }
        writeln!(f)?;
        for line in lines {
            writeln!(f, "                  {line}")?;
        }
        Ok(())
    }
}
