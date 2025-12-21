use std::{
    cell::LazyCell,
    io::{BufRead, stdin},
    num::NonZero,
};

use crate::{
    board::{Board, NullableLan},
    color::Color,
    engine::Engine,
    game_tree::Table,
    misc::MEBIBYTES,
    repl::repl,
    uci::{
        input::Input,
        output::{Boundary, IdField, Info, OptionType, OptionValue, Output, Score},
    },
};

mod input;
mod output;

const CHESS960: &str = "UCI_Chess960";
const ENGINE_ABOUT: &str = "UCI_EngineAbout";

const CONFIG: [Output; 7] = [
    Output::Id {
        field: IdField::Name,
        value: concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION")),
    },
    Output::Id {
        field: IdField::Author,
        value: env!("CARGO_PKG_AUTHORS"),
    },
    Output::Option {
        name: "Hash",
        kind: OptionType::Spin,
        default: Some(OptionValue::Int(0)),
        boundary: Some(Boundary::Boundary {
            min: 0,
            max: <i32>::MAX,
        }),
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
    Output::Option {
        name: "UCI_EngineAbout",
        kind: OptionType::String,
        default: Some(OptionValue::Str(env!("CARGO_PKG_REPOSITORY"))),
        boundary: None,
    },
    Output::UciOk,
];
pub fn uci_loop() {
    let input = stdin().lock();
    let mut lines = input.lines();

    let mut uci = false;
    let mut debug = false;
    let mut engine = LazyCell::new(Engine::new);
    let mut board = Board::starting_position();
    let mut new_game = true;
    let mut uci_new_game_available = false;
    loop {
        let text = lines.next().unwrap().unwrap();
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
                            "error parsing input but no error information found".to_string(),
                        );
                    } else {
                        for err in err {
                            debug_print(format!("error: {err}"));
                        }
                    }
                }
                continue;
            }
        };
        match parsed_input {
            Input::Uci => {
                for config in CONFIG {
                    println!("{config}");
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
                    println!("{}", Output::ReadyOk);
                }
            }
            Input::SetOption { name, value } => {
                if uci {
                    match name {
                        CHESS960 => {
                            if debug && !matches!(value, Some("true" | "false")) {
                                debug_print(format!("set {CHESS960} to invalid value; ignoring"));
                            }
                            // The engine can already work on chess960 without telling it to use chess960
                        }
                        "Hash" => {
                            let Some(value) = value else {
                                if debug {
                                    debug_print("set `Hash` without value; ignoring".to_string());
                                }
                                continue;
                            };
                            let size: usize = match value.parse() {
                                Ok(size) => size,
                                Err(err) => {
                                    if debug {
                                        debug_print(
                                            "set `Hash` to an invalid value; ignoring".to_string(),
                                        );
                                        debug_print(format!("error: {err}"));
                                    }
                                    continue;
                                }
                            };
                            engine.set_hash_max_capacity(
                                (size / Table::ELEMENT_SIZE).saturating_mul(MEBIBYTES),
                            );
                        }
                        "Clear Hash" => {
                            if value.is_none() {
                                engine.clear_hash();
                            } else if debug {
                                debug_print(
                                    "set `Clear Hash` to invalid value; ignoring".to_string(),
                                );
                            }
                        }
                        ENGINE_ABOUT => {
                            if debug {
                                debug_print(format!(
                                    "setting the option `{ENGINE_ABOUT}` is ignored"
                                ));
                            }
                        }
                        name => {
                            if debug {
                                debug_print(format!("unknown option `{name}`; ignoring"));
                            }
                        }
                    }
                }
            }
            Input::Register(_) => {
                if uci && debug {
                    debug_print("`register` is ignored".to_string());
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
                        board = match position.board() {
                            Ok(board) => board,
                            Err(err) => {
                                if debug {
                                    debug_print("error setting up board".to_string());
                                    debug_print(format!("error: {err}"));
                                }
                                Board::starting_position()
                            }
                        };
                        for movement in moves {
                            board.move_lan(movement);
                        }
                        engine.set_board(board.clone());
                        new_game = false;
                    } else if let Some(movement) = moves.last() {
                        board.move_lan(*movement);
                        engine.move_piece(*movement);
                    } else if debug {
                        debug_print("no moves found".to_string());
                    }
                }
            }
            Input::Go(go) => {
                if uci {
                    new_game = false;

                    let mate = go.mate.map(|moves| {
                        let moves = moves.get();
                        let plies = match board.current_player() {
                            Color::White => moves * 2,
                            Color::Black => moves * 2 - 1,
                        };
                        NonZero::new(plies).unwrap()
                    });
                    let current_player = board.current_player();
                    engine.calculate(
                        go.estimate_move_time(&board),
                        go.depth,
                        mate,
                        move |info| {
                            // precision doesn't matter
                            #[allow(
                                clippy::cast_possible_truncation,
                                clippy::cast_sign_loss,
                                clippy::cast_precision_loss
                            )]
                            let hash_full = (info.hash_capacity as f32
                                / info.hash_max_capacity as f32
                                * 1_000_000_f32) as u32;
                            #[allow(
                                clippy::cast_possible_truncation,
                                clippy::cast_sign_loss,
                                clippy::cast_precision_loss
                            )]
                            let nps = (info.nodes.get() as f32 / info.time.as_secs_f32()) as u32;
                            println!(
                                "{}",
                                Output::Info(vec![
                                    Info::Depth(info.depth),
                                    Info::Time(info.time),
                                    Info::Nodes(info.nodes),
                                    Info::Pv(info.pv),
                                    Info::Score(Score::from_centipawn(
                                        // TODO: don't unwrap
                                        info.score.unwrap().centipawn(),
                                        current_player,
                                    )),
                                    Info::HashFull(hash_full),
                                    Info::Nps(nps),
                                ])
                            );
                        },
                        |movement| {
                            println!(
                                "{}",
                                Output::BestMove {
                                    movement: NullableLan(movement),
                                    ponder: None
                                }
                            );
                        },
                    );
                    if debug {
                        if go.ponder {
                            debug_print("`go ponder` is unsupported; ignoring".to_string());
                        }
                        if go.search_moves.is_some() {
                            debug_print("`go searchmoves` is unsupported; ignoring".to_string());
                        }
                        if go.nodes.is_some() {
                            debug_print("`go nodes` is unsupported; ignoring".to_string());
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
                    debug_print("`ponderhit` is unsupported; ignoring".to_string());
                }
            }
            Input::Quit => {
                if uci {
                    return;
                }
            }
            Input::Repl => {
                if !uci {
                    drop(lines);
                    drop(engine);
                    repl();
                    return;
                }
            }
        }
    }
}
fn debug_print(message: String) {
    println!("{}", Output::Info(vec![Info::String(message)]));
}
