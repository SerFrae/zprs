use clap::{App, Arg, ArgMatches, SubCommand};
use ansi_term::Colour::RGB;

const INSERT_SYMBOL: &str = "❯";
const COMMAND_SYMBOL: &str = "❮";
const COMMAND_KEYMAP: &str = "vicmd";
const NO_ERROR: &str = "0";

pub fn display(sub_matches: &ArgMatches<'_>) {
    let last_return_code = sub_matches.value_of("last_return_code").unwrap_or("0");
    let keymap = sub_matches.value_of("keymap").unwrap_or("US");
    let venv_name = sub_matches.value_of("venv").unwrap_or("");

    let symbol = match keymap {
        COMMAND_KEYMAP => COMMAND_SYMBOL,
        _ => INSERT_SYMBOL,
    };

    let shell_color = match (symbol, last_return_code) {
        (COMMAND_SYMBOL, _) => RGB(33,207,95),
        (_, NO_ERROR) => RGB(33,207,95),
        _ => RGB(255,0,75),
    };

    let venv = match venv_name.len() {
        0 => String::from(""),
        _ => format!("%F{{11}}|{}|%f ", venv_name),
    };
    let display_symbol = shell_color.paint(symbol);
    print!("{}{} ", venv, display_symbol);
}

pub fn args<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("prompt")
        .arg(
            Arg::with_name("last_return_code")
                .short("r")
                .takes_value(true),
        )
        .arg(Arg::with_name("keymap").short("k").takes_value(true))
        .arg(Arg::with_name("venv").long("venv").takes_value(true))
}
