#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
// #![allow(dead_code, reason = "work in progress code")]

use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    io::{BufRead, stdin},
    str::FromStr,
};

use crate::{fuzz::fuzz, repl::repl, uci::uci_loop};

mod board;
mod board_display;
mod castling_right;
mod color;
mod coord;
mod end_state;
mod engine;
mod fen;
mod fuzz;
mod game_tree;
mod heuristics;
mod misc;
mod piece;
mod repl;
mod simple_board;
mod uci;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Input {
    Uci,
    Repl,
    Fuzz,
}
impl Display for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Input::Uci => write!(f, "uci")?,
            Input::Repl => write!(f, "repl")?,
            Input::Fuzz => write!(f, "fuzz")?,
        }
        Ok(())
    }
}
impl FromStr for Input {
    type Err = ParseInputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "uci" => Ok(Input::Uci),
            "repl" => Ok(Input::Repl),
            "fuzz" => Ok(Input::Fuzz),
            _ => Err(ParseInputError),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ParseInputError;

impl Display for ParseInputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "provided string was not `uci`, `repl`, or `fuzz`")?;
        Ok(())
    }
}
impl Error for ParseInputError {}
fn main() {
    let input = stdin().lock();
    let mut lines = input.lines();
    let input_text = lines.next().unwrap().unwrap();
    let parsed_input = match input_text.trim().parse() {
        Ok(input) => input,
        Err(err) => {
            eprintln!("Error: {err}");
            return;
        }
    };
    drop(lines);
    match parsed_input {
        Input::Uci => uci_loop(),
        Input::Repl => repl(),
        Input::Fuzz => fuzz(),
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
