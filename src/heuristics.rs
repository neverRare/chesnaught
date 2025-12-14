use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
};

use crate::{board::Board, color::Color, end_state::EndState};

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
    End(EndState),
    Estimated(Estimated),
}
impl Advantage {
    pub const WHITE_WINS: Self = Advantage::End(EndState::Win(Color::White));
    pub const BLACK_WINS: Self = Advantage::End(EndState::Win(Color::Black));
    pub const DRAW: Self = Advantage::End(EndState::Draw);
}
impl Display for Advantage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Advantage::End(EndState::Draw) => write!(f, "will draw")?,
            Advantage::End(EndState::Win(color)) => write!(f, "{color} will win")?,
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
            (&Advantage::DRAW, Advantage::Estimated(advantage)) => {
                Ord::cmp(&Estimated::default(), advantage)
            }
            (Advantage::Estimated(advantage), &Advantage::DRAW) => {
                Ord::cmp(advantage, &Estimated::default())
            }

            (&Advantage::BLACK_WINS, &Advantage::BLACK_WINS)
            | (&Advantage::WHITE_WINS, &Advantage::WHITE_WINS)
            | (&Advantage::DRAW, &Advantage::DRAW) => Ordering::Equal,

            (_, &Advantage::BLACK_WINS) | (&Advantage::WHITE_WINS, _) => Ordering::Greater,

            (&Advantage::BLACK_WINS, _) | (_, &Advantage::WHITE_WINS) => Ordering::Less,
        }
    }
}
pub fn estimate(board: &Board) -> Estimated {
    todo!()
}
