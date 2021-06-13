use clap::{App, AppSettings};

mod precmd;
mod prompt;

fn main() {
    let matches = App::new("zprs")
        .setting(AppSettings::SubcommandRequired)
        .subcommand(precmd::args())
        .subcommand(prompt::args())
        .get_matches();

    match matches.subcommand() {
        ("precmd", Some(s)) => precmd::display(s),
        ("prompt", Some(s)) => prompt::display(s),
        _ => (),
    }
}
