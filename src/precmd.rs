use ansi_term::{
    Colour::{Blue, Cyan, Green, Purple, Red},
    ANSIGenericString, ANSIStrings,
};
use clap::{App, Arg, ArgMatches, SubCommand};
use git2::{self, Repository, StatusOptions};
use std::env;
use tico::tico;

fn repo_status(r: &Repository, detail: bool) -> Option<String> {
    let mut out = vec![];

    if let Some(name) = get_head(r) {
        out.push(Cyan.paint(name));
    }

    if !detail {
        if let Some((ic, wtc, cflt, utrk)) = count_statuses(r) {
            if ic != 0 || wtc != 0 || cflt != 0 || utrk !=0 {
                out.push(Red.bold().paint("*"))
            }
        }
    } else {
        if let Some((ahead, behind)) = get_pos(r) {
            if ahead > 0 {
                out.push(Cyan.paint(format!("↑{}", ahead)));
            }
            if behind > 0 {
                out.push(Cyan.paint(format!("↓{}", behind)));
            }
        }
        if let Some((ic, wtc, cflt, utrk)) = count_statuses(r) {
            if ic == 0 && wtc == 0 && cflt == 0 && utrk == 0 {
                out.push(Green.paint("✔"));
            } else {
                if ic > 0 {
                    out.push(Green.paint(format!("♦{}", ic)));
                }
                if cflt > 0 {
                    out.push(Red.paint(format!("✖{}", cflt)));
                }
                if wtc > 0 {
                    out.push(ANSIGenericString::from(format!("✚{}", wtc)));
                }
                if utrk > 0 {
                    out.push(ANSIGenericString::from("…"));
                }
            }
        }

        if let Some(action) = get_action(r) {
            out.push(Purple.paint(format!(" {}", action)));
        }
    }

    Some(ANSIStrings(&out).to_string())
}

fn get_pos(r: &Repository) -> Option<(usize, usize)> {
    let head = r.head().ok()?;
    if !head.is_branch() {
        return None;
    }

    let head_name = head.shorthand()?;
    let head_branch = r.find_branch(head_name, git2::BranchType::Local).ok()?;
    let upstream = head_branch.upstream().ok()?;
    let head_oid = head.target()?;
    let upstream_oid = upstream.get().target()?;

    r.graph_ahead_behind(head_oid, upstream_oid).ok()
}

fn get_head(r: &Repository) -> Option<String> {
    let head = r.head().ok()?;
    if let Some(short) = head.shorthand() {
        if short != "HEAD" {
            return Some(short.to_string());
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

fn get_action(r: &Repository) -> Option<String> {
    let gitdir = r.path();

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

pub fn display(sub_matches: &ArgMatches<'_>) {
    let path = env::current_dir().unwrap();
    let display_path = Blue.paint(truncate_path(path.to_str().unwrap()));

    let branch = match Repository::discover(path) {
        Ok(r) => repo_status(&r, sub_matches.is_present("detail")),
        Err(_) => None,
    };
    let display_branch = Cyan.paint(branch.unwrap_or_default());

    println!("\n{} {}", display_path, display_branch);
}

fn truncate_path(cwd: &str) -> String {
    let home = dirs::home_dir().unwrap();

    tico(&cwd, home.to_str())
}

pub fn args<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("precmd").arg(
        Arg::with_name("detail")
        .long("detail")
        .help("Print git status")
    )
}
