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

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParseInputError {
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
impl<'a> Input<'a> {
    fn from_str_from_start(src: &'a str) -> Result<Self, ParseInputError> {
        if starts_with_separator(src, "uci") {
            return Ok(Input::Uci);
        } else if starts_with_separator(src, "debug") {
            let src = &src[5..].trim_start();
            if starts_with_separator(src, "on") {
                Ok(Input::Debug(true))
            } else if starts_with_separator(src, "off") {
                Ok(Input::Debug(false))
            } else {
                Err(ParseInputError::NotOnOrOff)
            }
        } else if starts_with_separator(src, "isready") {
            Ok(Input::IsReady)
        } else if starts_with_separator(src, "setoption") {
            let src = src[9..].trim_start();
            if !starts_with_separator(src, "name") {
                return Err(ParseInputError::NoName);
            }
            let src = src[4..].trim_start();
            let Some(separator) = find_separator(src, "value") else {
                return Ok(Input::SetOption {
                    name: src,
                    value: None,
                });
            };
            Ok(Input::SetOption {
                name: src[..separator].trim_end(),
                value: Some(src[(separator + 5)..].trim_start()),
            })
        } else if starts_with_separator(src, "register") {
            Ok(Input::Register(Register::parse(src[8..].trim_start())))
        } else if starts_with_separator(src, "ucinewgame") {
            Ok(Input::UciNewGame)
        } else if starts_with_separator(src, "position") {
            let src = src[8..].trim_start();
            let (move_start, move_end) = match find_separator(src, "moves") {
                Some(i) => (i, i + 5),
                None => (src.len(), src.len()),
            };
            let position = src[..move_start].trim_end().parse()?;
            let move_src = &mut src[move_end..].trim_start();
            let moves = from_fn(|| {
                if *move_src == "" {
                    None
                } else {
                    let index = move_src
                        .find(<char>::is_whitespace)
                        .unwrap_or_else(|| move_src.len());
                    src[..index].parse().ok().map(|value| {
                        *move_src = move_src[index..].trim_start();
                        value
                    })
                }
            })
            .collect();
            Ok(Input::Position { position, moves })
        } else if starts_with_separator(src, "go") {
            todo!()
        } else if starts_with_separator(src, "stop") {
            Ok(Input::Stop)
        } else if starts_with_separator(src, "ponderhit") {
            Ok(Input::PonderHit)
        } else if starts_with_separator(src, "quit") {
            Ok(Input::Quit)
        } else if starts_with_separator(src, "repl") {
            Ok(Input::Repl)
        } else {
            Err(ParseInputError::UnknownCommand(extract_command(src).into()))
        }
    }
    fn from_str(src: &'a str) -> Result<Self, Vec<ParseInputError>> {
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
    fn parse(mut src: &'a str) -> Vec<Self> {
        let src = &mut src;
        from_fn(|| {
            if *src == "" {
                None
            } else {
                Register::partial_parse(src).ok().map(|value| {
                    *src = src.trim_start();
                    value
                })
            }
        })
        .collect()
    }
    // TODO: consider spaces inside provided values
    fn partial_parse(src: &mut &'a str) -> Result<Self, ParseRegisterError> {
        if starts_with_separator(src, "later") {
            *src = &src[5..];
            Ok(Register::Later)
        } else if let Some(command) = src.get(..4)
            && src[4..].chars().next().is_some_and(<char>::is_whitespace)
        {
            let index = match src[4..].trim_start().find(<char>::is_whitespace) {
                Some(index) => index + 5,
                None => src.len(),
            };
            let name = &src[5..index].trim_start();
            *src = &src[index..];
            match command {
                "name" => Ok(Register::Name(name)),
                "code" => Ok(Register::Code(name)),
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
        if s == "startpos" {
            Ok(Position::StartPos)
        } else if starts_with_separator(s, "fen") {
            Ok(Position::Fen(s[3..].trim_start().parse()?))
        } else if starts_with_separator(s, "startpos") {
            match s[8..].trim_start().chars().next() {
                Some(c) => Err(ParsePositionError::Unexpected(c)),
                None => Ok(Position::StartPos),
            }
        } else {
            Err(ParsePositionError::UnknownCommand(
                extract_command(s).into(),
            ))
        }
    }
}
fn starts_with_separator(src: &str, search: &str) -> bool {
    src.starts_with(search)
        && src[search.len()..]
            .chars()
            .next()
            .is_none_or(<char>::is_whitespace)
}
fn find_separator(src: &str, search: &str) -> Option<usize> {
    src.find(search).filter(|i| {
        src[(i + search.len())..]
            .chars()
            .next()
            .is_none_or(<char>::is_whitespace)
            && src[..*i]
                .chars()
                .rev()
                .next()
                .is_none_or(<char>::is_whitespace)
    })
}
fn extract_command(src: &str) -> &str {
    match src.find(<char>::is_whitespace) {
        Some(i) => &src[..i],
        None => src,
    }
}
