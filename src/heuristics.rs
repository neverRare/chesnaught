use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
};

use crate::{board::Board, color::Color};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Estimated {
    king_safety: i32,
    square_control: i32,
}
impl Display for Estimated {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.king_safety == 0 {
            write!(f, "positional advantage: {}", self.square_control)?;
        } else {
            write!(f, "king safety: {}", self.king_safety)?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Advantage {
    Win(Color),
    Estimated(Estimated),
}
impl Advantage {
    pub const WHITE_WINS: Self = Advantage::Win(Color::White);
    pub const BLACK_WINS: Self = Advantage::Win(Color::Black);
}
impl Display for Advantage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Advantage::Win(color) => write!(f, "{color} will win")?,
            Advantage::Estimated(estimated) => write!(f, "{estimated}")?,
        }
        Ok(())
    }
}
impl Default for Advantage {
    fn default() -> Self {
        Advantage::Estimated(Estimated::default())
    }
}
impl PartialOrd for Advantage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Advantage {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Advantage::Estimated(a), Advantage::Estimated(b)) => Ord::cmp(a, b),

            (&Advantage::BLACK_WINS, &Advantage::BLACK_WINS)
            | (&Advantage::WHITE_WINS, &Advantage::WHITE_WINS) => Ordering::Equal,

            (_, &Advantage::BLACK_WINS) | (&Advantage::WHITE_WINS, _) => Ordering::Greater,

            (&Advantage::BLACK_WINS, _) | (_, &Advantage::WHITE_WINS) => Ordering::Less,
        }
    }
}
pub fn estimate(board: &Board) -> Estimated {
    todo!()
}
