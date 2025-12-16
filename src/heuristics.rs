use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
};

use crate::{board::Board, color::Color};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Estimated {
    pub king_constriction: i8,
    pub king_safety: i8,
    pub end_game_pawn_advancement: [CompoundI8; 4],
    pub square_control: i16,
    pub material: i8,
    pub pawn_advancement: i8,
}
impl Display for Estimated {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.king_constriction != 0 {
            let color = match self.king_constriction.signum() {
                1 => Color::Black,
                -1 => Color::White,
                _ => unreachable!(),
            };
            write!(
                f,
                "available space for {color}'s king: {}",
                64 - self.king_constriction.abs()
            )?;
        } else if self.king_safety != 0 {
            write!(f, "king safety: {}", self.king_safety)?;
        } else if self.end_game_pawn_advancement != [CompoundI8::default(); 4] {
            write!(f, "pawn advancement:")?;
            for score in self
                .end_game_pawn_advancement
                .into_iter()
                .flat_map(|compound| [compound.left(), compound.right()])
            {
                write!(f, " {score}")?;
            }
        } else {
            write!(
                f,
                "square control: {}; material: {}; pawn advancement: {}",
                self.square_control, self.material, self.pawn_advancement
            )?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompoundI8(i8);
impl CompoundI8 {
    fn new(left: i8, right: i8) -> Self {
        debug_assert!(left < 8);
        debug_assert!(left >= -8);
        debug_assert!(right < 8);
        debug_assert!(right >= -8);
        CompoundI8((left << 4) | (right & 0b_1111))
    }
    fn left(self) -> i8 {
        self.0 >> 4
    }
    fn right(self) -> i8 {
        (self.0 << 4) >> 4
    }
}
impl Default for CompoundI8 {
    fn default() -> Self {
        CompoundI8::new(0, 0)
    }
}
impl PartialOrd for CompoundI8 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for CompoundI8 {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&(self.left(), self.right()), &(other.left(), other.right()))
    }
}
impl Neg for CompoundI8 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        CompoundI8::new(-self.left(), -self.right())
    }
}
impl Add<CompoundI8> for CompoundI8 {
    type Output = Self;

    fn add(self, rhs: CompoundI8) -> Self::Output {
        CompoundI8::new(self.left() + rhs.left(), self.right() + rhs.right())
    }
}
impl AddAssign<CompoundI8> for CompoundI8 {
    fn add_assign(&mut self, rhs: CompoundI8) {
        *self = CompoundI8::new(self.left() + rhs.left(), self.right() + rhs.right());
    }
}
impl Sub<CompoundI8> for CompoundI8 {
    type Output = Self;

    fn sub(self, rhs: CompoundI8) -> Self::Output {
        CompoundI8::new(self.left() - rhs.left(), self.right() - rhs.right())
    }
}
impl SubAssign<CompoundI8> for CompoundI8 {
    fn sub_assign(&mut self, rhs: CompoundI8) {
        *self = CompoundI8::new(self.left() - rhs.left(), self.right() - rhs.right());
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
