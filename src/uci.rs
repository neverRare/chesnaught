use std::{num::NonZero, time::Duration};

use crate::{board::Lan, fen::Fen};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Input<'a> {
    Uci,
    Debug(bool),
    IsReady,
    SetOption {
        name: &'a str,
        value: Option<&'a str>,
    },
    Register(Register<'a>),
    UciNewGame,
    Position {
        position: Position,
        moves: Vec<Lan>,
    },
    Go {
        search_moves: Option<Vec<Lan>>,
        ponder: bool,
        w_time: Option<Duration>,
        b_time: Option<Duration>,
        w_inc: Option<Duration>,
        b_inc: Option<Duration>,
        moves_to_go: Option<NonZero<u32>>,
        depth: Option<NonZero<u32>>,
        nodes: Option<NonZero<u32>>,
        mate: Option<NonZero<u32>>,
        move_time: Option<Duration>,
        infinite: bool,
    },
    Stop,
    PonderHit,
    Quit,
    Repl,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Register<'a> {
    Later,
    NameCode { name: &'a str, code: &'a str },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Position {
    StartPos,
    Fen(Fen),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Output {
    Id {
        name: &'static str,
        author: &'static str,
    },
    UciOk,
    ReadyOk,
    BestMove {
        movement: Lan,
        ponder: Option<Lan>,
    },
    CopyProtection,
    Registration,
    Info(Vec<Info>),
    Option {
        name: &'static str,
        kind: OptionType,
        default: OptionValue,
        possible_values: Boundary,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Info {
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
    CurrMove(Lan),
    CurrMoveNumber(u8),
    HashFull(u32),
    Nps(u32),
    TbHits(u32),
    SbHits(u32),
    CpuLoad(u32),
    String(String),
    Refutation(Vec<Lan>),
    CurrLine(Vec<Lan>),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Score {
    Cp(i32),
    Mate(NonZero<u32>),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ScoreBound {
    Lower,
    Upper,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum OptionType {
    Check,
    Spin,
    Combo,
    Button,
    String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum OptionValue {
    Bool(bool),
    Int(i32),
    Str(&'static str),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Boundary {
    Unbounded,
    Bounded { lower_bound: i32, upper_bound: i32 },
    Selection(&'static [&'static str]),
}
