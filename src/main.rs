#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
// #![allow(dead_code, reason = "work in progress code")]

use crate::uci::uci_loop;

mod board;
mod board_display;
mod castling_right;
mod color;
mod coord;
mod end_state;
mod engine;
mod fen;
mod game_tree;
mod heuristics;
mod misc;
mod piece;
mod repl;
mod simple_board;
mod uci;

fn main() {
    uci_loop();
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
