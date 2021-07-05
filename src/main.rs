extern crate libc;
use clap::{App, AppSettings, Arg, SubCommand};
use git2::{self, Repository, StatusOptions};
use std::env;

const INSERT_SYMBOL: &str = ">";
const COMMAND_SYMBOL: &str = "<";
const COMMAND_KEYMAP: &str = "vicmd";
const RED: &str = "#fb4934";
const BLUE: &str = "#83a598";
const CYAN: &str = "#8ec07c";
//const GREEN: &str = "#b8bb26";
const PURPLE: &str = "#b3869b";
const ORANGE: &str = "#fe8019";
const YELLOW: &str = "#fabd2f";
const WHITE: &str = "#d5c4a1";
const BRBLACK: &str = "#665c54";

fn repo_status(repo: &Repository) -> Option<String> {
    let mut output = vec![];

    if let Some(name) = get_head(repo) {
        output.push(format!("%F{{{}}} {}%f", CYAN, name));
    }

    if let Some((ahead, behind)) = get_ahead_behind(repo) {
        if ahead > 0 {
            output.push(format!("%F{{{}}} ↑{}%f", YELLOW, ahead));
        }
        if behind > 0 {
            output.push(format!("%F{{{}}} ↓{}%F", ORANGE, behind));
        }
    }

    if let Some((ic, wtc, conflict, untracked)) = count_statuses(repo) {
        if ic == 0 && wtc == 0 && conflict == 0 && untracked == 0 {
            output.push(format!("%F{{{}}} Σ%f", CYAN));
        } else {
            if ic > 0 {
                output.push(format!("%F{{{}}} +{}%f", YELLOW, ic));
            }
            if conflict > 0 {
                output.push(format!("%F{{{}}} !{}%f", RED, conflict));
            }
            if wtc > 0 {
                output.push(format!("%F{{{}}} *{}%f", ORANGE, wtc));
            }
            if untracked > 0 {
                output.push(format!("%F{{{}}} ?%f", PURPLE));
            }
        }
    }

    Some(output.into_iter().collect::<String>())
}

fn get_ahead_behind(repo: &Repository) -> Option<(usize, usize)> {
    let head = repo.head().ok()?;
    if !head.is_branch() {
        return None;
    }

    let head_name = head.shorthand()?;
    let head_branch = repo.find_branch(head_name, git2::BranchType::Local).ok()?;
    let upstream = head_branch.upstream().ok()?;
    let head_oid = head.target()?;
    let upstream_oid = upstream.get().target()?;

    repo.graph_ahead_behind(head_oid, upstream_oid).ok()
}

fn get_head(repo: &Repository) -> Option<String> {
    let head = repo.head().ok()?;
    if let Some(shorthand) = head.shorthand() {
        if shorthand != "HEAD" {
            return Some(shorthand.to_string());
        }
    }

    let object = head.peel(git2::ObjectType::Commit).ok()?;
    let short_id = object.short_id().ok()?;

    Some(format!(
        ":{}",
        short_id.iter().map(|ch| *ch as char).collect::<String>()
    ))
}

fn count_statuses(r: &Repository) -> Option<(usize, usize, usize, usize)> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);

    fn count_files(statuses: &git2::Statuses<'_>, status: git2::Status) -> usize {
        statuses
            .iter()
            .filter(|entry| entry.status().intersects(status))
            .count()
    }

    let statuses = r.statuses(Some(&mut opts)).ok()?;

    Some((
        count_files(
            &statuses,
            git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED
                | git2::Status::INDEX_TYPECHANGE,
        ),
        count_files(
            &statuses,
            git2::Status::WT_MODIFIED
                | git2::Status::WT_DELETED
                | git2::Status::WT_TYPECHANGE
                | git2::Status::WT_RENAMED,
        ),
        count_files(&statuses, git2::Status::CONFLICTED),
        count_files(&statuses, git2::Status::WT_NEW),
    ))
}

fn get_time() -> String {
    chrono::Local::now().time().format("%H:%M").to_string()
}

fn get_hostname() -> String {
    let mut string = [0 as libc::c_char; 255];

    unsafe {
        libc::gethostname(&mut string[0], 255);
    }

    ptr_to_string(&mut string[0])
}

fn ptr_to_string(name: *mut i8) -> String {
    let uname = name as *mut _ as *mut u8;

    let s;
    let string;

    unsafe {
        s = ::std::slice::from_raw_parts(uname, libc::strlen(name));
        string = String::from_utf8_lossy(s).to_string();
    }

    string
}

fn pwd(path: &str) -> &str {
    let home = dirs::home_dir().unwrap();
    match path {
        "/" => "root",
        p if Some(p) == home.to_str() => "home",
        _ => &path[path.rfind('/').unwrap() + 1..],
    }
}

fn main() {
    let matches = App::new("zprs")
        .setting(AppSettings::SubcommandRequired)
        .subcommand(SubCommand::with_name("precmd"))
        .subcommand(
            SubCommand::with_name("prompt")
                .arg(Arg::with_name("keymap").short("k").takes_value(true)),
        )
        .get_matches();

    match matches.subcommand() {
        ("precmd", _) => {
            let path = env::current_dir().unwrap();
            let display_path = format!("%F{{{}}}{}%f", YELLOW, pwd(path.to_str().unwrap()));

            let branch = match Repository::discover(path) {
                Ok(r) => repo_status(&r),
                Err(_) => None,
            };

            let display_time = format!("%F{{{}}}[{}] %f", WHITE, get_time());
            let display_host = format!("%F{{{}}}{}%f%F{{{}}}:%f", BLUE, get_hostname(), BRBLACK);
            let display_branch = format!("%F{{{}}}%f{} ", CYAN, branch.unwrap_or_default());

            print!(
                "{}{}{}{}",
                display_time, display_host, display_path, display_branch
            );
        }
        ("prompt", Some(s)) => {
            let keymap = s.value_of("keymap").unwrap_or("US");

            let symbol = match keymap {
                COMMAND_KEYMAP => COMMAND_SYMBOL,
                _ => INSERT_SYMBOL,
            };

            print!("%F{{{}}}{}%f ", RED, symbol);
        }
        _ => (),
    }
}
