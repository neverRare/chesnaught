use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
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
#[derive(Debug, Clone, Copy)]
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
impl PartialEq for Advantage {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::End(a), Self::End(b)) => a == b,
            (Self::Estimated(a), Self::Estimated(b)) => a == b,
            (Self::End(EndState::Draw), Self::Estimated(advantage))
            | (Self::Estimated(advantage), Self::End(EndState::Draw)) => {
                &Estimated::default() == advantage
            }
            _ => false,
        }
    }
}
impl Eq for Advantage {}
impl PartialOrd for Advantage {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Advantage {
    fn cmp(&self, other: &Self) -> Ordering {
        macro_rules! draw {
            () => {
                Advantage::End(EndState::Draw)
            };
        }
        macro_rules! white_wins {
            () => {
                Advantage::End(EndState::Win(Color::White))
            };
        }
        macro_rules! black_wins {
            () => {
                Advantage::End(EndState::Win(Color::Black))
            };
        }
        match (self, other) {
            (Advantage::Estimated(a), Advantage::Estimated(b)) => Ord::cmp(a, b),
            (draw!(), Advantage::Estimated(advantage)) => {
                Ord::cmp(&Estimated::default(), advantage)
            }
            (Advantage::Estimated(advantage), draw!()) => {
                Ord::cmp(advantage, &Estimated::default())
            }

            (black_wins!(), black_wins!())
            | (white_wins!(), white_wins!())
            | (draw!(), draw!()) => Ordering::Equal,

            (_, black_wins!()) | (white_wins!(), _) => Ordering::Greater,

            (black_wins!(), _) | (_, white_wins!()) => Ordering::Less,
        }
    }
}
impl Hash for Advantage {
    fn hash<H: Hasher>(&self, state: &mut H) {
        #[derive(Hash)]
        pub enum Hashable {
            End(EndState),
            Estimated(Estimated),
        }
        let hashable = match self {
            Advantage::End(EndState::Draw) => Hashable::Estimated(Estimated::default()),
            Advantage::End(end_state) => Hashable::End(*end_state),
            Advantage::Estimated(estimated) => Hashable::Estimated(*estimated),
        };
        hashable.hash(state);
    }
}
pub fn estimate(board: &Board) -> Estimated {
    todo!()
}
