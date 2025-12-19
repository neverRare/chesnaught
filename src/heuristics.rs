use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
};

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
                "available space for {color} king: {}",
                64 - self.king_constriction.unsigned_abs()
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
                " material: {}; square control: {}; pawn advancement: {}",
                self.material, self.square_control, self.pawn_advancement
            )?;
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

    pub fn from_end_state(end_state: EndState) -> Self {
        match end_state {
            EndState::Win(color) => Score::Win(color),
            EndState::Draw => Score::Estimated(Estimated::default()),
        }
    }
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
