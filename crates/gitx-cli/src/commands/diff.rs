//! `gitx diff <from> <to> [--path P] [--stat]` — unified diff between two
//! refs, processed file-by-file so only one file's hunks are in memory at a
//! time (docs/13 §8: large diffs are paginated/streamed, never materialized
//! whole). On a TTY the output streams into `less -R`; piped output prints
//! directly.

use crate::Cli;
use crate::commands::{open_repo, paginate};
use std::io::{IsTerminal, Write};

pub fn diff(cli: &Cli, from: &str, to: &str, path: Option<&str>, stat: bool) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let from_id = crate::commands::resolve_ref(&repo, from)?;
    let to_id = crate::commands::resolve_ref(&repo, to)?;
    let from_commit = repo.find_commit(from_id)?;
    let to_commit = repo.find_commit(to_id)?;
    let changes = repo.diff_tree_to_tree(Some(from_commit.tree_id), to_commit.tree_id)?;

    let mut lines: Vec<String> = Vec::new();
    let header = format!("diff {} -> {} ({} files changed)", from, to, changes.len());

    if stat {
        lines.push(header);
        for change in changes
            .iter()
            .filter(|c| path.is_none_or(|p| c.path.starts_with(p)))
        {
            let mark = match change.change_type {
                gitx_git::models::ChangeType::Added => "A",
                gitx_git::models::ChangeType::Deleted => "D",
                gitx_git::models::ChangeType::Renamed => "R",
                gitx_git::models::ChangeType::Copied => "C",
                gitx_git::models::ChangeType::Modified => "M",
                _ => "?",
            };
            lines.push(format!(
                "{mark}  +{}/-{}  {}",
                change.insertions,
                change.deletions,
                change.path.display()
            ));
        }
        return paginate(lines);
    }

    // Full diff: stream per file into the pager (or stdout when piped).
    let mut child = if std::io::stdout().is_terminal() {
        Some(
            std::process::Command::new("less")
                .arg("-R")
                .stdin(std::process::Stdio::piped())
                .spawn()?,
        )
    } else {
        None
    };
    let mut sink: Box<dyn Write> = match &mut child {
        Some(c) => Box::new(c.stdin.take().expect("piped stdin")),
        None => Box::new(std::io::stdout()),
    };
    writeln!(sink, "{header}")?;
    for change in changes
        .iter()
        .filter(|c| path.is_none_or(|p| c.path.starts_with(p)))
    {
        if let Some(patch) = gitx_git::diff::render_file_patch(
            &repo,
            Some(from_commit.tree_id),
            to_commit.tree_id,
            change,
        )? {
            write!(sink, "{patch}")?;
        }
    }
    drop(sink);
    if let Some(mut c) = child {
        c.wait()?;
    }
    Ok(())
}
