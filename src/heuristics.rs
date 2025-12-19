use std::cmp::Ordering;

use crate::{color::Color, end_state::EndState, misc::CompoundI8};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Estimated {
    pub king_constriction: i8,
    pub king_safety: i8,
    pub end_game_pawn_advancement: [CompoundI8; 4],
    pub material: i8,
    pub square_control: i16,
    pub pawn_advancement: i8,
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
