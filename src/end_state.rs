use std::fmt::{self, Display, Formatter};

use crate::color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EndState {
    Win(Color),
    Draw,
}
impl Display for EndState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EndState::Win(color) => write!(f, "{color} wins")?,
            EndState::Draw => write!(f, "draw")?,
        }
        Ok(())
    }
}
