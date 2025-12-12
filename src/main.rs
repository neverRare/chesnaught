#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
// TODO: remove this when the engine is fully implemented
#![allow(dead_code, reason = "work in progress code")]

use std::{
    collections::HashMap,
    fmt::Write,
    io::{stdin, stdout},
};

use crate::{board::Board, board_display::BoardDisplay, color::Color, fen::Fen};

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

#[allow(
    clippy::too_many_lines,
    reason = "the state and procedure are very clearly defined; no need to decompose these into separate functions"
)]
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
                println!("reset          - reset to starting position");
                println!("fen            - export the position as fen");
                println!("exit           - exit the game");
                println!("e2             - view valid moves");
                println!("e2e4           - play the move");
                println!("e7e8q          - move and promote");
                println!("e1g1 (or e1h1) - perform castling");
            } else if input == "reset" {
                board = Board::starting_position();
                update = true;
                highlighted.clear();
            } else if input == "exit" {
                return;
            } else if input == "flip" {
                view = !view;
            } else if input == "fen" {
                println!(
                    "{}",
                    Fen {
                        board: board.as_hashable(),
                        half_move: 0,
                        full_move: 1
                    }
                );
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
                highlighted.clear();
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
