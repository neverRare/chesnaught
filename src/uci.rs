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

const CONFIG: [Output; 3] = [
    Output::Id {
        name: "Chesnaught",
        author: "Koko",
    },
    Output::Option {
        name: "UCI_Chess960",
        kind: OptionType::Check,
        default: OptionValue::Bool(false),
        boundary: None,
    },
    // TODO: use `option name UCI_EngineAbout ...`
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
            Input::SetOption { .. } => {
                // We can ignore options, for Chess960, the engine can
                // automatically adjust without any extra information
            }
            input @ Input::Register(_) => {
                if debug {
                    debug_print!(
                        output,
                        "registration is ignored; command received: `{input}`"
                    )?;
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
