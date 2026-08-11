use crate::cli::{Cli, RecoveryAction, ReleaseAction};
use crate::commands::{format_ts, open_repo, print_json, short_oid};
use anyhow::Context;
use serde_json::json;

pub fn recovery(cli: &Cli, action: Option<RecoveryAction>) -> anyhow::Result<()> {
    match action.unwrap_or(RecoveryAction::Reflog) {
        RecoveryAction::Reflog => reflog(cli),
        RecoveryAction::Unreachable => unreachable(cli),
        RecoveryAction::Show { oid } => show(cli, &oid),
    }
}

fn reflog(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let entries = gitx_analysis::collect_reflog(&repo)?;

    if cli.json {
        return print_json(&json!(
            entries
                .iter()
                .map(|e| json!({
                    "reference": e.reference,
                    "previous_oid": e.previous_oid.to_string(),
                    "new_oid": e.new_oid.to_string(),
                    "actor": e.actor_name,
                    "email": e.actor_email,
                    "timestamp": e.timestamp,
                    "message": e.message,
                }))
                .collect::<Vec<_>>()
        ));
    }

    if entries.is_empty() {
        println!("No reflog entries found (reflogs may be disabled).");
        return Ok(());
    }
    println!("Reflog entries (newest first):");
    for e in entries.iter().take(100) {
        println!(
            "  {:<28} {} → {}  {}  {}",
            e.reference,
            short_oid(&e.previous_oid),
            short_oid(&e.new_oid),
            e.timestamp.map(format_ts).unwrap_or_else(|| "-".into()),
            e.message
        );
    }
    Ok(())
}

pub fn unreachable(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let commits = gitx_analysis::find_unreachable_commits(&repo, None)?;

    if cli.json {
        return print_json(&json!(
            commits
                .iter()
                .map(|c| json!({"oid": c.oid.to_string()}))
                .collect::<Vec<_>>()
        ));
    }

    if commits.is_empty() {
        println!("No unreachable commits found.");
        return Ok(());
    }
    println!(
        "{} unreachable commit(s) — candidates for `git gc`, recoverable via reflog:",
        commits.len()
    );
    for commit in commits.iter().take(100) {
        let summary = repo
            .find_commit(commit.oid)
            .map(|c| c.message)
            .unwrap_or_else(|_| "?".into());
        println!("  {}  {}", short_oid(&commit.oid), summary);
    }
    Ok(())
}

fn show(cli: &Cli, oid: &str) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let id = gitx_git::models::ObjectId::from_hex(oid)
        .with_context(|| format!("invalid object id `{oid}`"))?;
    let commit = repo
        .find_commit(id)
        .with_context(|| format!("`{oid}` is not a commit"))?;

    if cli.json {
        return print_json(&json!({
            "oid": commit.id.to_string(),
            "message": commit.message,
            "author": commit.author.name,
            "time": commit.author.time,
            "parents": commit.parents.iter().map(|p| p.to_string()).collect::<Vec<_>>(),
        }));
    }
    println!("commit {}", commit.id);
    println!("Author: {} <{}>", commit.author.name, commit.author.email);
    println!("Date:   {}", format_ts(commit.author.time));
    println!();
    println!("    {}", commit.message);
    Ok(())
}

pub fn release(cli: &Cli, action: ReleaseAction) -> anyhow::Result<()> {
    match action {
        ReleaseAction::Show { tag } => release_show(cli, &tag),
        ReleaseAction::Diff { from, to } => release_diff(cli, &from, &to),
    }
}

fn release_show(cli: &Cli, tag: &str) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let tags = repo.tags()?;
    let tag_obj = tags
        .iter()
        .find(|t| t.name == tag)
        .with_context(|| format!("no such tag `{tag}`"))?;

    let commit = repo.find_commit(tag_obj.target)?;
    let mut commit_count = 0u64;
    for _ in repo.rev_walk(tag_obj.target)? {
        commit_count += 1;
    }

    if cli.json {
        return print_json(&json!({
            "tag": tag,
            "target": tag_obj.target.to_string(),
            "message": commit.message,
            "released_at": commit.committer.time,
            "commits": commit_count,
        }));
    }

    println!("release {tag}");
    println!(
        "  target     : {} — {}",
        short_oid(&tag_obj.target),
        commit.message
    );
    println!("  tagged at  : {}", format_ts(commit.committer.time));
    println!("  commits    : {commit_count}");
    Ok(())
}

/// Summary of a release window: commits added between `from` and `to`, and the
/// files they touched (docs/07 §17).
fn release_diff(cli: &Cli, from: &str, to: &str) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let from_id = crate::commands::resolve_ref(&repo, from)?;
    let to_id = crate::commands::resolve_ref(&repo, to)?;

    // Commits in `to` but not reachable from `from`.
    let from_set: std::collections::HashSet<String> = repo
        .rev_walk(from_id)?
        .collect::<gitx_git::Result<Vec<_>>>()?
        .into_iter()
        .map(|id| id.to_string())
        .collect();
    let mut new_commits = Vec::new();
    for id_res in repo.rev_walk(to_id)? {
        let id = id_res?;
        if !from_set.contains(&id.to_string()) {
            new_commits.push(repo.find_commit(id)?);
        }
    }

    // Files changed across the window (first-parent diffs, deduplicated).
    let mut files: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut insertions: u64 = 0;
    let mut deletions: u64 = 0;
    for commit in &new_commits {
        let parent_tree = match commit.parents.first() {
            Some(parent) => Some(repo.find_commit(*parent)?.tree_id),
            None => None,
        };
        for change in repo.diff_tree_to_tree(parent_tree, commit.tree_id)? {
            files.insert(change.path.display().to_string());
            insertions += change.insertions as u64;
            deletions += change.deletions as u64;
        }
    }

    if cli.json {
        return print_json(&json!({
            "from": from,
            "to": to,
            "commits": new_commits.iter().map(|c| json!({
                "oid": c.id.to_string(),
                "author": c.author.name,
                "time": c.author.time,
                "message": c.message,
            })).collect::<Vec<_>>(),
            "files_changed": files.iter().collect::<Vec<_>>(),
            "insertions": insertions,
            "deletions": deletions,
        }));
    }

    println!("release diff {from} → {to}");
    println!(
        "  {} commits, {} files changed, +{} −{}",
        new_commits.len(),
        files.len(),
        insertions,
        deletions
    );
    for commit in new_commits.iter().take(30) {
        println!(
            "    {} {}  {}",
            short_oid(&commit.id),
            format_ts(commit.author.time),
            commit.message
        );
    }
    if files.len() > 10 {
        println!("  files ({} total):", files.len());
        for f in files.iter().take(20) {
            println!("    {f}");
        }
    } else if !files.is_empty() {
        println!("  files:");
        for f in &files {
            println!("    {f}");
        }
    }
    Ok(())
}
