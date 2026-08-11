use crate::cli::{Cli, RecoveryAction, ReleaseAction};
use crate::commands::{format_ts, open_repo, print_json, short_oid};
use anyhow::Context;
use serde_json::json;

pub fn recovery(cli: &Cli, action: Option<RecoveryAction>) -> anyhow::Result<()> {
    match action.unwrap_or(RecoveryAction::Reflog) {
        RecoveryAction::Reflog => reflog(cli),
        RecoveryAction::Unreachable => unreachable(cli),
        RecoveryAction::Show { oid } => show(cli, &oid),
        RecoveryAction::Export { oid, output } => export(cli, &oid, output),
    }
}

/// Export a commit as a unified patch (docs/12 §6). Read-only: the patch is
/// written to a file, never applied automatically.
fn export(cli: &Cli, oid: &str, output: Option<std::path::PathBuf>) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let id = crate::commands::resolve_ref(&repo, oid)
        .with_context(|| format!("invalid object id `{oid}`"))?;
    let patch = gitx_git::diff::render_commit_patch(&repo, id)
        .with_context(|| format!("cannot render patch for `{oid}`"))?;

    let path = output.unwrap_or_else(|| {
        std::path::PathBuf::from(format!("gitx-recovery-{}.patch", short_oid(&id)))
    });
    std::fs::write(&path, patch).with_context(|| format!("cannot write {}", path.display()))?;

    if cli.json {
        return print_json(&json!({"oid": oid, "exported_to": path.display().to_string()}));
    }
    println!("Exported {} to {}", short_oid(&id), path.display());
    Ok(())
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
    let mut lines = vec!["Reflog entries (newest first):".to_string()];
    for e in entries.iter().take(100) {
        lines.push(format!(
            "  {:<28} {} → {}  {}  {}",
            e.reference,
            short_oid(&e.previous_oid),
            short_oid(&e.new_oid),
            e.timestamp.map(format_ts).unwrap_or_else(|| "-".into()),
            e.message
        ));
    }
    crate::commands::paginate(lines)
}

