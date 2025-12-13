use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    iter::from_fn,
    num::NonZero,
    str::FromStr,
    time::Duration,
};

use crate::{
    board::Lan,
    fen::{Fen, ParseFenError},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Input<'a> {
    Uci,
    Debug(bool),
    IsReady,
    SetOption {
        name: &'a str,
        value: Option<&'a str>,
    },
    Register(Vec<Register<'a>>),
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
            Input::Register(register) => {
                write!(f, "register")?;
                for register in register {
                    write!(f, " {register}")?;
                }
            }
            Input::UciNewGame => write!(f, "ucinewgame")?,
            Input::Position { position, moves } => {
                write!(f, "position {position} moves")?;
                for movement in moves {
                    write!(f, " {movement}")?;
                }
            }
            Input::Go {
                search_moves,
                ponder,
                w_time,
                b_time,
                w_inc,
                b_inc,
                moves_to_go,
                depth,
                nodes,
                mate,
                move_time,
                infinite,
            } => {
                write!(f, "go")?;
                if let Some(search_moves) = search_moves {
                    write!(f, " search_moves")?;
                    for movement in search_moves {
                        write!(f, " {movement}")?;
                    }
                }
                if *ponder {
                    write!(f, " ponder")?;
                }
                if let Some(w_time) = w_time {
                    write!(f, " wtime {}", w_time.as_millis())?;
                }
                if let Some(b_time) = b_time {
                    write!(f, " btime {}", b_time.as_millis())?;
                }
                if let Some(w_inc) = w_inc {
                    write!(f, " winc {}", w_inc.as_millis())?;
                }
                if let Some(b_inc) = b_inc {
                    write!(f, " binc {}", b_inc.as_millis())?;
                }
                if let Some(moves_to_go) = moves_to_go {
                    write!(f, " movestogo {moves_to_go}",)?;
                }
                if let Some(depth) = depth {
                    write!(f, " depth {depth}",)?;
                }
                if let Some(nodes) = nodes {
                    write!(f, " nodes {nodes}",)?;
                }
                if let Some(mate) = mate {
                    write!(f, " mate {mate}",)?;
                }
                if let Some(move_time) = move_time {
                    write!(f, " movetime {}", move_time.as_millis())?;
                }
                if *infinite {
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ParseRegisterError {
    UnknownCommand,
    NoName,
    NoCode,
}
impl Display for ParseRegisterError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseRegisterError::UnknownCommand => {
                write!(f, "provided prefix was not `later`, `name`, or `code`")?
            }
            ParseRegisterError::NoName => write!(f, "no name provided")?,
            ParseRegisterError::NoCode => write!(f, "no code provided")?,
        }
        Ok(())
    }
}
impl Error for ParseRegisterError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Register<'a> {
    Later,
    Name(&'a str),
    Code(&'a str),
}
impl<'a> Register<'a> {
    fn parse(mut src: &'a str) -> Result<Vec<Self>, ParseRegisterError> {
        let src = &mut src;
        from_fn(|| {
            (*src != "").then(|| {
                Register::partial_parse(src).map(|value| {
                    *src = src.trim_start();
                    value
                })
            })
        })
        .collect()
    }
    fn partial_parse(src: &mut &'a str) -> Result<Self, ParseRegisterError> {
        if src.get(..5) == Some("later")
            && src[5..].chars().next().is_none_or(<char>::is_whitespace)
        {
            *src = &src[5..];
            Ok(Register::Later)
        } else if let Some(command) = src.get(..5) {
            let index = match src[5..].find(<char>::is_whitespace) {
                Some(index) => index + 5,
                None => src.len(),
            };
            let name = &src[5..index];
            *src = &src[index..];
            match command {
                "name " => Ok(Register::Name(name)),
                "code " => Ok(Register::Code(name)),
                _ => Err(ParseRegisterError::UnknownCommand),
            }
        } else {
            match *src {
                "name" => Err(ParseRegisterError::NoName),
                "code" => Err(ParseRegisterError::NoCode),
                _ => Err(ParseRegisterError::UnknownCommand),
            }
        }
    }
}
impl Display for Register<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Register::Later => write!(f, "later")?,
            Register::Name(name) => write!(f, "name {name}")?,
            Register::Code(code) => write!(f, "code {code}")?,
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsePositionError {
    UnknownCommand,
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
            ParsePositionError::UnknownCommand => {
                write!(f, "provided prefix was not `startpos` or `fen`")?
            }
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Position {
    StartPos,
    Fen(Fen),
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
        if matches!(s, "startpos" | "startpos ") {
            Ok(Position::StartPos)
        } else if s.get(..4) == Some("fen ") {
            Ok(Position::Fen(s[4..].trim_start().parse()?))
        } else if s.get(..9) == Some("startpos ") {
            Err(ParsePositionError::Unexpected(
                s[9..].chars().next().unwrap(),
            ))
        } else {
            Err(ParsePositionError::UnknownCommand)
        }
    }
}
