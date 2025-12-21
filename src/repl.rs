use rand::random_range;
use rustc_hash::FxHashSet;

use crate::{
    board::{Board, Lan, ParseLanError},
    board_display::BoardDisplay,
    color::Color,
    coord::Coord,
    fen::{Fen, ParseFenError},
    game_tree::{GameTree, Table},
    misc::{MEBIBYTES, strip_prefix_token},
};
use std::{
    collections::HashSet,
    error::Error,
    fmt::{self, Display, Formatter, Write as _},
    io::{BufRead, Write, stderr, stdin, stdout},
    num::ParseIntError,
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Input {
    Help,
    Flip,
    Restart,
    StartChess960,
    Quit,
    Import(Fen),
    ExportFen,
    Coord(Coord),
    Move(Lan),
    Bot(u32),
}
impl Display for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Input::Help => write!(f, "help")?,
            Input::Flip => write!(f, "flip")?,
            Input::Restart => write!(f, "restart")?,
            Input::StartChess960 => write!(f, "start chess960")?,
            Input::Quit => write!(f, "quit")?,
            Input::Import(fen) => write!(f, "import {fen}")?,
            Input::ExportFen => write!(f, "fen")?,
            Input::Coord(position) => write!(f, "{position}")?,
            Input::Move(movement) => write!(f, "{movement}")?,
            Input::Bot(depth) => write!(f, "bot {depth}")?,
        }
        Ok(())
    }
}
impl FromStr for Input {
    type Err = ParseInputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "help" => Ok(Input::Help),
            "flip" => Ok(Input::Flip),
            "restart" => Ok(Input::Restart),
            "start chess960" => Ok(Input::StartChess960),
            "quit" => Ok(Input::Quit),
            "fen" => Ok(Input::ExportFen),
            s => {
                if let Some(s) = strip_prefix_token(s, "import") {
                    Ok(Input::Import(s.parse()?))
                } else if let Some(s) = strip_prefix_token(s, "bot") {
                    Ok(Input::Bot(s.parse()?))
                } else if let Ok(position) = s.parse() {
                    Ok(Input::Coord(position))
                } else {
                    Ok(Input::Move(s.parse()?))
                }
            }
        }
    }
}
#[allow(
    clippy::too_many_lines,
    reason = "further decomposition could potentially hurt readability"
)]
pub fn repl() {
    let input = stdin().lock();
    let mut output = stdout().lock();
    let mut error = stderr().lock();

    let mut lines = input.lines();

    let mut board = Board::starting_position();
    let mut info = String::new();
    let mut highlighted = Vec::new();
    let mut valid_moves: FxHashSet<_> = HashSet::default();
    let mut update = true;
    let mut view = Color::White;
    let mut first_time = true;
    let mut game_tree = GameTree::new(board.clone());
    let mut table = Table::new(4096 * MEBIBYTES / Table::ELEMENT_SIZE);
    loop {
        if update {
            valid_moves.clear();
            info.clear();
            match board.valid_moves() {
                Ok(moves) => {
                    valid_moves.extend(moves.flat_map(|movement| movement.as_lan_iter(&board)));
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
        writeln!(
            output,
            "{}",
            BoardDisplay {
                board: &board,
                view,
                show_coordinates: true,
                highlighted: &highlighted,
                info: &info,
            },
        )
        .unwrap();
        loop {
            write!(output, "> ").unwrap();
            output.flush().unwrap();
            let text = lines.next().unwrap().unwrap();
            let input = match text.trim().parse() {
                Ok(input) => input,
                Err(err) => {
                    writeln!(error, "Error: {err}").unwrap();
                    writeln!(error, "for available command, enter `help`").unwrap();
                    continue;
                }
            };
            match input {
                Input::Help => {
                    writeln!(output, "flip           - flip the board").unwrap();
                    writeln!(output, "restart        - reset to starting position").unwrap();
                    writeln!(output, "start chess960 - start a new chess960 game").unwrap();
                    writeln!(output, "quit           - quit the game").unwrap();
                    writeln!(output, "import <fen>   - import a position").unwrap();
                    writeln!(output, "fen            - export the position as fen").unwrap();
                    writeln!(output, "e2             - view valid moves").unwrap();
                    writeln!(output, "e2e4           - play the move").unwrap();
                    writeln!(output, "e7e8q          - move and promote").unwrap();
                    writeln!(output, "e1g1 (or e1h1) - perform castling").unwrap();
                    writeln!(output, "bot <depth>    - let a bot play").unwrap();
                }
                Input::Flip => {
                    view = !view;
                }
                Input::Restart => {
                    board = Board::starting_position();
                    game_tree = GameTree::new(board.clone());
                    update = true;
                    highlighted.clear();
                }
                Input::StartChess960 => {
                    board = Board::chess960(random_range(0..960));
                    game_tree = GameTree::new(board.clone());
                    update = true;
                    highlighted.clear();
                }
                Input::Quit => return,
                Input::Import(fen) => {
                    board = match fen.board.try_into() {
                        Ok(board) => board,
                        Err(err) => {
                            writeln!(error, "Error: {err}").unwrap();
                            continue;
                        }
                    };
                    game_tree = GameTree::new(board.clone());
                    update = true;
                    highlighted.clear();
                }
                Input::ExportFen => {
                    writeln!(
                        output,
                        "{}",
                        Fen {
                            board: board.as_hashable(),
                            half_move: 0,
                            full_move: 1
                        }
                    )
                    .unwrap();
                }
                Input::Coord(position) => {
                    if let Some(piece) = board.index(position) {
                        if piece.color() != board.current_player() {
                            writeln!(error, "Error: It is {}'s turn", board.current_player())
                                .unwrap();
                            continue;
                        }
                        highlighted.clear();
                        highlighted.extend(
                            valid_moves
                                .iter()
                                .copied()
                                .filter(|movement| movement.origin == position)
                                .map(|movement| movement.destination),
                        );
                    } else {
                        writeln!(error, "Error: No piece found on {position}").unwrap();
                        continue;
                    }
                }
                Input::Move(lan) => {
                    let Some(movement) = valid_moves.get(&lan) else {
                        writeln!(error, "Error: {lan} is an invalid move").unwrap();
                        continue;
                    };
                    board.move_lan(*movement);
                    game_tree.move_piece(*movement);
                    highlighted.clear();
                    highlighted.push(lan.origin);
                    highlighted.push(lan.destination);
                    update = true;
                }
                Input::Bot(depth) => {
                    table.clear_allocation();
                    game_tree.calculate(depth, &mut table);
                    let movement = game_tree.best_move().unwrap();
                    board.move_lan(movement);
                    game_tree.move_piece(movement);
                    update = true;
                }
            }
            break;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParseInputError {
    Fen(ParseFenError),
    Move(ParseLanError),
    Int(ParseIntError),
}
impl From<ParseFenError> for ParseInputError {
    fn from(value: ParseFenError) -> Self {
        ParseInputError::Fen(value)
    }
}
impl From<ParseLanError> for ParseInputError {
    fn from(value: ParseLanError) -> Self {
        ParseInputError::Move(value)
    }
}
impl From<ParseIntError> for ParseInputError {
    fn from(value: ParseIntError) -> Self {
        ParseInputError::Int(value)
    }
}
impl Display for ParseInputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseInputError::Fen(err) => write!(f, "{err}").unwrap(),
            ParseInputError::Move(err) => write!(f, "{err}").unwrap(),
            ParseInputError::Int(err) => write!(f, "{err}").unwrap(),
        }
        Ok(())
    }
}
impl Error for ParseInputError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ParseInputError::Fen(err) => Some(err),
            ParseInputError::Move(err) => Some(err),
            ParseInputError::Int(err) => Some(err),
        }
    }
}
