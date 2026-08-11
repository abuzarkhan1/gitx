use crate::cli::Cli;
use crate::commands::{format_ts, open_repo, parse_ts, print_json, short_oid};
use anyhow::Context;
use gitx_git::models::ObjectId;
use gitx_history::timeline::{HistoryService, TimelineOptions};
use serde_json::json;
use std::path::PathBuf;

pub fn timeline(
    cli: &Cli,
    author: Option<String>,
    since: Option<String>,
    until: Option<String>,
    branch: Option<String>,
    path: Option<String>,
    max: Option<usize>,
) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let service = HistoryService::new(&repo);

    let from = match &branch {
        Some(name) => {
            let branches = repo.branches()?;
            let target = branches
                .iter()
                .find(|b| b.name == *name)
                .with_context(|| format!("no such branch `{name}`"))?;
            Some(target.target)
        }
        None => None,
    };

    let commits = service.timeline(TimelineOptions {
        max_count: max,
        from,
        path: path.map(PathBuf::from),
        author,
        since: since.as_deref().map(parse_ts).transpose()?,
        until: until.as_deref().map(parse_ts).transpose()?,
    })?;

    if cli.json {
        return print_json(&json!(
            commits
                .iter()
                .map(|c| json!({
                    "oid": c.id.to_string(),
                    "author": c.author.name,
                    "email": c.author.email,
                    "time": c.author.time,
                    "message": c.message,
                    "parents": c.parents.iter().map(|p| p.to_string()).collect::<Vec<_>>(),                }))
                .collect::<Vec<_>>()
        ));
    }

    let lines: Vec<String> = commits
        .iter()
        .map(|c| {
            format!(
                "{} {}  {}  {}",
                short_oid(&c.id),
                format_ts(c.author.time),
                c.author.name,
                c.message
            )
        })
        .collect();
    crate::commands::paginate(lines)
}

pub fn commit(cli: &Cli, oid: &str) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let id = resolve_oid(&repo, oid)?;
    let commit = repo.find_commit(id)?;

    // Diff against the first parent for changed-file stats.
    let parent_tree = match commit.parents.first() {
        Some(parent) => Some(repo.find_commit(*parent)?.tree_id),
        None => None,
    };
    let changes = repo.diff_tree_to_tree(parent_tree, commit.tree_id)?;
    let insertions: u32 = changes.iter().map(|c| c.insertions).sum();
    let deletions: u32 = changes.iter().map(|c| c.deletions).sum();

    if cli.json {
        return print_json(&json!({
            "oid": commit.id.to_string(),
            "tree": commit.tree_id.to_string(),
            "parents": commit.parents.iter().map(|p| p.to_string()).collect::<Vec<_>>(),
            "author": {"name": commit.author.name, "email": commit.author.email, "time": commit.author.time},
            "committer": {"name": commit.committer.name, "email": commit.committer.email, "time": commit.committer.time},
            "message": commit.message,
            "insertions": insertions,
            "deletions": deletions,
            "files": changes.iter().map(|c| json!({
                "path": c.path.display().to_string(),
                "old_path": c.old_path.as_ref().map(|p| p.display().to_string()),
                "change_type": format!("{:?}", c.change_type),
                "insertions": c.insertions,
                "deletions": c.deletions,
            })).collect::<Vec<_>>(),
        }));
    }

    println!("commit {}", commit.id);
    println!("Author: {} <{}>", commit.author.name, commit.author.email);
    println!("Date:   {}", format_ts(commit.author.time));
    if commit.committer.email != commit.author.email {
        println!(
            "Committer: {} <{}>",
            commit.committer.name, commit.committer.email
        );
    }
    if !commit.parents.is_empty() {
        println!(
            "Parents: {}",
            commit
                .parents
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );
    }
    println!();
    for line in commit.message.lines() {
        println!("    {line}");
    }
    println!();
    println!(
        " {} files changed, {} insertions(+), {} deletions(-)",
        changes.len(),
        insertions,
        deletions
    );
    for change in changes.iter().take(30) {
        println!(
            "  {:?} {:>5} {:>5}  {}",
            change.change_type,
            change.insertions,
            change.deletions,
            change.path.display()
        );
    }
    Ok(())
}

pub fn file_history(
    cli: &Cli,
    path: &str,
    _follow: bool,
    since: Option<String>,
    lines: bool,
) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let service = HistoryService::new(&repo);
    let path_buf = PathBuf::from(path);

    if lines {
        // `history --lines` is paginated like `blame` (docs/07 §7).
        return blame_inner(cli, path_buf, 500);
    }

    let commits = service.timeline(TimelineOptions {
        max_count: None,
        from: None,
        path: Some(path_buf),
        author: None,
        since: since.as_deref().map(parse_ts).transpose()?,
        until: None,
    })?;

    if cli.json {
        return print_json(&json!(
            commits
                .iter()
                .map(|c| json!({
                    "oid": c.id.to_string(),
                    "author": c.author.name,
                    "time": c.author.time,
                    "message": c.message,
                }))
                .collect::<Vec<_>>()
        ));
    }

    println!("History of {path}:");
    for commit in commits {
        println!(
            "{} {}  {}  {}",
            short_oid(&commit.id),
            format_ts(commit.author.time),
            commit.author.name,
            commit.message
        );
    }
    Ok(())
}

