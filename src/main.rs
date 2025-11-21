#![forbid(unsafe_code)]

use std::io::{Write, stdin, stdout};

use crate::{chess::Board, tui::Tui};

mod chess;
mod tui;

fn main() {
    let mut board = Board::default();
    println!(
        "To play, enter the piece origin and destination coordinates like e2e4 without space in between."
    );
    println!(
        "To view valid moves, enter the piece coordinates. To choose, you still have to specify the origin coordinates like in previous instruction."
    );
    println!();
    loop {
        match board.state() {
            Some(state) => println!("{state}"),
            None => println!("{} to play", board.current_player),
        }
        println!(
            "{}",
            Tui {
                board,
                highlighted: []
            }
        );
        print!("> ");
        stdout().flush().unwrap();
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        let input = input.trim();
    }
}
