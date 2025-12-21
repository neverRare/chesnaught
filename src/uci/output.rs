use std::{
    fmt::{self, Display, Formatter},
    num::NonZero,
    time::Duration,
};

use crate::board::{Lan, NullableLan};

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
    Info(Vec<Info>),
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
            Output::Info(infos) => {
                write!(f, "info")?;
                for info in infos {
                    write!(f, " {info}")?;
                }
            }
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
    Depth(NonZero<u32>),
    SelDepth(NonZero<u32>),
    Time(Duration),
    Nodes(NonZero<u32>),
    Pv(Vec<Lan>),
    MultiPv(u32),
    Score {
        score: Score,
        bound: Option<ScoreBound>,
    },
    CurrMove(NullableLan),
    CurrMoveNumber(u8),
    HashFull(u32),
    Nps(u32),
    TbHits(u32),
    SbHits(u32),
    CpuLoad(u32),
    String(String),
    Refutation(Vec<NullableLan>),
    CurrLine(Vec<NullableLan>),
}
impl Display for Info {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Info::Depth(depth) => write!(f, "depth {depth}")?,
            Info::SelDepth(depth) => write!(f, "seldepth {depth}")?,
            Info::Time(time) => write!(f, "time {}", time.as_millis())?,
            Info::Nodes(nodes) => write!(f, "node {nodes}")?,
            Info::Pv(moves) => {
                write!(f, "pv")?;
                for movement in moves {
                    write!(f, " {movement}")?;
                }
            }
            Info::MultiPv(rank) => write!(f, "multipv {rank}")?,
            Info::Score { score, bound } => {
                write!(f, "score {score}")?;
                if let Some(bound) = bound {
                    write!(f, " {bound}")?;
                }
            }
            Info::CurrMove(movement) => write!(f, "currmove {movement}")?,
            Info::CurrMoveNumber(order) => write!(f, "currmovenumber {order}")?,
            Info::HashFull(permill) => write!(f, "hashfull {permill}")?,
            Info::Nps(nps) => write!(f, "nps {nps}")?,
            Info::TbHits(hits) => write!(f, "tbhits {hits}")?,
            Info::SbHits(hits) => write!(f, "sbhits {hits}")?,
            Info::CpuLoad(permill) => write!(f, "cpuload {permill}")?,
            Info::String(s) => write!(f, "string {s}")?,
            Info::Refutation(moves) => {
                write!(f, "refutation")?;
                for movement in moves {
                    write!(f, " {movement}")?;
                }
            }
            Info::CurrLine(moves) => {
                write!(f, "currline")?;
                for movement in moves {
                    write!(f, " {movement}")?;
                }
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Score {
    Cp(i32),
    Mate(u32),
}
impl Display for Score {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Score::Cp(cp) => write!(f, "cp {cp}")?,
            Score::Mate(moves) => write!(f, "mate {moves}")?,
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