pub fn blame(cli: &Cli, path: &str, limit: usize) -> anyhow::Result<()> {
    blame_inner(cli, PathBuf::from(path), limit)
}

/// File lineage: every commit that touched the file, following renames
/// (docs/10 file archaeology).
pub fn lineage(cli: &Cli, path: &str) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let service = HistoryService::new(&repo);
    let result = service
        .get_file_lineage(PathBuf::from(path), None)
        .with_context(|| format!("cannot trace lineage of {}", path))?;

    if cli.json {
        return print_json(&json!(
            result
                .history
                .iter()
                .map(|node| json!({
                    "commit": node.commit_id.to_string(),
                    "path": node.path.display().to_string(),
                    "action": format!("{:?}", node.action),
                }))
                .collect::<Vec<_>>()
        ));
    }

    if result.history.is_empty() {
        println!("No history found for {path}.");
        return Ok(());
    }
    println!("Lineage of {path} (newest first):");
    for node in &result.history {
        let action = match &node.action {
            gitx_history::FileAction::Added => "added   ".to_string(),
            gitx_history::FileAction::Modified => "modified".to_string(),
            gitx_history::FileAction::Deleted => "deleted ".to_string(),
            gitx_history::FileAction::Renamed { from } => {
                format!("renamed from {}", from.display())
            }
        };
        println!(
            "  {}  {}  {}",
            short_oid(&node.commit_id),
            format_ts(repo.find_commit(node.commit_id)?.author.time),
            action
        );
    }
    Ok(())
}

fn blame_inner(cli: &Cli, path: PathBuf, limit: usize) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let service = HistoryService::new(&repo);
    let result = service
        .blame(path.clone(), None)
        .with_context(|| format!("cannot blame {}", path.display()))?;
    // Paginate (docs/07 §7: blame is expensive and must be paginated).
    let lines = result.lines.iter().take(limit).collect::<Vec<_>>();

    if cli.json {
        return print_json(&json!(
            lines
                .iter()
                .map(|l| json!({
                    "line": l.line_no,
                    "commit": l.commit_id.to_string(),
                    "content": l.content,
                }))
                .collect::<Vec<_>>()
        ));
    }

    for line in lines {
        println!(
            "{} {:>5} | {}",
            short_oid(&line.commit_id),
            line.line_no,
            line.content
        );
    }
    Ok(())
}

