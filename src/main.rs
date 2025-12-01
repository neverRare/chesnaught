#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![allow(dead_code)]

use std::{
    collections::HashMap,
    fmt::Write,
    io::{stdin, stdout},
};

use crate::{
    board_display::BoardDisplay,
    chess::{Board, Color},
    fen::Fen,
};

mod board_display;
mod chess;
mod fen;
mod game_tree;
mod heuristics;

fn main() {
    let mut board = Board::starting_position();
    let mut info = String::new();
    let mut highlighted = Vec::new();
    let mut valid_moves = HashMap::new();
    let mut update = true;
    let mut view = Color::White;
    let mut first_time = true;
    loop {
        if update {
            valid_moves.clear();
            info.clear();
            match board.valid_moves() {
                Ok(moves) => {
                    valid_moves.extend(
                        moves.map(|movement| {
                            (movement.as_long_algebraic_notation(&board), movement)
                        }),
                    );
                    writeln!(&mut info, "{} plays", board.current_player()).unwrap();
                }
                Err(end_state) => {
                    writeln!(&mut info, "{end_state}").unwrap();
                }
            }
        }
        if first_time {
            writeln!(&mut info, "type `help` for instructions").unwrap();
            first_time = false;
        }
        update = false;
        print!(
            "{}",
            BoardDisplay {
                board: &board,
                view,
                show_coordinates: true,
                highlighted: &highlighted,
                info: &info,
            },
        );
        loop {
            print!("> ");
            {
                use std::io::Write;
                stdout().flush().unwrap();
            }
            let mut input = String::new();
            stdin().read_line(&mut input).unwrap();

            let input = input.trim();
            if input == "help" {
                println!("flip           - flip the board");
                println!("import <fen>   - import a position");
                println!("exit           - exit the game");
                println!("e2             - view valid moves");
                println!("e2e4           - play the move");
                println!("e7e8q          - move and promote");
                println!("e1g1 (or e1h1) - perform castling");
            } else if input == "show raw moves" {
                board.display_raw_moves();
            } else if input == "exit" {
                return;
            } else if input == "flip" {
                view = !view;
            } else if input.get(0..7) == Some("import ") {
                let fen: Fen = match input[7..].parse() {
                    Ok(fen) => fen,
                    Err(err) => {
                        eprintln!("Error: {err}");
                        continue;
                    }
                };
                board = match fen.board.try_into() {
                    Ok(board) => board,
                    Err(err) => {
                        eprintln!("Error: {err}");
                        continue;
                    }
                };
                update = true;
            } else if let Ok(position) = input.parse() {
                highlighted.clear();
                highlighted.extend(
                    valid_moves
                        .keys()
                        .copied()
                        .filter(|movement| movement.origin == position)
                        .map(|movement| movement.destination),
                );
            } else {
                let long_algebraic_notation = match input.parse() {
                    Ok(movement) => movement,
                    Err(err) => {
                        eprintln!("Error: {err}");
                        continue;
                    }
                };
                let Some(movement) = valid_moves.get(&long_algebraic_notation) else {
                    eprintln!("Error: {input} is an invalid move");
                    continue;
                };
                board.move_piece(movement);
                highlighted.clear();
                highlighted.push(long_algebraic_notation.origin);
                highlighted.push(long_algebraic_notation.destination);
                update = true;
            }
            break;
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
