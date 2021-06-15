use clap::{App, AppSettings, Arg, SubCommand};
use git2::{self, Repository, StatusOptions};
use std::env;

const INSERT_SYMBOL: &str = ">";
const COMMAND_SYMBOL: &str = "<";
const COMMAND_KEYMAP: &str = "vicmd";
const NO_ERROR: &str = "0";
const RED: &str = "#ff004b";
const BLUE: &str = "#00c0ff";
const GREEN: &str = "#21cf5f";
const PURPLE: &str = "#d448ff";
const ORANGE: &str = "#ff8c00";
const YELLOW: &str = "#ffca00";

fn repo_status(repo: &Repository) -> Option<String> {
    let mut output = vec![];

    if let Some(name) = get_head(repo) {
        output.push(format!("%B%F{{{}}}{}%f%b", GREEN, name));
    }

    if let Some((ahead, behind)) = get_ahead_behind(repo) {
        if ahead > 0 {
            output.push(format!("%B%F{{{}}} ↑{}%f%b", YELLOW, ahead));
        }
        if behind > 0 {
            output.push(format!("%B%F{{{}}} ↓{}%F%b", ORANGE, behind));
        }
    }
    if let Some((ic, wtc, conflict, untracked)) = count_statuses(repo) {
        if ic == 0 && wtc == 0 && conflict == 0 && untracked == 0 {
            output.push(format!("%B%F{{{}}} Σ%f%b", GREEN));
        } else {
            if ic > 0 {
                output.push(format!("%B%F{{{}}} Π{}%f%b", YELLOW, ic));
            }
            if conflict > 0 {
                output.push(format!("%B%F{{{}}} ‼️{}%f%b", RED, conflict));
            }
            if wtc > 0 {
                output.push(format!("%B%F{{{}}} Δ{}%f%b", ORANGE, wtc));
            }
            if untracked > 0 {
                output.push(format!("%B%F{{{}}} ?%f%b", PURPLE));
            }
        }
    }

    if let Some(action) = get_action(repo) {
        output.push(format!(" {}", action));
    }
    output.push(String::from(" "));
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

fn get_action(repo: &Repository) -> Option<String> {
    let gitdir = repo.path();

    for tmp in &[
        gitdir.join("rebase-apply"),
        gitdir.join("rebase"),
        gitdir.join("..").join(".dotest"),
    ] {
        if tmp.join("rebasing").exists() {
            return Some("rebase".to_string());
        }
        if tmp.join("applying").exists() {
            return Some("am".to_string());
        }
        if tmp.exists() {
            return Some("am/rebase".to_string());
        }
    }

    for tmp in &[
        gitdir.join("rebase-merge").join("interactive"),
        gitdir.join(".dotest-merge").join("interactive"),
    ] {
        if tmp.exists() {
            return Some("rebase-i".to_string());
        }
    }

    for tmp in &[gitdir.join("rebase-merge"), gitdir.join(".dotest-merge")] {
        if tmp.exists() {
            return Some("rebase-m".to_string());
        }
    }

    if gitdir.join("MERGE_HEAD").exists() {
        return Some("merge".to_string());
    }

    if gitdir.join("BISECT_LOG").exists() {
        return Some("bisect".to_string());
    }

    if gitdir.join("CHERRY_PICK_HEAD").exists() {
        if gitdir.join("sequencer").exists() {
            return Some("cherry-seq".to_string());
        } else {
            return Some("cherry".to_string());
        }
    }

    if gitdir.join("sequencer").exists() {
        return Some("cherry-or-revert".to_string());
    }

    None
}

fn truncate_path(path: &str) -> String {
    let home = dirs::home_dir().unwrap();
    let truncated = match home.to_str() {
        Some(dir) => path.replacen(&dir, "~", 1),
        None => path.to_owned(),
    };

    let mut shortened = String::from("");
    let mut skip_char = false;
    let mut count = 0;
    let sections = truncated.chars().filter(|&x| x == '/').count();

    for c in truncated.chars() {
        match c {
            '~' => {
                if !skip_char {
                    shortened.push(c)
                }
            }
            '.' => {
                skip_char = false;
                shortened.push(c);
            }
            '/' => {
                skip_char = false;
                count += 1;
                shortened.push(c)
            }
            _ => {
                if skip_char && count < sections {
                    continue;
                } else {
                    skip_char = true;
                    shortened.push(c);
                }
            }
        }
    }

    shortened
}

fn main() {
    let matches = App::new("zprs")
        .setting(AppSettings::SubcommandRequired)
        .subcommand(SubCommand::with_name("precmd"))
        .subcommand(SubCommand::with_name("prompt")
            .arg(
                Arg::with_name("last_return_code")
                    .short("r")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("keymap")
                    .short("k")
                    .takes_value(true)
            )
        )
        .get_matches();

    match matches.subcommand() {
        ("precmd", _) => {
            let path = env::current_dir().unwrap();
            let display_path = format!("%B%F{{{}}}{}%f%b", BLUE, truncate_path(path.to_str().unwrap()));

            let branch = match Repository::discover(path) {
                Ok(r) => repo_status(&r),
                Err(_) => None,
            };
            let display_branch = format!("%F{{{}}}%f{}", GREEN, branch.unwrap_or_default());
            print!("{} {}", display_path, display_branch);
        },
        ("prompt", Some(s)) => {
            let last_return_code = s.value_of("last_return_code").unwrap_or("0");
            let keymap = s.value_of("keymap").unwrap_or("US");

            let symbol = match keymap {
                COMMAND_KEYMAP => COMMAND_SYMBOL,
                _ => INSERT_SYMBOL,
            };

            let shell_colour = match (symbol, last_return_code) {
                (COMMAND_SYMBOL, _) => GREEN,
                (_, NO_ERROR) => GREEN,
                _ => RED,
            };

            print!("%B%F{{{}}}{}%f%b ", shell_colour, symbol);
        },
        _ => (),
    }
}