pub fn branches(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let branches = repo.branches()?;

    // Intelligence per local branch vs the default branch (docs/07 §8,
    // docs/10 §5): ahead/behind, age, stale flag.
    let default_name = branches
        .iter()
        .find(|b| b.name == "main" || b.name == "master")
        .map(|b| b.name.clone())
        .or_else(|| {
            branches
                .iter()
                .find(|b| !b.is_remote)
                .map(|b| b.name.clone())
        });

    let mut rows: Vec<serde_json::Value> = Vec::new();
    for branch in &branches {
        let base = default_name
            .as_deref()
            .and_then(|d| branches.iter().find(|b| b.name == d));
        let intelligence = gitx_analysis::branch::branch_intelligence(&repo, branch, base)
            .ok()
            .flatten();
        let (ahead, behind, age_days, is_stale) = intelligence
            .as_ref()
            .map(|i| (i.ahead, i.behind, i.branch_age_days, i.is_stale))
            .unwrap_or((0, 0, 0, false));
        let activity = repo
            .find_commit(branch.target)
            .map(|c| format_ts(c.author.time))
            .unwrap_or_else(|_| "?".into());
        rows.push(json!({
            "name": branch.name,
            "tip": branch.target.to_string(),
            "is_remote": branch.is_remote,
            "ahead": ahead,
            "behind": behind,
            "age_days": age_days,
            "is_stale": is_stale,
            "last_activity": activity,
        }));

        if !cli.json {
            let mark = if branch.is_remote {
                "[remote]"
            } else {
                "[local] "
            };
            let stale = if is_stale { "  STALE" } else { "" };
            let vs = if branch.is_remote || default_name.as_deref() == Some(branch.name.as_str()) {
                String::new()
            } else {
                format!("  ahead {ahead} behind {behind}")
            };
            println!(
                "{mark} {:<24} {}  {age_days:>4}d old  {}{vs}{stale}",
                branch.name,
                short_oid(&branch.target),
                activity
            );
        }
    }
    if cli.json {
        return print_json(&json!(rows));
    }
    Ok(())
}
pub fn branch(cli: &Cli, name: &str) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let branches = repo.branches()?;
    let branch = branches
        .iter()
        .find(|b| b.name == name)
        .with_context(|| format!("no such branch `{name}`"))?;

    let tip = repo.find_commit(branch.target)?;
    let mut commit_count = 0u64;
    for _ in repo.rev_walk(branch.target)? {
        commit_count += 1;
    }

    // Divergence vs the default branch (docs/07 §8): ahead = commits on this
    // branch not on default; behind = commits on default not on this branch.
    let default_name = branches
        .iter()
        .find(|b| b.name == "main" || b.name == "master")
        .map(|b| b.name.clone())
        .or_else(|| {
            branches
                .iter()
                .find(|b| b.name != name && !b.is_remote)
                .map(|b| b.name.clone())
        });
    let (ahead, behind) = match &default_name {
        Some(default) if *default != name => {
            let default_branch = branches.iter().find(|b| b.name == *default);
            let ours = reachable_set(&repo, branch.target)?;
            let theirs = default_branch
                .map(|b| reachable_set(&repo, b.target))
                .transpose()?
                .unwrap_or_default();
            let ahead = ours.difference(&theirs).count() as u64;
            let behind = theirs.difference(&ours).count() as u64;
            (ahead, behind)
        }
        _ => (0, 0),
    };

    // Shared files + merge-complexity estimate (docs/10 §5; the estimate is
    // labeled as such, never a conflict guarantee).
    let base_branch = default_name
        .as_deref()
        .and_then(|d| branches.iter().find(|b| b.name == d));
    let intelligence = gitx_analysis::branch::branch_intelligence(&repo, branch, base_branch)
        .ok()
        .flatten();

    if cli.json {
        return print_json(&json!({
            "name": branch.name,
            "tip": branch.target.to_string(),
            "is_remote": branch.is_remote,
            "commit_count": commit_count,
            "last_activity": tip.author.time,
            "message": tip.message,
            "ahead": ahead,
            "behind": behind,
            "divergence": ahead + behind,
            "default_branch": default_name,
            "age_days": intelligence.as_ref().map(|i| i.branch_age_days),
            "recent_activity_days": intelligence.as_ref().map(|i| i.recent_activity_days),
            "shared_files": intelligence.as_ref().map(|i| i.shared_files),
            "is_stale": intelligence.as_ref().map(|i| i.is_stale),
            "merge_complexity_estimate": intelligence.as_ref().map(|i| i.merge_complexity),
        }));
    }

    println!("branch {name}");
    println!(
        "  tip        : {} — {}",
        short_oid(&branch.target),
        tip.message
    );
    println!("  commits    : {commit_count}");
    println!("  activity   : {}", format_ts(tip.author.time));
    if let Some(i) = &intelligence {
        println!("  age        : {} days", i.branch_age_days);
    }
    if let Some(default) = &default_name
        && default != name
    {
        println!(
            "  vs {default:<24} ahead {ahead} | behind {behind} | divergence {}",
            ahead + behind
        );
        if let Some(i) = &intelligence {
            println!("  shared files: {} changed on both sides", i.shared_files);
            println!("  merge complexity (estimate): {:.1}", i.merge_complexity);
        }
    }
    if intelligence.as_ref().map(|i| i.is_stale).unwrap_or(false) {
        println!("  note       : stale (no recent activity)");
    }
    if branch.is_remote {
        println!("  origin     : remote-tracking");
    }
    Ok(())
}

/// Set of commit oids reachable from `tip` (docs/24 divergence algorithm).
fn reachable_set(
    repo: &gitx_git::Repository,
    tip: ObjectId,
) -> anyhow::Result<std::collections::HashSet<String>> {
    let mut set = std::collections::HashSet::new();
    for id_res in repo.rev_walk(tip)? {
        set.insert(id_res?.to_string());
    }
    Ok(set)
}

/// Resolve a full or abbreviated object id against the object database.
fn resolve_oid(repo: &gitx_git::Repository, oid: &str) -> anyhow::Result<ObjectId> {
    if let Some(full) = ObjectId::from_hex(oid) {
        if repo.object_kind(full)?.is_some() {
            return Ok(full);
        }
        anyhow::bail!("object `{oid}` does not exist");
    }
    // Abbreviated: match a unique prefix across all objects.
    let mut matches = Vec::new();
    for id_res in repo.all_object_ids()? {
        let id = id_res?;
        if id.to_string().starts_with(oid) {
            matches.push(id);
        }
    }
    match matches.len() {
        0 => anyhow::bail!("object `{oid}` does not exist"),
        1 => Ok(matches[0]),
        _ => anyhow::bail!("object `{oid}` is ambiguous ({} candidates)", matches.len()),
    }
}
