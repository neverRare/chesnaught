use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    iter::from_fn,
    num::NonZero,
    str::FromStr,
    time::Duration,
};

use crate::{
    board::{Board, InvalidBoard, Lan},
    fen::{Fen, ParseFenError},
    misc::{extract_prefix_token, split_by_token, starts_with_token, strip_prefix_token},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Input<'a> {
    Uci,
    Debug(bool),
    IsReady,
    SetOption {
        name: &'a str,
        value: Option<&'a str>,
    },

    // Recognized but ignored, hence associated data are not parsed
    Register(&'a str),

    UciNewGame,
    Position {
        position: Position,
        moves: Vec<Lan>,
    },
    Go(Go),
    Stop,
    PonderHit,
    Quit,
    Repl,
}
impl<'a> Input<'a> {
    fn from_str_from_start(src: &'a str) -> Result<Self, ParseInputError> {
        if starts_with_token(src, "uci") {
            Ok(Input::Uci)
        } else if let Some(src) = strip_prefix_token(src, "debug") {
            if starts_with_token(src, "on") {
                Ok(Input::Debug(true))
            } else if starts_with_token(src, "off") {
                Ok(Input::Debug(false))
            } else {
                Err(ParseInputError::NotOnOrOff)
            }
        } else if starts_with_token(src, "isready") {
            Ok(Input::IsReady)
        } else if let Some(src) = strip_prefix_token(src, "setoption") {
            let Some(src) = strip_prefix_token(src, "name") else {
                return Err(ParseInputError::NoName);
            };
            let Some((name, value)) = split_by_token(src, "value") else {
                return Ok(Input::SetOption {
                    name: src,
                    value: None,
                });
            };
            Ok(Input::SetOption {
                name,
                value: Some(value),
            })
        } else if let Some(src) = strip_prefix_token(src, "register") {
            Ok(Input::Register(src))
        } else if starts_with_token(src, "ucinewgame") {
            Ok(Input::UciNewGame)
        } else if let Some(src) = strip_prefix_token(src, "position") {
            let (position, mut moves) = split_by_token(src, "moves").unwrap_or((src, ""));
            let position = position.parse()?;
            let moves = &mut moves;
            let moves = from_fn(|| {
                if moves.is_empty() {
                    None
                } else {
                    let index = moves.find(<char>::is_whitespace).unwrap_or(moves.len());
                    let (movement, rest) = src.split_at(index);
                    *moves = rest.trim_start();
                    movement.parse().ok()
                }
            })
            .collect();
            Ok(Input::Position { position, moves })
        } else if starts_with_token(src, "go") {
            todo!()
        } else if starts_with_token(src, "stop") {
            Ok(Input::Stop)
        } else if starts_with_token(src, "ponderhit") {
            Ok(Input::PonderHit)
        } else if starts_with_token(src, "quit") {
            Ok(Input::Quit)
        } else if starts_with_token(src, "repl") {
            Ok(Input::Repl)
        } else {
            Err(ParseInputError::UnknownCommand(
                extract_prefix_token(src).into(),
            ))
        }
    }
    pub fn from_str(src: &'a str) -> Result<Self, Vec<ParseInputError>> {
        let mut errors = Vec::new();
        for (i, _) in src.char_indices() {
            match Input::from_str_from_start(&src[i..]) {
                Ok(input) => return Ok(input),
                Err(err) => errors.push(err),
            }
        }
        Err(errors)
    }
}
impl Display for Input<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Input::Uci => write!(f, "uci")?,
            Input::Debug(debug) => {
                let switch = if *debug { "on" } else { "false" };
                write!(f, "debug {switch}")?;
            }
            Input::IsReady => write!(f, "isready")?,
            Input::SetOption { name, value } => {
                write!(f, "setoption name {name}")?;
                if let Some(value) = value {
                    write!(f, " value {value}")?;
                }
            }
            Input::Register(register) => write!(f, "register {register}")?,
            Input::UciNewGame => write!(f, "ucinewgame")?,
            Input::Position { position, moves } => {
                write!(f, "position {position} moves")?;
                for movement in moves {
                    write!(f, " {movement}")?;
                }
            }
            Input::Go(go) => {
                write!(f, "go")?;
                if let Some(search_moves) = &go.search_moves {
                    write!(f, " search_moves")?;
                    for movement in search_moves {
                        write!(f, " {movement}")?;
                    }
                }
                if go.ponder {
                    write!(f, " ponder")?;
                }
                if let Some(w_time) = go.w_time {
                    write!(f, " wtime {}", w_time.as_millis())?;
                }
                if let Some(b_time) = go.b_time {
                    write!(f, " btime {}", b_time.as_millis())?;
                }
                if let Some(w_inc) = go.w_inc {
                    write!(f, " winc {}", w_inc.as_millis())?;
                }
                if let Some(b_inc) = go.b_inc {
                    write!(f, " binc {}", b_inc.as_millis())?;
                }
                if let Some(moves_to_go) = go.moves_to_go {
                    write!(f, " movestogo {moves_to_go}",)?;
                }
                if let Some(depth) = go.depth {
                    write!(f, " depth {depth}",)?;
                }
                if let Some(nodes) = go.nodes {
                    write!(f, " nodes {nodes}",)?;
                }
                if let Some(mate) = go.mate {
                    write!(f, " mate {mate}",)?;
                }
                if let Some(move_time) = go.move_time {
                    write!(f, " movetime {}", move_time.as_millis())?;
                }
                if go.infinite {
                    write!(f, " infinite")?;
                }
            }
            Input::Stop => write!(f, "stop")?,
            Input::PonderHit => write!(f, "ponderhit")?,
            Input::Quit => write!(f, "quit")?,
            Input::Repl => write!(f, "repl")?,
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Position {
    StartPos,
    Fen(Fen),
}
impl TryFrom<Position> for Board {
    type Error = InvalidBoard;

    fn try_from(value: Position) -> Result<Self, Self::Error> {
        match value {
            Position::StartPos => Ok(Board::starting_position()),
            Position::Fen(fen) => fen.board.try_into(),
        }
    }
}
impl Display for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Position::StartPos => write!(f, "startpos")?,
            Position::Fen(fen) => write!(f, "fen {fen}")?,
        }
        Ok(())
    }
}
impl FromStr for Position {
    type Err = ParsePositionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "startpos" {
            Ok(Position::StartPos)
        } else if let Some(src) = strip_prefix_token(s, "fen") {
            Ok(Position::Fen(src.parse()?))
        } else if let Some(src) = strip_prefix_token(s, "startpos") {
            match src.chars().next() {
                Some(c) => Err(ParsePositionError::Unexpected(c)),
                None => Ok(Position::StartPos),
            }
        } else {
            Err(ParsePositionError::UnknownCommand(
                extract_prefix_token(s).into(),
            ))
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Go {
    search_moves: Option<Vec<Lan>>,
    ponder: bool,
    w_time: Option<Duration>,
    b_time: Option<Duration>,
    w_inc: Option<Duration>,
    b_inc: Option<Duration>,

    #[allow(clippy::struct_field_names)]
    moves_to_go: Option<NonZero<u32>>,
    depth: Option<NonZero<u32>>,
    nodes: Option<NonZero<u32>>,
    mate: Option<NonZero<u32>>,
    move_time: Option<Duration>,
    infinite: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseInputError {
    ParsePositionError(ParsePositionError),
    UnknownCommand(Box<str>),
    NotOnOrOff,
    NoName,
}
impl From<ParsePositionError> for ParseInputError {
    fn from(value: ParsePositionError) -> Self {
        ParseInputError::ParsePositionError(value)
    }
}
impl Display for ParseInputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseInputError::ParsePositionError(err) => write!(f, "{err}")?,
            ParseInputError::UnknownCommand(command) => write!(f, "unknown command `{command}`")?,
            ParseInputError::NotOnOrOff => write!(f, "provided string was not `on` or `off`")?,
            ParseInputError::NoName => write!(f, "token `name` was not found")?,
        }
        Ok(())
    }
}
impl Error for ParseInputError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseInputError::ParsePositionError(err) => Some(err),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsePositionError {
    UnknownCommand(Box<str>),
    Unexpected(char),
    ParseFenError(ParseFenError),
}
impl From<ParseFenError> for ParsePositionError {
    fn from(value: ParseFenError) -> Self {
        ParsePositionError::ParseFenError(value)
    }
}
impl Display for ParsePositionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParsePositionError::UnknownCommand(command) => write!(
                f,
                "found `{command}`, `startpos` or `fen` were expected instead"
            )?,
            ParsePositionError::Unexpected(c) => write!(f, "unexpected {c}")?,
            ParsePositionError::ParseFenError(parse_fen_error) => write!(f, "{parse_fen_error}")?,
        }
        Ok(())
    }
}
impl Error for ParsePositionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParsePositionError::ParseFenError(err) => Some(err),
            _ => None,
        }
    }
}
