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
pub enum Score {
    Win(Color),
    Estimated(Estimated),
}
impl Score {
    pub const WHITE_WINS: Self = Score::Win(Color::White);
    pub const BLACK_WINS: Self = Score::Win(Color::Black);
}
impl Display for Score {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Score::Win(color) => write!(f, "{color} will win")?,
            Score::Estimated(estimated) => write!(f, "{estimated}")?,
        }
        Ok(())
    }
}
impl Default for Score {
    fn default() -> Self {
        Score::Estimated(Estimated::default())
    }
}
impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Score::Estimated(a), Score::Estimated(b)) => Ord::cmp(a, b),

            (&Score::BLACK_WINS, &Score::BLACK_WINS) | (&Score::WHITE_WINS, &Score::WHITE_WINS) => {
                Ordering::Equal
            }

            (_, &Score::BLACK_WINS) | (&Score::WHITE_WINS, _) => Ordering::Greater,

            (&Score::BLACK_WINS, _) | (_, &Score::WHITE_WINS) => Ordering::Less,
        }
    }
}
pub fn estimate(board: &Board) -> Estimated {
    todo!()
}
