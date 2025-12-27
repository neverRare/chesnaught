use std::{
    cell::LazyCell,
    fmt::Write as _,
    io::{BufRead, Write, stdin, stdout},
    num::NonZero,
};

use crate::{
    board::{Board, Lan, NullableLan},
    color::Color,
    engine::{self, Engine},
    game_tree::Table,
    misc::MEBIBYTES,
    uci::{
        input::{Go, Input},
        output::{Boundary, IdField, Info, OptionType, OptionValue, Output, Score, SearchInfo},
    },
};

mod input;
mod output;

const CHESS960: &str = "UCI_Chess960";
const ENGINE_ABOUT: &str = "UCI_EngineAbout";

const CONFIG: [Output; 9] = [
    Output::Id {
        field: IdField::Name,
        value: concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION")),
    },
    Output::Id {
        field: IdField::Author,
        value: env!("CARGO_PKG_AUTHORS"),
    },
    Output::Option {
        name: "Thread",
        kind: OptionType::Spin,
        default: Some(OptionValue::Int(1)),
        boundary: Some(Boundary::Boundary {
            min: 1,
            max: <i32>::MAX,
        }),
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
        name: "Ponder",
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
    let mut output = stdout().lock();
    for config in CONFIG {
        writeln!(output, "{config}").unwrap();
    }
    drop(output);
    let input = stdin().lock();
    let mut lines = input.lines();

    let mut debug = false;
    let mut engine = LazyCell::new(Engine::new);
    let mut hash_max_capacity = 0;
    let mut board = Board::starting_position();
    let mut move_count = 0;
    let mut new_game = true;
    let mut uci_new_game_available = false;

    let mut ponder = false;

    let mut last_go = None;
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
        if debug {
            let input: Box<[_]> = text
                .split(<char>::is_whitespace)
                .filter(|token| !token.is_empty())
                .collect();
            let recognized = parsed_input.to_string();
            let recognized_tokens: Box<[_]> = recognized
                .split(<char>::is_whitespace)
                .filter(|token| !token.is_empty())
                .collect();
            if input != recognized_tokens {
                debug_print("warning: there are parts of input that aren't recognized".to_string());
                debug_print(format!("recognized input: {recognized}"));
            }
        }
        match parsed_input {
            Input::Debug(new_value) => debug = new_value,

            Input::IsReady => {
                engine.ready();
                println!("{}", Output::ReadyOk);
            }
            Input::SetOption { name, value } => {
                match name {
                    CHESS960 => {
                        if debug && !matches!(value, Some("true" | "false")) {
                            debug_print(format!("set {CHESS960} to invalid value; ignoring"));
                        }
                        // The engine can already work on chess960 without telling it to use chess960
                    }
                    "Thread" => {
                        let Some(value) = value else {
                            if debug {
                                debug_print("set `Thread` without value; ignoring".to_string());
                            }
                            continue;
                        };
                        let thread: NonZero<usize> = match value.parse() {
                            Ok(size) => size,
                            Err(err) => {
                                if debug {
                                    debug_print(
                                        "set `Thread` to an invalid value; ignoring".to_string(),
                                    );
                                    debug_print(format!("error: {err}"));
                                }
                                continue;
                            }
                        };
                        engine.set_thread(thread);
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
                        hash_max_capacity = (size / Table::ELEMENT_SIZE).saturating_mul(MEBIBYTES);
                        engine.set_hash_max_capacity(hash_max_capacity);
                    }
                    "Clear Hash" => {
                        if value.is_none() {
                            engine.clear_hash();
                        } else if debug {
                            debug_print("set `Clear Hash` to invalid value; ignoring".to_string());
                        }
                    }
                    "Ponder" => {
                        if let Some(value) = value {
                            let value = match value.parse() {
                                Ok(value) => value,
                                Err(err) => {
                                    if debug {
                                        debug_print(
                                            "set `Ponder` to an invalid value; ignoring"
                                                .to_string(),
                                        );
                                        debug_print(format!("error: {err}"));
                                    }
                                    continue;
                                }
                            };
                            ponder = value;
                        } else if debug {
                            debug_print("set `Ponder` without value; ignoring".to_string());
                        }
                    }
                    ENGINE_ABOUT => {
                        if debug {
                            debug_print(format!("setting the option `{ENGINE_ABOUT}` is ignored"));
                        }
                    }
                    name => {
                        if debug {
                            debug_print(format!("unknown option `{name}`; ignoring"));
                        }
                    }
                }
            }
            Input::Register(_) => {
                if debug {
                    debug_print("`register` is ignored".to_string());
                }
            }
            Input::UciNewGame => {
                new_game = true;
                uci_new_game_available = true;
                engine.set_board(Board::starting_position());
                board = Board::starting_position();
            }
            Input::Position { position, moves } => {
                if !uci_new_game_available || new_game {
                    if debug {
                        debug_print("setting up new board".to_string());
                    }
                    board = position.board().unwrap();
                    for movement in &moves {
                        board.move_lan(*movement);
                    }
                    engine.set_board(board.clone());
                    new_game = false;
                } else {
                    let moves = &moves[move_count..];
                    if debug {
                        let mut message = "reusing previous board. moves used:".to_string();
                        for movement in moves {
                            write!(&mut message, " {movement}").unwrap();
                        }
                        debug_print(message);
                    }
                    for movement in moves {
                        board.move_lan(*movement);
                        engine.move_piece(*movement);
                    }
                }
                move_count = moves.len();
            }
            Input::Go(go) => {
                new_game = false;

                last_go = Some(Go {
                    search_moves: None,
                    ponder: false,
                    depth: None,
                    nodes: None,
                    mate: None,
                    move_time: None,
                    ..go
                });
                let mate = go.mate.map(|moves| {
                    let moves = moves.get();
                    let plies = match board.current_player() {
                        Color::White => moves * 2,
                        Color::Black => moves * 2 - 1,
                    };
                    NonZero::new(plies).unwrap()
                });
                engine.calculate(
                    go.estimate_move_time(&board),
                    go.depth,
                    go.nodes,
                    mate,
                    info_callback(hash_max_capacity, board.current_player()),
                    best_move_callback(ponder, go.ponder),
                );
                if debug {
                    if go.search_moves.is_some() {
                        debug_print("`go searchmoves` is unsupported; ignoring".to_string());
                    }
                    if go.nodes.is_some() {
                        debug_print("`go nodes` is unsupported; ignoring".to_string());
                    }
                }
            }
            Input::Stop => engine.stop(),
            Input::PonderHit => {
                engine.stop();
                engine.move_piece(engine.ponder().unwrap());
                engine.calculate(
                    last_go.clone().unwrap().estimate_move_time(&board),
                    None,
                    None,
                    None,
                    info_callback(hash_max_capacity, board.current_player()),
                    best_move_callback(ponder, false),
                );
            }
            Input::Quit => return,
        }
    }
}
fn debug_print(message: String) {
    println!("{}", Output::Info(Info::Text(message.into_boxed_str())));
}
fn info_callback(hash_max_capacity: usize, current_player: Color) -> impl Fn(engine::Info) + Send {
    move |info| {
        // precision doesn't matter
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let hash_full = if info.hash_capacity >= hash_max_capacity {
            1_000
        } else {
            (info.hash_capacity as f32 / hash_max_capacity as f32 * 1_000_f32) as u32
        };
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let nps = (info.nodes.get() as f32 / info.time.as_secs_f32()) as u32;
        println!(
            "{}",
            Output::Info(Info::Search(SearchInfo {
                depth: info.depth,
                time: info.time,
                nodes: info.nodes,
                pv: info.pv,
                score: info
                    .score
                    .map(|score| Score::from_centipawn(score.centipawn(), current_player,)),
                hash_full,
                nps
            }))
        );
    }
}
fn best_move_callback(
    ponder_enabled: bool,
    ponder_mode: bool,
) -> impl Fn(Option<Lan>, Option<Lan>) + Send {
    move |movement, ponder_movement| {
        if !ponder_mode {
            let ponder_movement = if ponder_enabled {
                ponder_movement
            } else {
                None
            };
            println!(
                "{}",
                Output::BestMove {
                    movement: NullableLan(movement),
                    ponder: ponder_movement
                }
            );
        }
    }
}
