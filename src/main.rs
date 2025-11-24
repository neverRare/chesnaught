#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]

use crate::{chess::Board, game_tree::GameTree, tui::Tui};

mod chess;
mod fen;
mod game_tree;
mod heuristics;
mod tui;

fn main() {
    let mut board = Board::new();
    let mut game_tree = GameTree::new(board);
    let mut previous_move = None;
    let mut end = false;

    loop {
        if let Some(previous_move) = &previous_move {
            println!(
                "{}",
                Tui {
                    board,
                    highlighted: previous_move
                }
            );
        } else {
            println!(
                "{}",
                Tui {
                    board,
                    highlighted: &[]
                }
            );
        }
        if end {
            break;
        }
        let (movement, advantage) = game_tree.best(5, 0);
        println!("{advantage}");
        print!("idea:");
        for movement in game_tree.line() {
            print!(" {movement}");
        }
        println!();
        if let Some(movement) = movement {
            board.move_piece(movement);
            game_tree.move_piece(movement);
            previous_move = Some([movement.movement.origin, movement.movement.destination]);
        } else {
            end = true;
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