pub fn unreachable(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let commits = gitx_analysis::find_unreachable_commits(&repo, None)?;

    // Last known reference + reason from the reflog (docs/12 §5).
    let reflog = gitx_analysis::collect_reflog(&repo).unwrap_or_default();
    let last_known = |oid: &str| -> Option<(&str, &str)> {
        reflog
            .iter()
            .rev()
            .find(|e| e.new_oid.to_string() == oid || e.previous_oid.to_string() == oid)
            .map(|e| (e.reference.as_str(), e.message.as_str()))
    };

    let now = chrono::Utc::now().timestamp();
    let rows: Vec<serde_json::Value> = commits
        .iter()
        .map(|c| {
            let (ref_name, msg) = last_known(&c.oid.to_string())
                .map(|(r, m)| (Some(r.to_string()), Some(m.to_string())))
                .unwrap_or((None, None));
            let age = repo
                .find_commit(c.oid)
                .map(|cm| (now - cm.author.time).max(0) / 86_400)
                .unwrap_or(0);
            json!({
                "oid": c.oid.to_string(),
                "age_days": age,
                "last_known_reference": ref_name,
                "reason": if ref_name.is_some() { "reflog entry" } else { "no reflog trace" },
                "reflog_message": msg,
            })
        })
        .collect();

    if cli.json {
        return print_json(&json!(rows));
    }

    if commits.is_empty() {
        println!("No unreachable commits found.");
    } else {
        println!(
            "{} unreachable commit(s) — candidates for `git gc`, recoverable via reflog:",
            commits.len()
        );
    }
    for commit in commits.iter().take(100) {
        let summary = repo
            .find_commit(commit.oid)
            .map(|c| c.message)
            .unwrap_or_else(|_| "?".into());
        let (ref_name, reason) = last_known(&commit.oid.to_string())
            .map(|(r, _)| (format!(" via {r}"), "reflog".to_string()))
            .unwrap_or((String::new(), "no reflog trace".to_string()));
        println!(
            "  {}  {}  ({}d old, {reason}{ref_name})",
            short_oid(&commit.oid),
            summary,
            (now - repo
                .find_commit(commit.oid)
                .map(|c| c.author.time)
                .unwrap_or(now))
            .max(0)
                / 86_400
        );
    }
    println!(
        "\nNote: unreachable objects may be pruned by `git gc` — recoverability is not permanent (docs/12 §7)."
    );

    // Dangling trees/blobs (docs/12 §6): typically `git add`-then-`git reset`
    // accidents and the trees of unreachable commits.
    let dangling = gitx_analysis::find_dangling_objects(&repo, None, None).unwrap_or_default();
    let trees = dangling
        .iter()
        .filter(|d| d.kind == gitx_analysis::DanglingKind::Tree)
        .count();
    let blobs = dangling
        .iter()
        .filter(|d| d.kind == gitx_analysis::DanglingKind::Blob)
        .count();
    if trees + blobs > 0 {
        println!("\nDangling objects: {trees} tree(s), {blobs} blob(s)");
        for d in dangling.iter().take(50) {
            println!("  {:<7} {}", d.kind.to_string(), d.oid);
        }
        println!(
            "  (dangling blobs are often staged-then-discarded content; recover them with `gitx recovery export` after re-attaching via `git cat-file`)"
        );
    } else {
        println!("\nNo dangling trees or blobs.");
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

pub fn release(
    cli: &Cli,
    tag: Option<String>,
    action: Option<ReleaseAction>,
) -> anyhow::Result<()> {
    match (tag, action) {
        // `gitx release <TAG>` (docs/07 §17).
        (Some(tag), None) => release_show(cli, &tag),
        (Some(tag), Some(ReleaseAction::Show { tag: _ })) => release_show(cli, &tag),
        (None, Some(ReleaseAction::Show { tag })) => release_show(cli, &tag),
        (None, Some(ReleaseAction::Diff { from, to })) => release_diff(cli, &from, &to),
        (Some(_), Some(ReleaseAction::Diff { .. })) => {
            anyhow::bail!(
                "provide either `gitx release <TAG>` or `gitx release diff <REF1> <REF2>`, not both"
            )
        }
        (None, None) => {
            anyhow::bail!("release requires a tag (`gitx release <TAG>`) or `diff <REF1> <REF2>`")
        }
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

    // Release depth (docs/10 §12): contributors, classifications, top areas.
    let mut contributors_set: std::collections::BTreeSet<String> =
        std::collections::BTreeSet::new();
    let mut classifications: std::collections::BTreeMap<String, u32> =
        std::collections::BTreeMap::new();
    for commit in &new_commits {
        contributors_set.insert(commit.author.name.clone());
        let class = format!(
            "{:?}",
            gitx_analysis::classify_commit_message(&commit.message)
        );
        *classifications.entry(class).or_insert(0) += 1;
    }
    let mut areas: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    for f in &files {
        let dir = std::path::Path::new(f)
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "(root)".into());
        *areas.entry(dir).or_insert(0) += 1;
    }

    // Top hotspots touched by this release window (docs/10 §12).
    let hotspot_scores: std::collections::HashMap<String, (f64, &str)> =
        gitx_analysis::analyze_repository(&repo)
            .map(|a| {
                a.files
                    .iter()
                    .map(|f| (f.path.display().to_string(), (f.hotspot, f.classification)))
                    .collect()
            })
            .unwrap_or_default();
    let mut touched_hotspots: Vec<(&String, f64, &str)> = files
        .iter()
        .filter_map(|f| {
            hotspot_scores
                .get(f)
                .map(|(score, class)| (f, *score, *class))
        })
        .collect();
    touched_hotspots.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

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
            "contributors": contributors_set.iter().collect::<Vec<_>>(),
            "classifications": classifications.iter().map(|(k, v)| json!({k: v})).collect::<Vec<_>>(),
            "top_areas": areas.iter().take(10).map(|(k, v)| json!({k: v})).collect::<Vec<_>>(),
            "top_hotspots": touched_hotspots.iter().take(10).map(|(f, score, class)| json!({"file": f, "score": score, "classification": class})).collect::<Vec<_>>(),
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
    println!(
        "  contributors: {}",
        contributors_set
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    );
    if !classifications.is_empty() {
        let summary: Vec<String> = classifications
            .iter()
            .map(|(k, v)| format!("{k} {v}"))
            .collect();
        println!("  classifications: {}", summary.join(", "));
    }
    for commit in new_commits.iter().take(30) {
        println!(
            "    {} {}  {}",
            short_oid(&commit.id),
            format_ts(commit.author.time),
            commit.message
        );
    }
    if !touched_hotspots.is_empty() {
        println!("  top hotspots touched (maintenance risk):");
        for (f, score, class) in touched_hotspots.iter().take(5) {
            println!("    {score:>4.0} {class:<4} {f}");
        }
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
