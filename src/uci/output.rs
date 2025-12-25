use std::{
    fmt::{self, Display, Formatter},
    num::NonZero,
    time::Duration,
};

use crate::{
    board::{Lan, NullableLan},
    color::Color,
    heuristics::Centipawn,
    misc::WithSpace,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Output {
    Id {
        field: IdField,
        value: &'static str,
    },
    UciOk,
    ReadyOk,
    BestMove {
        movement: NullableLan,
        ponder: Option<Lan>,
    },
    // CopyProtection,
    // Registration,
    Info(Info),
    Option {
        name: &'static str,
        kind: OptionType,
        default: Option<OptionValue>,
        boundary: Option<Boundary>,
    },
}
impl Display for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Output::Id { field, value } => write!(f, "id {field} {value}")?,
            Output::UciOk => write!(f, "uciok")?,
            Output::ReadyOk => write!(f, "readyok")?,
            Output::BestMove { movement, ponder } => {
                write!(f, "bestmove {movement}")?;
                if let Some(ponder) = ponder {
                    write!(f, " ponder {ponder}")?;
                }
            }
            Output::Info(info) => write!(f, "info {info}")?,
            Output::Option {
                name,
                kind,
                default,
                boundary,
            } => {
                write!(f, "option name {name} type {kind}")?;
                if let Some(default) = default {
                    write!(f, " default {default}")?;
                }
                if let Some(boundary) = boundary {
                    write!(f, " {boundary}")?;
                }
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdField {
    Name,
    Author,
}
impl Display for IdField {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            IdField::Name => write!(f, "name")?,
            IdField::Author => write!(f, "author")?,
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Info {
    Search(SearchInfo),
    Text(Box<str>),
}
impl Display for Info {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Info::Search(info) => write!(f, "{info}")?,
            Info::Text(text) => write!(f, "text {text}")?,
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SearchInfo {
    pub depth: NonZero<u32>,
    pub time: Duration,
    pub nodes: NonZero<u32>,
    pub pv: Box<[Lan]>,
    pub score: Score,
    pub hash_full: u32,
    pub nps: u32,
}
impl Display for SearchInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "depth {} time {} nodes {} pv {} score {} hashfull {} nps {}",
            self.depth,
            self.time.as_millis(),
            self.nodes,
            WithSpace(&self.pv),
            self.score,
            self.hash_full,
            self.nps,
        )?;
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Score {
    score: ScoreValue,
    bound: Option<ScoreBound>,
}
impl Score {
    pub fn from_centipawn(centipawn: Centipawn, current_player: Color) -> Self {
        match centipawn {
            Centipawn::Centipawn(centipawn) => {
                let centipawn = match current_player {
                    Color::White => centipawn,
                    Color::Black => -centipawn,
                };
                Score {
                    score: ScoreValue::Cp(centipawn),
                    bound: None,
                }
            }
            Centipawn::Win(color) => {
                if color == current_player {
                    Score {
                        score: ScoreValue::Mate(1),
                        bound: Some(ScoreBound::LowerBound),
                    }
                } else {
                    Score {
                        score: ScoreValue::Mate(-1),
                        bound: Some(ScoreBound::UpperBound),
                    }
                }
            }
        }
    }
}
impl Display for Score {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.score)?;
        if let Some(bound) = self.bound {
            write!(f, " {bound}")?;
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScoreValue {
    Cp(i32),
    Mate(i32),
}
impl Display for ScoreValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ScoreValue::Cp(cp) => write!(f, "cp {cp}")?,
            ScoreValue::Mate(moves) => write!(f, "mate {moves}")?,
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScoreBound {
    LowerBound,
    UpperBound,
}
impl Display for ScoreBound {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ScoreBound::LowerBound => write!(f, "lowerbound")?,
            ScoreBound::UpperBound => write!(f, "upperbound")?,
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptionType {
    Check,
    Spin,
    Combo,
    Button,
    String,
}
impl Display for OptionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OptionType::Check => write!(f, "check")?,
            OptionType::Spin => write!(f, "spin")?,
            OptionType::Combo => write!(f, "combo")?,
            OptionType::Button => write!(f, "button")?,
            OptionType::String => write!(f, "string")?,
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptionValue {
    Bool(bool),
    Int(i64),
    Str(&'static str),
}
impl Display for OptionValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OptionValue::Bool(b) => write!(f, "{b}")?,
            OptionValue::Int(int) => write!(f, "{int}")?,
            OptionValue::Str(s) => write!(f, "{s}")?,
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Boundary {
    Boundary { min: i32, max: i32 },
    Var(&'static [&'static str]),
}
impl Display for Boundary {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Boundary::Boundary { min, max } => write!(f, "min {min} max {max}")?,
            Boundary::Var(vars) => {
                for var in *vars {
                    write!(f, "var {var}")?;
                }
            }
        }
        Ok(())
    }
}
