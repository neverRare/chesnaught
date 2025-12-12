#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
// TODO: remove this when the engine is fully implemented
#![allow(dead_code, reason = "work in progress code")]

use std::io::{self, stderr, stdin, stdout};

use crate::repl::repl;

mod board;
mod board_display;
mod castling_right;
mod color;
mod coord;
mod end_state;
mod error;
mod fen;
mod game_tree;
mod heuristics;
mod piece;
mod repl;

fn main() -> io::Result<()> {
    let mut output = stdout();
    let mut error = stderr();
    let lock = (!cfg!(debug_assertions)).then(|| (output.lock(), error.lock()));
    repl(&mut stdin().lock(), &mut output, &mut error)?;
    drop(lock);
    Ok(())
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
        $crate::coord::Coord::new(0, 0)
    };
    ("a7") => {
        $crate::coord::Coord::new(0, 1)
    };
    ("a6") => {
        $crate::coord::Coord::new(0, 2)
    };
    ("a5") => {
        $crate::coord::Coord::new(0, 3)
    };
    ("a4") => {
        $crate::coord::Coord::new(0, 4)
    };
    ("a3") => {
        $crate::coord::Coord::new(0, 5)
    };
    ("a2") => {
        $crate::coord::Coord::new(0, 6)
    };
    ("a1") => {
        $crate::coord::Coord::new(0, 7)
    };
    ("b8") => {
        $crate::coord::Coord::new(1, 0)
    };
    ("b7") => {
        $crate::coord::Coord::new(1, 1)
    };
    ("b6") => {
        $crate::coord::Coord::new(1, 2)
    };
    ("b5") => {
        $crate::coord::Coord::new(1, 3)
    };
    ("b4") => {
        $crate::coord::Coord::new(1, 4)
    };
    ("b3") => {
        $crate::coord::Coord::new(1, 5)
    };
    ("b2") => {
        $crate::coord::Coord::new(1, 6)
    };
    ("b1") => {
        $crate::coord::Coord::new(1, 7)
    };
    ("c8") => {
        $crate::coord::Coord::new(2, 0)
    };
    ("c7") => {
        $crate::coord::Coord::new(2, 1)
    };
    ("c6") => {
        $crate::coord::Coord::new(2, 2)
    };
    ("c5") => {
        $crate::coord::Coord::new(2, 3)
    };
    ("c4") => {
        $crate::coord::Coord::new(2, 4)
    };
    ("c3") => {
        $crate::coord::Coord::new(2, 5)
    };
    ("c2") => {
        $crate::coord::Coord::new(2, 6)
    };
    ("c1") => {
        $crate::coord::Coord::new(2, 7)
    };
    ("d8") => {
        $crate::coord::Coord::new(3, 0)
    };
    ("d7") => {
        $crate::coord::Coord::new(3, 1)
    };
    ("d6") => {
        $crate::coord::Coord::new(3, 2)
    };
    ("d5") => {
        $crate::coord::Coord::new(3, 3)
    };
    ("d4") => {
        $crate::coord::Coord::new(3, 4)
    };
    ("d3") => {
        $crate::coord::Coord::new(3, 5)
    };
    ("d2") => {
        $crate::coord::Coord::new(3, 6)
    };
    ("d1") => {
        $crate::coord::Coord::new(3, 7)
    };
    ("e8") => {
        $crate::coord::Coord::new(4, 0)
    };
    ("e7") => {
        $crate::coord::Coord::new(4, 1)
    };
    ("e6") => {
        $crate::coord::Coord::new(4, 2)
    };
    ("e5") => {
        $crate::coord::Coord::new(4, 3)
    };
    ("e4") => {
        $crate::coord::Coord::new(4, 4)
    };
    ("e3") => {
        $crate::coord::Coord::new(4, 5)
    };
    ("e2") => {
        $crate::coord::Coord::new(4, 6)
    };
    ("e1") => {
        $crate::coord::Coord::new(4, 7)
    };
    ("f8") => {
        $crate::coord::Coord::new(5, 0)
    };
    ("f7") => {
        $crate::coord::Coord::new(5, 1)
    };
    ("f6") => {
        $crate::coord::Coord::new(5, 2)
    };
    ("f5") => {
        $crate::coord::Coord::new(5, 3)
    };
    ("f4") => {
        $crate::coord::Coord::new(5, 4)
    };
    ("f3") => {
        $crate::coord::Coord::new(5, 5)
    };
    ("f2") => {
        $crate::coord::Coord::new(5, 6)
    };
    ("f1") => {
        $crate::coord::Coord::new(5, 7)
    };
    ("g8") => {
        $crate::coord::Coord::new(6, 0)
    };
    ("g7") => {
        $crate::coord::Coord::new(6, 1)
    };
    ("g6") => {
        $crate::coord::Coord::new(6, 2)
    };
    ("g5") => {
        $crate::coord::Coord::new(6, 3)
    };
    ("g4") => {
        $crate::coord::Coord::new(6, 4)
    };
    ("g3") => {
        $crate::coord::Coord::new(6, 5)
    };
    ("g2") => {
        $crate::coord::Coord::new(6, 6)
    };
    ("g1") => {
        $crate::coord::Coord::new(6, 7)
    };
    ("h8") => {
        $crate::coord::Coord::new(7, 0)
    };
    ("h7") => {
        $crate::coord::Coord::new(7, 1)
    };
    ("h6") => {
        $crate::coord::Coord::new(7, 2)
    };
    ("h5") => {
        $crate::coord::Coord::new(7, 3)
    };
    ("h4") => {
        $crate::coord::Coord::new(7, 4)
    };
    ("h3") => {
        $crate::coord::Coord::new(7, 5)
    };
    ("h2") => {
        $crate::coord::Coord::new(7, 6)
    };
    ("h1") => {
        $crate::coord::Coord::new(7, 7)
    };
}
