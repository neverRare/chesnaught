use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
};

use crate::chess::{Board, Color, EndState};

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
            (Advantage::End(EndState::Draw), Advantage::Estimated(advantage)) => {
                Ord::cmp(&Estimated::default(), advantage)
            }
            (Advantage::Estimated(advantage), Advantage::End(EndState::Draw)) => {
                Ord::cmp(advantage, &Estimated::default())
            }

            (
                Advantage::End(EndState::Win(Color::Black)),
                Advantage::End(EndState::Win(Color::Black)),
            ) => Ordering::Equal,
            (Advantage::End(EndState::Win(Color::Black)), _) => Ordering::Less,
            (_, Advantage::End(EndState::Win(Color::Black))) => Ordering::Greater,

            (
                Advantage::End(EndState::Win(Color::White)),
                Advantage::End(EndState::Win(Color::White)),
            ) => Ordering::Equal,
            (Advantage::End(EndState::Win(Color::White)), _) => Ordering::Greater,
            (_, Advantage::End(EndState::Win(Color::White))) => Ordering::Less,

            (Advantage::End(EndState::Draw), Advantage::End(EndState::Draw)) => Ordering::Equal,
        }
    }
}
pub fn estimate(board: Board) -> Estimated {
    todo!()
}
