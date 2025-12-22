use std::{
    convert::Infallible,
    error::Error,
    fmt::{self, Display, Formatter},
    iter::from_fn,
    num::NonZero,
    str::FromStr,
    time::Duration,
};

use crate::{
    board::{Board, InvalidBoard, Lan},
    color::Color,
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
    Fuzz,
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
            let (position, moves) = split_by_token(src, "moves").unwrap_or((src, ""));
            let position = position.parse()?;
            let moves = moves
                .split(<char>::is_whitespace)
                .filter(|token| !token.is_empty())
                .map_while(|token| token.parse().ok())
                .collect();
            Ok(Input::Position { position, moves })
        } else if let Some(src) = strip_prefix_token(src, "go") {
            Ok(Input::Go(src.parse().unwrap()))
        } else if starts_with_token(src, "stop") {
            Ok(Input::Stop)
        } else if starts_with_token(src, "ponderhit") {
            Ok(Input::PonderHit)
        } else if starts_with_token(src, "quit") {
            Ok(Input::Quit)
        } else if starts_with_token(src, "repl") {
            Ok(Input::Repl)
        } else if starts_with_token(src, "fuzz") {
            Ok(Input::Fuzz)
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
            Input::Go(go) => write!(f, "go {go}")?,
            Input::Stop => write!(f, "stop")?,
            Input::PonderHit => write!(f, "ponderhit")?,
            Input::Quit => write!(f, "quit")?,
            Input::Repl => write!(f, "repl")?,
            Input::Fuzz => write!(f, "fuzz")?,
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Position {
    StartPos,
    Fen(Fen),
}
impl Position {
    pub fn board(self) -> Result<Board, InvalidBoard> {
        match self {
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Go {
    pub search_moves: Option<Vec<Lan>>,
    pub ponder: bool,
    pub w_time: Option<Duration>,
    pub b_time: Option<Duration>,
    pub w_inc: Option<Duration>,
    pub b_inc: Option<Duration>,

    #[allow(clippy::struct_field_names)]
    pub moves_to_go: Option<NonZero<u32>>,
    pub depth: Option<NonZero<u32>>,
    pub nodes: Option<NonZero<u32>>,
    pub mate: Option<NonZero<u32>>,
    pub move_time: Option<Duration>,
    pub infinite: bool,
}
impl Go {
    pub fn estimate_move_time(&self, board: &Board) -> Option<Duration> {
        if let Some(move_time) = self.move_time {
            Some(move_time)
        } else if self.infinite {
            None
        } else {
            let (time, inc) = match board.current_player() {
                Color::White => (self.w_time, self.w_inc),
                Color::Black => (self.b_time, self.b_inc),
            };
            if let Some(time) = time {
                let total_moves = board.estimate_moves_left();
                let moves_to_go = if let Some(moves_to_go) = self.moves_to_go {
                    #[allow(clippy::cast_precision_loss, reason = "we don't need the precision")]
                    <f32>::min(moves_to_go.get() as f32, total_moves)
                } else {
                    total_moves
                };
                let estimated_time = time.div_f32(moves_to_go) + inc.unwrap_or_default();
                if estimated_time > time {
                    Some(time / 2)
                } else {
                    Some(estimated_time)
                }
            } else {
                None
            }
        }
    }
}
impl Display for Go {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut first = true;
        let mut add_space = |f: &mut Formatter<'_>| -> fmt::Result {
            if !first {
                write!(f, " ")?;
            }
            first = false;
            Ok(())
        };
        if let Some(search_moves) = &self.search_moves {
            add_space(f)?;
            write!(f, "searchmoves")?;
            for movement in search_moves {
                write!(f, " {movement}")?;
            }
        }
        if self.ponder {
            add_space(f)?;
            write!(f, "ponder")?;
        }
        if let Some(w_time) = self.w_time {
            add_space(f)?;
            write!(f, "wtime {}", w_time.as_millis())?;
        }
        if let Some(b_time) = self.b_time {
            add_space(f)?;
            write!(f, "btime {}", b_time.as_millis())?;
        }
        if let Some(w_inc) = self.w_inc {
            add_space(f)?;
            write!(f, "winc {}", w_inc.as_millis())?;
        }
        if let Some(b_inc) = self.b_inc {
            add_space(f)?;
            write!(f, "binc {}", b_inc.as_millis())?;
        }
        if let Some(moves_to_go) = self.moves_to_go {
            add_space(f)?;
            write!(f, "movestogo {moves_to_go}")?;
        }
        if let Some(depth) = self.depth {
            add_space(f)?;
            write!(f, "depth {depth}")?;
        }
        if let Some(nodes) = self.nodes {
            add_space(f)?;
            write!(f, "nodes {nodes}")?;
        }
        if let Some(mate) = self.mate {
            add_space(f)?;
            write!(f, "mate {mate}")?;
        }
        if let Some(move_time) = self.move_time {
            add_space(f)?;
            write!(f, "movetime {}", move_time.as_millis())?;
        }
        if self.infinite {
            add_space(f)?;
            write!(f, "infinite")?;
        }
        Ok(())
    }
}
impl FromStr for Go {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut go = Go::default();
        let mut tokens = s
            .split(<char>::is_whitespace)
            .filter(|token| !token.is_empty())
            .peekable();
        while let Some(token) = tokens.next() {
            match token {
                "searchmoves" => {
                    let moves = from_fn(|| {
                        tokens.peek().copied().and_then(|token| {
                            token.parse().ok().inspect(|_| {
                                tokens.next();
                            })
                        })
                    })
                    .collect();
                    go.search_moves = Some(moves);
                }
                "ponder" => go.ponder = true,
                prefix @ ("wtime" | "btime" | "winc" | "binc" | "movetime") => {
                    let Some(time) = tokens.next().and_then(|token| token.parse().ok()) else {
                        continue;
                    };
                    let time = Duration::from_millis(time);
                    match prefix {
                        "wtime" => go.w_time = Some(time),
                        "btime" => go.b_time = Some(time),
                        "winc" => go.w_inc = Some(time),
                        "binc" => go.b_inc = Some(time),
                        "movetime" => go.move_time = Some(time),
                        _ => unreachable!(),
                    }
                }
                prefix @ ("movestogo" | "depth" | "nodes" | "mate") => {
                    let Some(count) = tokens.next().and_then(|token| token.parse().ok()) else {
                        continue;
                    };
                    match prefix {
                        "movestogo" => go.moves_to_go = Some(count),
                        "depth" => go.depth = Some(count),
                        "nodes" => go.nodes = Some(count),
                        "mate" => go.mate = Some(count),
                        _ => unreachable!(),
                    }
                }
                "infinite" => go.infinite = true,
                _ => (),
            }
        }
        Ok(go)
    }
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
#[cfg(test)]
mod test {

    use crate::uci::input::{Input, Position};

    #[test]
    fn parse_position() {
        let input = Input::from_str("position startpos moves e2e4").unwrap();
        assert_eq!(
            input,
            Input::Position {
                position: Position::StartPos,
                moves: vec!["e2e4".parse().unwrap()]
            }
        );
    }
}
