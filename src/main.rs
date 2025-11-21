#![forbid(unsafe_code)]

use std::{
    error::Error,
    fmt::Display,
    io::{Write, stdin, stdout},
    str::FromStr,
};

use crate::{
    chess::{
        Board, Coord, Move, ParseCoordError, ParsePieceKindError, PieceKind, PieceWithContext,
    },
    tui::Tui,
};

mod chess;
mod tui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Destination {
    destination: Coord,
    kind: Option<PieceKind>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Input {
    origin: Coord,
    destination: Option<Destination>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseInputError {
    InsufficientLength,
    ParseCoordError(ParseCoordError),
    ParsePieceKindError(ParsePieceKindError),
    UnexpectedSymbol(char),
}
impl From<ParseCoordError> for ParseInputError {
    fn from(value: ParseCoordError) -> Self {
        ParseInputError::ParseCoordError(value)
    }
}
impl From<ParsePieceKindError> for ParseInputError {
    fn from(value: ParsePieceKindError) -> Self {
        ParseInputError::ParsePieceKindError(value)
    }
}
impl Display for ParseInputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseInputError::InsufficientLength => write!(f, "insufficient text length")?,
            ParseInputError::ParseCoordError(err) => write!(f, "{err}")?,
            ParseInputError::ParsePieceKindError(err) => write!(f, "{err}")?,
            ParseInputError::UnexpectedSymbol(c) => write!(f, "unexpected `${c}`")?,
        }
        Ok(())
    }
}
impl Error for ParseInputError {
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            ParseInputError::InsufficientLength => None,
            ParseInputError::ParseCoordError(err) => Some(err),
            ParseInputError::ParsePieceKindError(err) => Some(err),
            ParseInputError::UnexpectedSymbol(_) => None,
        }
    }
}
impl FromStr for Input {
    type Err = ParseInputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut characters = s.chars().fuse();
        let x = characters
            .next()
            .ok_or(ParseInputError::InsufficientLength)?;
        let y = characters
            .next()
            .ok_or(ParseInputError::InsufficientLength)?;
        let origin = Coord::from_char(x, y)?;
        match characters.next() {
            Some(x) => {
                let y = characters
                    .next()
                    .ok_or(ParseInputError::InsufficientLength)?;
                let destination = Coord::from_char(x, y)?;
                let kind = characters.next().map(TryInto::try_into).transpose()?;
                if let Some(c) = characters.next() {
                    return Err(ParseInputError::UnexpectedSymbol(c));
                }
                Ok(Input {
                    origin,
                    destination: Some(Destination { destination, kind }),
                })
            }
            None => Ok(Input {
                origin,
                destination: None,
            }),
        }
    }
}

fn main() {
    let mut print_board = true;
    let mut board = Board::new();
    let mut highlighted = Vec::new();
    println!(
        "To play, enter the piece origin and destination coordinates like e2e4 without space in between."
    );
    println!(
        "To view valid moves, enter the piece coordinates. To choose, you still have to specify the origin coordinates like in previous instruction."
    );
    println!("For promotion, include the desired promotion piece after the input like e7e8q");
    println!();
    loop {
        let state = board.state();
        if print_board {
            match state {
                Some(state) => println!("{state}"),
                None => println!("{} to play", board.current_player),
            }
            println!(
                "{}",
                Tui {
                    board,
                    highlighted: &highlighted
                }
            );
        }
        highlighted = Vec::new();
        print_board = true;
        if state.is_some() {
            break;
        }
        print!("> ");
        stdout().flush().unwrap();
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        let input: Input = match input.trim().parse() {
            Ok(input) => input,
            Err(err) => {
                println!("Error: {}", err);
                print_board = false;
                continue;
            }
        };
        let piece = match board[input.origin] {
            Some(piece) => {
                if piece.color != board.current_player {
                    println!("Error: it is {}'s turn", board.current_player);
                    print_board = false;
                    continue;
                }
                piece
            }
            None => {
                println!("Error: that is an empty square");
                print_board = false;
                continue;
            }
        };
        let piece = PieceWithContext {
            piece,
            position: input.origin,
            board,
        };
        match input.destination {
            Some(destination) => {
                let movement = piece.valid_moves().find(|movement| {
                    movement.destination() == destination.destination
                        && movement.promotion_piece() == destination.kind
                });
                let movement = match movement {
                    Some(movement) => movement,
                    None => {
                        println!("Error: invalid move");
                        print_board = false;
                        continue;
                    }
                };
                board.move_piece(movement);
            }
            None => highlighted.extend(piece.valid_moves().map(Move::destination)),
        }
    }
}
