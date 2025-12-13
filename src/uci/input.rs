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
impl<'a> Input<'a> {
    fn from_str_strict(src: &'a str) -> Option<Self> {
        if starts_with_separator(src, "uci") {
            return Some(Input::Uci);
        } else if starts_with_separator(src, "debug") {
            let src = &src[5..].trim_start();
            if starts_with_separator(src, "on") {
                Some(Input::Debug(true))
            } else if starts_with_separator(src, "off") {
                Some(Input::Debug(false))
            } else {
                None
            }
        } else if starts_with_separator(src, "isready") {
            Some(Input::IsReady)
        } else if starts_with_separator(src, "setoption") {
            let src = src[9..].trim_start();
            if !starts_with_separator(src, "name") {
                return None;
            }
            let src = src[4..].trim_start();
            let Some(separator) = find_separator(src, "value") else {
                return Some(Input::SetOption {
                    name: src,
                    value: None,
                });
            };
            Some(Input::SetOption {
                name: src[..separator].trim_end(),
                value: Some(src[(separator + 5)..].trim_start()),
            })
        } else if starts_with_separator(src, "register") {
            Some(Input::Register(Register::parse(src[8..].trim_start())))
        } else if starts_with_separator(src, "ucinewgame") {
            Some(Input::UciNewGame)
        } else if starts_with_separator(src, "position") {
            let src = src[8..].trim_start();
            let (move_start, move_end) = match find_separator(src, "moves") {
                Some(i) => (i, i + 5),
                None => (src.len(), src.len()),
            };
            let Ok(position) = src[..move_start].trim_end().parse() else {
                return None;
            };
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
            Some(Input::Position { position, moves })
        } else if starts_with_separator(src, "go") {
            todo!()
        } else if starts_with_separator(src, "stop") {
            Some(Input::Stop)
        } else if starts_with_separator(src, "ponderhit") {
            Some(Input::PonderHit)
        } else if starts_with_separator(src, "quit") {
            Some(Input::Quit)
        } else if starts_with_separator(src, "repl") {
            Some(Input::Repl)
        } else {
            None
        }
    }
    fn from_str(src: &'a str) -> Option<Self> {
        for (i, _) in src.char_indices() {
            if let Some(input) = Input::from_str_strict(&src[i..]) {
                return Some(input);
            }
        }
        None
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
        if s == "startpos" {
            Ok(Position::StartPos)
        } else if starts_with_separator(s, "fen") {
            Ok(Position::Fen(s[3..].trim_start().parse()?))
        } else if starts_with_separator(s, "startpos") {
            match s[9..].chars().next() {
                Some(c) => Err(ParsePositionError::Unexpected(c)),
                None => Ok(Position::StartPos),
            }
        } else {
            Err(ParsePositionError::UnknownCommand)
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
