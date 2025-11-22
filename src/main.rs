#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use std::{
    error::Error,
    fmt::Display,
    io::{Write, stdin, stdout},
    str::FromStr,
};

use crate::{
    chess::{Board, Coord, ParseCoordError, ParsePieceKindError, PieceKind, PieceWithContext},
    tui::Tui,
};

mod chess;
mod fen;
mod tui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Destination {
    destination: Coord,
    promotion_piece: Option<PieceKind>,
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
            ParseInputError::ParseCoordError(err) => Some(err),
            ParseInputError::ParsePieceKindError(err) => Some(err),
            ParseInputError::InsufficientLength | ParseInputError::UnexpectedSymbol(_) => None,
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
                let promotion_piece = characters.next().map(TryInto::try_into).transpose()?;
                if let Some(c) = characters.next() {
                    return Err(ParseInputError::UnexpectedSymbol(c));
                }
                Ok(Input {
                    origin,
                    destination: Some(Destination {
                        destination,
                        promotion_piece,
                    }),
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
                println!("Error: {err}");
                print_board = false;
                continue;
            }
        };
        let piece = if let Some(piece) = board[input.origin] {
            if piece.color != board.current_player {
                println!("Error: it is {}'s turn", board.current_player);
                print_board = false;
                continue;
            }
            piece
        } else {
            println!("Error: that is an empty square");
            print_board = false;
            continue;
        };
        let piece = PieceWithContext {
            piece,
            position: input.origin,
            board,
        };
        match input.destination {
            Some(destination) => {
                let movement = piece.valid_moves().find(|movement| {
                    movement.movement.destination == destination.destination
                        && movement.promotion_piece == destination.promotion_piece
                });
                let Some(movement) = movement else {
                    println!("Error: invalid move");
                    print_board = false;
                    continue;
                };
                board.move_piece(movement);
            }
            None => highlighted.extend(
                piece
                    .valid_moves()
                    .map(|movement| movement.movement.destination),
            ),
        }
    }
}
#[macro_export]
macro_rules! coord_x {
    ("a") => {
        0
    };
    ("b") => {
        1
    };
    ("c") => {
        2
    };
    ("d") => {
        3
    };
    ("e") => {
        4
    };
    ("f") => {
        5
    };
    ("g") => {
        6
    };
    ("h") => {
        7
    };
}
#[macro_export]
macro_rules! coord_y {
    ("8") => {
        0
    };
    ("7") => {
        1
    };
    ("6") => {
        2
    };
    ("5") => {
        3
    };
    ("4") => {
        4
    };
    ("3") => {
        5
    };
    ("2") => {
        6
    };
    ("1") => {
        7
    };
}
#[macro_export]
macro_rules! coord {
    ("a8") => {
        $crate::chess::Coord { x: 0, y: 0 }
    };
    ("a7") => {
        $crate::chess::Coord { x: 0, y: 1 }
    };
    ("a6") => {
        $crate::chess::Coord { x: 0, y: 2 }
    };
    ("a5") => {
        $crate::chess::Coord { x: 0, y: 3 }
    };
    ("a4") => {
        $crate::chess::Coord { x: 0, y: 4 }
    };
    ("a3") => {
        $crate::chess::Coord { x: 0, y: 5 }
    };
    ("a2") => {
        $crate::chess::Coord { x: 0, y: 6 }
    };
    ("a1") => {
        $crate::chess::Coord { x: 0, y: 7 }
    };
    ("b8") => {
        $crate::chess::Coord { x: 1, y: 0 }
    };
    ("b7") => {
        $crate::chess::Coord { x: 1, y: 1 }
    };
    ("b6") => {
        $crate::chess::Coord { x: 1, y: 2 }
    };
    ("b5") => {
        $crate::chess::Coord { x: 1, y: 3 }
    };
    ("b4") => {
        $crate::chess::Coord { x: 1, y: 4 }
    };
    ("b3") => {
        $crate::chess::Coord { x: 1, y: 5 }
    };
    ("b2") => {
        $crate::chess::Coord { x: 1, y: 6 }
    };
    ("b1") => {
        $crate::chess::Coord { x: 1, y: 7 }
    };
    ("c8") => {
        $crate::chess::Coord { x: 2, y: 0 }
    };
    ("c7") => {
        $crate::chess::Coord { x: 2, y: 1 }
    };
    ("c6") => {
        $crate::chess::Coord { x: 2, y: 2 }
    };
    ("c5") => {
        $crate::chess::Coord { x: 2, y: 3 }
    };
    ("c4") => {
        $crate::chess::Coord { x: 2, y: 4 }
    };
    ("c3") => {
        $crate::chess::Coord { x: 2, y: 5 }
    };
    ("c2") => {
        $crate::chess::Coord { x: 2, y: 6 }
    };
    ("c1") => {
        $crate::chess::Coord { x: 2, y: 7 }
    };
    ("d8") => {
        $crate::chess::Coord { x: 3, y: 0 }
    };
    ("d7") => {
        $crate::chess::Coord { x: 3, y: 1 }
    };
    ("d6") => {
        $crate::chess::Coord { x: 3, y: 2 }
    };
    ("d5") => {
        $crate::chess::Coord { x: 3, y: 3 }
    };
    ("d4") => {
        $crate::chess::Coord { x: 3, y: 4 }
    };
    ("d3") => {
        $crate::chess::Coord { x: 3, y: 5 }
    };
    ("d2") => {
        $crate::chess::Coord { x: 3, y: 6 }
    };
    ("d1") => {
        $crate::chess::Coord { x: 3, y: 7 }
    };
    ("e8") => {
        $crate::chess::Coord { x: 4, y: 0 }
    };
    ("e7") => {
        $crate::chess::Coord { x: 4, y: 1 }
    };
    ("e6") => {
        $crate::chess::Coord { x: 4, y: 2 }
    };
    ("e5") => {
        $crate::chess::Coord { x: 4, y: 3 }
    };
    ("e4") => {
        $crate::chess::Coord { x: 4, y: 4 }
    };
    ("e3") => {
        $crate::chess::Coord { x: 4, y: 5 }
    };
    ("e2") => {
        $crate::chess::Coord { x: 4, y: 6 }
    };
    ("e1") => {
        $crate::chess::Coord { x: 4, y: 7 }
    };
    ("f8") => {
        $crate::chess::Coord { x: 5, y: 0 }
    };
    ("f7") => {
        $crate::chess::Coord { x: 5, y: 1 }
    };
    ("f6") => {
        $crate::chess::Coord { x: 5, y: 2 }
    };
    ("f5") => {
        $crate::chess::Coord { x: 5, y: 3 }
    };
    ("f4") => {
        $crate::chess::Coord { x: 5, y: 4 }
    };
    ("f3") => {
        $crate::chess::Coord { x: 5, y: 5 }
    };
    ("f2") => {
        $crate::chess::Coord { x: 5, y: 6 }
    };
    ("f1") => {
        $crate::chess::Coord { x: 5, y: 7 }
    };
    ("g8") => {
        $crate::chess::Coord { x: 6, y: 0 }
    };
    ("g7") => {
        $crate::chess::Coord { x: 6, y: 1 }
    };
    ("g6") => {
        $crate::chess::Coord { x: 6, y: 2 }
    };
    ("g5") => {
        $crate::chess::Coord { x: 6, y: 3 }
    };
    ("g4") => {
        $crate::chess::Coord { x: 6, y: 4 }
    };
    ("g3") => {
        $crate::chess::Coord { x: 6, y: 5 }
    };
    ("g2") => {
        $crate::chess::Coord { x: 6, y: 6 }
    };
    ("g1") => {
        $crate::chess::Coord { x: 6, y: 7 }
    };
    ("h8") => {
        $crate::chess::Coord { x: 7, y: 0 }
    };
    ("h7") => {
        $crate::chess::Coord { x: 7, y: 1 }
    };
    ("h6") => {
        $crate::chess::Coord { x: 7, y: 2 }
    };
    ("h5") => {
        $crate::chess::Coord { x: 7, y: 3 }
    };
    ("h4") => {
        $crate::chess::Coord { x: 7, y: 4 }
    };
    ("h3") => {
        $crate::chess::Coord { x: 7, y: 5 }
    };
    ("h2") => {
        $crate::chess::Coord { x: 7, y: 6 }
    };
    ("h1") => {
        $crate::chess::Coord { x: 7, y: 7 }
    };
}
