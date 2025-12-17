use std::{
    cell::LazyCell,
    io::{self, BufRead, Write, stdin, stdout},
};

use crate::{
    board::Board,
    engine::Engine,
    repl::repl,
    uci::{
        input::Input,
        output::{Boundary, Info, OptionType, OptionValue, Output},
    },
};

mod input;
mod output;

const CHESS960: &str = "UCI_Chess960";

const CONFIG: [Output; 5] = [
    Output::Id {
        name: "Chesnaught",
        author: "neverRare",
    },
    Output::Option {
        name: "Hash",
        kind: OptionType::Spin,
        default: Some(OptionValue::Int(0)),
        boundary: Some(Boundary::Boundary { min: 0, max: 4096 }),
    },
    Output::Option {
        name: "Clear Hash",
        kind: OptionType::Button,
        default: None,
        boundary: None,
    },
    Output::Option {
        name: CHESS960,
        kind: OptionType::Check,
        default: Some(OptionValue::Bool(false)),
        boundary: None,
    },
    Output::UciOk,
];
pub fn uci_loop() -> io::Result<()> {
    let input = stdin().lock();
    let mut output = stdout();

    let mut lines = input.lines();

    let mut uci = false;
    let mut debug = false;
    let mut engine = LazyCell::new(Engine::new);
    let mut board = Board::starting_position();
    let mut new_game = true;
    let mut uci_new_game_available = false;
    loop {
        let text = lines.next().unwrap()?;
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
                            &mut output,
                            "error parsing input but no error information found".to_string(),
                        )?;
                    } else {
                        for err in err {
                            debug_print(&mut output, format!("error: {err}"))?;
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
                                    &mut output,
                                    format!("set {CHESS960} to invalid value; ignoring"),
                                )?;
                            }
                            // The engine can already work on chess960 without telling it to use chess960
                        }
                        "Hash" => {
                            let Some(value) = value else {
                                debug_print(
                                    &mut output,
                                    "set `Hash` without value; ignoring".to_string(),
                                )?;
                                continue;
                            };
                            let size: usize = match value.parse() {
                                Ok(size) => size,
                                Err(err) => {
                                    debug_print(
                                        &mut output,
                                        "set `Hash` to an invalid value; ignoring".to_string(),
                                    )?;
                                    debug_print(&mut output, format!("error: {err}"))?;
                                    continue;
                                }
                            };
                            if size <= 4096 {
                                engine.set_hash_size(size * 1024 * 1024);
                            } else {
                                debug_print(
                                    &mut output,
                                    "set `Hash` to an invalid value; ignoring".to_string(),
                                )?;
                            }
                        }
                        "Clear Hash" => {
                            if value.is_none() {
                                engine.clear_hash();
                            } else if debug {
                                debug_print(
                                    &mut output,
                                    "set `Clear Hash` to invalid value; ignoring".to_string(),
                                )?;
                            }
                        }
                        name => {
                            if debug {
                                debug_print(
                                    &mut output,
                                    format!("unknown option `{name}`; ignoring"),
                                )?;
                            }
                        }
                    }
                }
            }
            Input::Register(_) => {
                if uci && debug {
                    debug_print(&mut output, "`register` is ignored".to_string())?;
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
                                    debug_print(&mut output, "error setting up board".to_string())?;
                                    debug_print(&mut output, format!("error: {err}"))?;
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
                        debug_print(&mut output, "no moves found".to_string())?;
                    }
                }
            }
            Input::Go(go) => {
                if uci {
                    new_game = false;
                    let mut new_output = stdout();
                    let callback = move |movement| {
                        writeln!(
                            new_output,
                            "{}",
                            Output::BestMove {
                                movement,
                                ponder: None
                            }
                        )
                        .unwrap();
                    };
                    engine.calculate(go.estimate_move_time(&board), go.depth, callback);
                    if debug {
                        if go.ponder {
                            debug_print(
                                &mut output,
                                "`go ponder` is unsupported; ignoring".to_string(),
                            )?;
                        }
                        if go.search_moves.is_some() {
                            debug_print(
                                &mut output,
                                "`go searchmoves` is unsupported; ignoring".to_string(),
                            )?;
                        }
                        if go.mate.is_some() {
                            debug_print(
                                &mut output,
                                "`go mate` is unsupported; ignoring".to_string(),
                            )?;
                        }
                    }
                }
            }
            Input::Stop => {
                if uci {
                    engine.stop();
                }
            }
            Input::PonderHit => {
                if uci && debug {
                    debug_print(
                        &mut output,
                        "`ponderhit` is unsupported; ignoring".to_string(),
                    )?;
                }
            }
            Input::Quit => {
                if uci {
                    return Ok(());
                }
            }
            Input::Repl => {
                if !uci {
                    drop(lines);
                    repl()?;
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
