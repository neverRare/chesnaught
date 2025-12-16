use std::{
    cell::LazyCell,
    io::{self, BufRead, Write, stderr},
};

use crate::{
    board::Board,
    engine::Engine,
    repl::repl,
    uci::{
        input::Input,
        output::{Info, OptionType, OptionValue, Output},
    },
};

mod input;
mod output;

const CHESS960: &str = "UCI_Chess960";

const CONFIG: [Output; 3] = [
    Output::Id {
        name: "Chesnaught",
        author: "neverRare",
    },
    Output::Option {
        name: CHESS960,
        kind: OptionType::Check,
        default: Some(OptionValue::Bool(false)),
        boundary: None,
    },
    Output::UciOk,
];
pub fn uci_loop(input: &mut impl BufRead, output: &mut impl Write) -> io::Result<()> {
    let mut uci = false;
    let mut debug = false;
    let engine = LazyCell::new(Engine::new);
    let mut board = Board::starting_position();
    let mut new_game = true;
    let mut uci_new_game_available = false;
    loop {
        let mut text = String::new();
        input.read_line(&mut text)?;
        let text = text.trim();
        if text.is_empty() {
            continue;
        }
        let parsed_input = match Input::from_str(text) {
            Ok(input) => input,
            Err(err) => {
                if debug {
                    if err.is_empty() {
                        debug_print(
                            output,
                            "error parsing input but no error information found".to_string(),
                        )?;
                    } else {
                        for err in err {
                            debug_print(output, format!("error: {err}"))?;
                        }
                    }
                }
                continue;
            }
        };
        match parsed_input {
            Input::Uci => {
                for config in CONFIG {
                    writeln!(output, "{config}")?;
                }
                uci = true;
            }
            Input::Debug(b) => {
                if uci {
                    debug = b;
                }
            }
            Input::IsReady => {
                if uci {
                    engine.ready();
                    writeln!(output, "{}", Output::ReadyOk)?;
                }
            }
            Input::SetOption { name, value } => {
                if uci {
                    match name {
                        CHESS960 => {
                            if debug && !matches!(value, Some("true" | "false")) {
                                debug_print(
                                    output,
                                    format!("set {CHESS960} to invalid value; ignoring"),
                                )?;
                            }
                            // The engine can already work on chess960 without telling it to use chess960
                        }
                        name => {
                            if debug {
                                debug_print(output, format!("unknown option `{name}`; ignoring"))?;
                            }
                        }
                    }
                }
            }
            Input::Register(_) => {
                if uci && debug {
                    debug_print(output, "registration is ignored".to_string())?;
                }
            }
            Input::UciNewGame => {
                if uci {
                    new_game = true;
                    uci_new_game_available = true;
                    engine.set_board(Board::starting_position());
                    board = Board::starting_position();
                }
            }
            Input::Position { position, moves } => {
                if uci {
                    if !uci_new_game_available || new_game {
                        board = match position.try_into() {
                            Ok(board) => board,
                            Err(err) => {
                                if debug {
                                    debug_print(output, "error setting up board".to_string())?;
                                    debug_print(output, format!("error: {err}"))?;
                                }
                                Board::starting_position()
                            }
                        };
                        for movement in moves {
                            board.move_piece(&movement);
                        }
                        engine.set_board(board.clone());
                        new_game = false;
                    } else if let Some(movement) = moves.last() {
                        board.move_piece(movement);
                        engine.move_piece(*movement);
                    } else if debug {
                        debug_print(output, "no moves found".to_string())?;
                    }
                }
            }
            Input::Go(go) => {
                if uci {
                    new_game = false;
                    todo!()
                }
            }
            Input::Stop => {
                if uci {
                    engine.stop();
                }
            }
            Input::PonderHit => {
                if uci && debug {
                    debug_print(output, "pondering is unsupported; ignoring".to_string())?;
                }
            }
            Input::Quit => {
                if uci {
                    return Ok(());
                }
            }
            Input::Repl => {
                if !uci {
                    repl(input, output, &mut stderr().lock())?;
                    return Ok(());
                }
            }
        }
    }
}
fn debug_print(output: &mut impl Write, message: String) -> io::Result<()> {
    writeln!(output, "{}", Output::Info(vec![Info::String(message)]))?;
    Ok(())
}
