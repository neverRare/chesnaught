use std::{
    cmp::Ordering,
    ops::{Add, AddAssign, Neg, Sub, SubAssign},
};

use crate::{color::Color, end_state::EndState, misc::CompoundI8};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PawnAdvancement(pub [CompoundI8; 4]);

impl PawnAdvancement {
    pub fn new(array: [i8; 8]) -> Self {
        PawnAdvancement([
            CompoundI8::new(array[0], array[1]),
            CompoundI8::new(array[2], array[3]),
            CompoundI8::new(array[4], array[5]),
            CompoundI8::new(array[6], array[7]),
        ])
    }
}
impl Neg for PawnAdvancement {
    type Output = Self;

    fn neg(self) -> Self::Output {
        PawnAdvancement(self.0.map(|value| -value))
    }
}
impl Add for PawnAdvancement {
    type Output = PawnAdvancement;

    fn add(self, rhs: Self) -> Self::Output {
        PawnAdvancement([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
            self.0[3] + rhs.0[3],
        ])
    }
}
impl AddAssign for PawnAdvancement {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl Sub for PawnAdvancement {
    type Output = PawnAdvancement;

    fn sub(self, rhs: Self) -> Self::Output {
        PawnAdvancement([
            self.0[0] - rhs.0[0],
            self.0[1] - rhs.0[1],
            self.0[2] - rhs.0[2],
            self.0[3] - rhs.0[3],
        ])
    }
}
impl SubAssign for PawnAdvancement {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Estimated {
    pub material: i8,
    pub king_safety: i8,
    pub square_control: i16,
    pub pawn_advancement: PawnAdvancement,
}
impl Estimated {
    pub fn centipawn(self) -> i32 {
        <i32>::from(self.material) * 100
            + <i32>::from(self.king_safety) * 10
            + <i32>::from(self.square_control)
    }
}
impl Add for Estimated {
    type Output = Estimated;

    fn add(self, rhs: Self) -> Self::Output {
        Estimated {
            king_safety: self.king_safety + rhs.king_safety,
            pawn_advancement: self.pawn_advancement + rhs.pawn_advancement,
            material: self.material + rhs.material,
            square_control: self.square_control + rhs.square_control,
        }
    }
}
impl AddAssign for Estimated {
    fn add_assign(&mut self, rhs: Self) {
        self.king_safety += rhs.king_safety;
        self.pawn_advancement += rhs.pawn_advancement;
        self.material += rhs.material;
        self.square_control += rhs.square_control;
    }
}
impl Sub for Estimated {
    type Output = Estimated;

    fn sub(self, rhs: Self) -> Self::Output {
        Estimated {
            king_safety: self.king_safety - rhs.king_safety,
            pawn_advancement: self.pawn_advancement - rhs.pawn_advancement,
            material: self.material - rhs.material,
            square_control: self.square_control - rhs.square_control,
        }
    }
}
impl SubAssign for Estimated {
    fn sub_assign(&mut self, rhs: Self) {
        self.king_safety -= rhs.king_safety;
        self.pawn_advancement -= rhs.pawn_advancement;
        self.material -= rhs.material;
        self.square_control -= rhs.square_control;
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

    pub fn from_end_state(end_state: EndState) -> Self {
        match end_state {
            EndState::Win(color) => Score::Win(color),
            EndState::Draw => Score::Estimated(Estimated::default()),
        }
    }
    pub fn is_win(self) -> bool {
        matches!(self, Score::Win(_))
    }
    pub fn centipawn(self) -> Centipawn {
        match self {
            Score::Win(color) => Centipawn::Win(color),
            Score::Estimated(estimated) => Centipawn::Centipawn(estimated.centipawn()),
        }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Centipawn {
    Centipawn(i32),
    Win(Color),
}
impl Default for Centipawn {
    fn default() -> Self {
        Centipawn::Centipawn(0)
    }
}
