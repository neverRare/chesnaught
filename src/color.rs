use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    ops::Not,
    str::FromStr,
};

use crate::error::InvalidByte;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParseColorError;
impl Display for ParseColorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "provided string was not `w`, `b`, `W`, `B`, `white`, or `black`"
        )?;
        Ok(())
    }
}
impl Error for ParseColorError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Color {
    White = 1,
    Black = 0,
}
impl Color {
    pub fn lowercase(self) -> char {
        match self {
            Color::White => 'w',
            Color::Black => 'b',
        }
    }
}
impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Color::White => write!(f, "white")?,
            Color::Black => write!(f, "black")?,
        }
        Ok(())
    }
}
impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let color = match s {
            "w" | "W" | "white" => Color::White,
            "b" | "B" | "black" => Color::Black,
            _ => return Err(ParseColorError),
        };
        Ok(color)
    }
}
impl TryFrom<u8> for Color {
    type Error = InvalidByte;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let color = match value {
            0 => Color::Black,
            1 => Color::White,
            2.. => return Err(InvalidByte),
        };
        Ok(color)
    }
}
impl From<Color> for u8 {
    fn from(value: Color) -> Self {
        match value {
            Color::White => 1,
            Color::Black => 0,
        }
    }
}
impl Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}
