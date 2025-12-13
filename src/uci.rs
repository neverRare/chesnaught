use std::io::{self, BufRead, Write, stderr};

use crate::{
    repl::repl,
    uci::{
        input::Input,
        output::{OptionType, OptionValue, Output},
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
macro_rules! debug_print {
    ($expr:expr, $($tt:tt)*) => {
        ::std::writeln!(
            $expr,
            "{}",
            $crate::uci::output::Output::Info(::std::vec![$crate::uci::output::Info::String(
                ::std::format!($($tt)*)
            )])
        )
    };
}
pub fn uci_loop(input: &mut impl BufRead, output: &mut impl Write) -> io::Result<()> {
    let mut uci = false;
    let mut debug = false;
    loop {
        let mut text = String::new();
        input.read_line(&mut text)?;
        let text = text.trim();
        if text == "" {
            continue;
        }
        let parsed_input = match Input::from_str(&text) {
            Ok(input) => input,
            Err(err) => {
                if debug {
                    if err.is_empty() {
                        debug_print!(output, "error parsing input but no error information found")?;
                    } else {
                        for err in err {
                            debug_print!(output, "error: {err}")?;
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
                    writeln!(output, "{}", Output::ReadyOk)?;
                }
            }
            Input::SetOption { name, value } => match name {
                CHESS960 => {
                    if !matches!(value, Some("true" | "false")) {
                        debug_print!(output, "set {CHESS960} to invalid value; ignoring")?;
                    }
                    // The engine can already work on chess960 without telling it to use chess960
                }
                name => debug_print!(output, "unknown command `{name}`; ignoring")?,
            },
            Input::Register(_) => {
                if debug {
                    debug_print!(output, "registration is ignored")?;
                }
            }
            Input::UciNewGame => todo!(),
            Input::Position { position, moves } => todo!(),
            Input::Go {
                search_moves,
                ponder,
                w_time,
                b_time,
                w_inc,
                b_inc,
                moves_to_go,
                depth,
                nodes,
                mate,
                move_time,
                infinite,
            } => todo!(),
            Input::Stop => todo!(),
            Input::PonderHit => todo!(),
            Input::Quit => {
                if uci {
                    return Ok(());
                }
            }
            Input::Repl => {
                if !uci {
                    let mut error = stderr();
                    let lock = (!cfg!(debug_assertions)).then(|| error.lock());
                    repl(input, output, &mut error)?;
                    drop(lock);
                    return Ok(());
                }
            }
        }
    }
}
