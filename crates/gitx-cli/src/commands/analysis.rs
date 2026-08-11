use crate::cli::{Cli, DependenciesAction};
use crate::commands::config::load_config;
use crate::commands::{format_ts, open_repo, print_json};
use gitx_analysis::{FileAnalysis, HotspotWeights, analyze_repository_with};
use serde_json::json;
use std::collections::HashMap;

/// Analysis weights from the effective configuration (docs/16 §3), falling
/// back to the documented defaults when no config file exists.
fn weights(cli: &Cli) -> HotspotWeights {
    let analysis = match load_config(cli) {
        Ok(c) => c.analysis,
        Err(_) => return HotspotWeights::default(),
    };
    HotspotWeights {
        change_frequency: analysis.hotspot_change_frequency_weight,
        recent_churn: analysis.hotspot_recent_churn_weight,
        bug_fix: analysis.hotspot_bug_fix_weight,
        ownership: analysis.hotspot_ownership_weight,
        complexity: analysis.hotspot_complexity_weight,
    }
}

pub fn contributors(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let head = repo.head_commit_id()?;

    let mut stats: HashMap<String, (u64, i64, i64)> = HashMap::new(); // key -> (commits, first, last)
    for id_res in repo.rev_walk(head)? {
        let commit = repo.find_commit(id_res?)?;
        let key = format!("{} <{}>", commit.author.name, commit.author.email);
        let entry = stats.entry(key).or_insert((0, i64::MAX, i64::MIN));
        entry.0 += 1;
        entry.1 = entry.1.min(commit.author.time);
        entry.2 = entry.2.max(commit.author.time);
    }

    let mut list: Vec<(String, u64, i64, i64)> = stats
        .into_iter()
        .map(|(k, (commits, first, last))| (k, commits, first, last))
        .collect();
    list.sort_by_key(|(_, commits, _, _)| std::cmp::Reverse(*commits));

    if cli.json {
        return print_json(&json!(
            list.iter()
                .map(|(key, commits, first, last)| json!({
                    "author": key,
                    "commits": commits,
                    "first_activity": first,
                    "last_activity": last,
                }))
                .collect::<Vec<_>>()
        ));
    }

    println!("Contributors");
    for (key, commits, first, last) in list {
        println!(
            "  {:<40} {:>6} commits   {} → {}",
            key,
            commits,
            format_ts(first),
            format_ts(last)
        );
    }
    Ok(())
}

pub fn contributor(cli: &Cli, name: &str) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let head = repo.head_commit_id()?;

    let mut commit_count = 0u64;
    let mut first = i64::MAX;
    let mut last = i64::MIN;
    let mut touched_files: std::collections::HashSet<String> = std::collections::HashSet::new();

    for id_res in repo.rev_walk(head)? {
        let commit = repo.find_commit(id_res?)?;
        let matches = commit.author.name.contains(name) || commit.author.email.contains(name);
        if !matches {
            continue;
        }
        commit_count += 1;
        first = first.min(commit.author.time);
        last = last.max(commit.author.time);

        let parent_tree = match commit.parents.first() {
            Some(parent) => Some(repo.find_commit(*parent)?.tree_id),
            None => None,
        };
        for change in repo.diff_tree_to_tree(parent_tree, commit.tree_id)? {
            touched_files.insert(change.path.display().to_string());
        }
    }

    if commit_count == 0 {
        anyhow::bail!("no commits found for contributor `{name}`");
    }

    if cli.json {
        return print_json(&json!({
            "author": name,
            "commits": commit_count,
            "first_activity": first,
            "last_activity": last,
            "files_touched": touched_files.len(),
        }));
    }

    println!("Contributor: {name}");
    println!("  commits      : {commit_count}");
    println!("  first active : {}", format_ts(first));
    println!("  last active  : {}", format_ts(last));
    println!("  files touched: {}", touched_files.len());
    Ok(())
}

pub fn ownership(cli: &Cli, path: Option<&str>) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let analysis = analyze_repository_with(&repo, weights(cli))?;

    let mut files: Vec<&FileAnalysis> = analysis
        .files
        .iter()
        .filter(|f| path.is_none_or(|p| f.path.starts_with(p)))
        .filter(|f| !f.author_lines.is_empty())
        .collect();
    files.sort_by_key(|f| std::cmp::Reverse((f.ownership_concentration * 1000.0) as i64));

    if cli.json {
        return print_json(&json!(files
            .iter()
            .take(50)
            .map(|f| {
                let mut owners: Vec<(String, u64)> = f.author_lines.clone().into_iter().collect();
                owners.sort_by_key(|(_, lines)| std::cmp::Reverse(*lines));
                json!({
                    "file": f.path.display().to_string(),
                    "ownership_concentration": f.ownership_concentration,
                    "contributors": f.metrics.unique_contributors,
                    "top_author": owners.first().map(|(a, _)| a),
                    "author_lines": owners.into_iter().map(|(a, l)| json!({"author": a, "lines": l})).collect::<Vec<_>>(),
                })
            })
            .collect::<Vec<_>>()));
    }

    println!("Ownership concentration (highest first)");
    for f in files.iter().take(30) {
        let top = f.author_lines.iter().max_by_key(|(_, v)| *v);
        let top_str = top
            .map(|(a, l)| format!("{a} ({l} lines)"))
            .unwrap_or_else(|| "-".into());
        println!(
            "  {:>5.1}%  {:<48} {} contributors  top: {}",
            f.ownership_concentration,
            f.path
                .display()
                .to_string()
                .chars()
                .take(48)
                .collect::<String>(),
            f.metrics.unique_contributors,
            top_str
        );
    }
    Ok(())
}

pub fn hotspots(cli: &Cli, limit: usize, path: Option<&str>) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let analysis = analyze_repository_with(&repo, weights(cli))?;

    let files: Vec<&FileAnalysis> = analysis
        .files
        .iter()
        .filter(|f| path.is_none_or(|p| f.path.starts_with(p)))
        .take(limit)
        .collect();

    if cli.json {
        return print_json(&json!({
            "repository": repo.work_dir().map(|p| p.display().to_string()),
            "hotspots": files.iter().map(|f| file_json(f)).collect::<Vec<_>>(),
        }));
    }

    println!("Hotspots (change/maintenance risk, 0–100)");
    for f in &files {
        println!(
            "  {:>5.1}  {:<8}  {}",
            f.hotspot,
            f.classification,
            f.path.display()
        );
        println!(
            "         changes {} | churn 30d {} | fixes {} | contributors {} | ownership {:.0}% | LOC {}",
            f.metrics.change_frequency,
            f.metrics.lines_added + f.metrics.lines_deleted,
            f.metrics.bug_fix_count,
            f.metrics.unique_contributors,
            f.ownership_concentration,
            f.metrics
                .lines_added
                .saturating_sub(f.metrics.lines_deleted),
        );
    }
    Ok(())
}

pub fn risk(cli: &Cli, path: Option<&str>) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let analysis = analyze_repository_with(&repo, weights(cli))?;

    let files: Vec<&FileAnalysis> = match path {
        Some(p) => analysis
            .files
            .iter()
            .filter(|f| f.path == std::path::Path::new(p) || f.path.starts_with(p))
            .collect(),
        None => analysis.files.iter().take(20).collect(),
    };

    if files.is_empty() {
        anyhow::bail!("no files found for the given path");
    }

    if cli.json {
        return print_json(&json!(
            files.iter().map(|f| file_json(f)).collect::<Vec<_>>()
        ));
    }

    // Docs/10 §3: risk output must show evidence, never a bare number.
    for f in &files {
        println!("⚠ {}  (risk {:.0}/100)", f.path.display(), f.risk);
        println!("   change frequency     {}", f.metrics.change_frequency);
        println!(
            "   churn (30d)          {}",
            f.metrics.lines_added + f.metrics.lines_deleted
        );
        println!("   bug-fix commits      {}", f.metrics.bug_fix_count);
        println!("   contributors         {}", f.metrics.unique_contributors);
        println!("   ownership conc.      {:.0}%", f.ownership_concentration);
    }
    Ok(())
}

pub fn health(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let analysis = analyze_repository_with(&repo, weights(cli))?;
    let h = &analysis.health;

    if cli.json {
        return print_json(&json!({
            "overall": h.overall_score,
            "sub_scores": {
                "code_hotspots": h.code_hotspots_score,
                "ownership_risk": h.ownership_risk_score,
                "branch_hygiene": h.branch_hygiene_score,
                "change_volatility": h.change_volatility_score,
                "architecture_stability": h.architecture_stability_score,
                "recovery_risk": h.recovery_risk_score,
            },
            "evidence": {
                "commits": analysis.total_commits,
                "contributors": analysis.total_contributors,
                "current_files": analysis.current_files,
                "analyzed_files": analysis.files.len(),
                "duration_ms": analysis.analysis_duration_ms,
            }
        }));
    }

    println!("Repository Health  (composite, deterministic — docs/10 §8)");
    println!();
    println!(
        "  Code Hotspots          {:>5.0}/100",
        h.code_hotspots_score
    );
    println!(
        "  Ownership Risk         {:>5.0}/100",
        h.ownership_risk_score
    );
    println!(
        "  Branch Hygiene         {:>5.0}/100",
        h.branch_hygiene_score
    );
    println!(
        "  Change Volatility      {:>5.0}/100",
        h.change_volatility_score
    );
    println!(
        "  Architecture Stability {:>5.0}/100",
        h.architecture_stability_score
    );
    println!(
        "  Recovery Risk          {:>5.0}/100",
        h.recovery_risk_score
    );
    println!();
    println!("  Overall                {:>5.0}/100", h.overall_score);
    println!();
    println!(
        "  Evidence: {} commits, {} contributors, {} files ({} analyzed) in {} ms",
        analysis.total_commits,
        analysis.total_contributors,
        analysis.current_files,
        analysis.files.len(),
        analysis.analysis_duration_ms
    );
    Ok(())
}

/// Structural diff between two commits: added/removed/modified files and the
/// modules they affect (docs/07 §11, docs/10 §10).
pub fn architecture_diff(cli: &Cli, from: &str, to: &str) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let from_id = crate::commands::resolve_ref(&repo, from)?;
    let to_id = crate::commands::resolve_ref(&repo, to)?;

    let from_commit = repo.find_commit(from_id)?;
    let to_commit = repo.find_commit(to_id)?;

    let old = snapshot_from_tree(&repo, from_commit.tree_id, from)?;
    let new = snapshot_from_tree(&repo, to_commit.tree_id, to)?;
    let diff = gitx_graph::compare::compare_snapshots(&old, &new);

    let mut added_dirs: std::collections::HashSet<String> = std::collections::HashSet::new();
    for path in &diff.added {
        if let Some(dir) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            added_dirs.insert(dir.display().to_string());
        }
    }

    if cli.json {
        return print_json(&json!({
            "from": from,
            "to": to,
            "added": diff.added.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            "removed": diff.removed.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            "modified": diff.modified.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            "modules_added": added_dirs.iter().collect::<Vec<_>>(),
        }));
    }

    println!("architecture diff {from} → {to}");
    println!(
        "  {} files added, {} removed, {} modified",
        diff.added.len(),
        diff.removed.len(),
        diff.modified.len()
    );
    for path in diff.added.iter().take(20) {
        println!("    + {}", path.display());
    }
    for path in diff.removed.iter().take(20) {
        println!("    - {}", path.display());
    }
    for path in diff.modified.iter().take(20) {
        println!("    ~ {}", path.display());
    }
    if !added_dirs.is_empty() {
        println!("  modules added:");
        for dir in added_dirs {
            println!("    + {dir}/");
        }
    }
    Ok(())
}

fn snapshot_from_tree(
    repo: &gitx_git::Repository,
    tree_id: gitx_git::models::ObjectId,
    label: &str,
) -> anyhow::Result<gitx_graph::snapshot::DirectorySnapshot> {
    let mut snapshot = gitx_graph::snapshot::DirectorySnapshot::new(
        std::path::PathBuf::from(label),
        Some(label.to_string()),
    );
    for (path, oid) in repo.tree_entries(tree_id)? {
        snapshot.add_file(gitx_graph::snapshot::FileMetadata {
            path,
            size: 0,
            modified: std::time::SystemTime::UNIX_EPOCH,
            hash: oid.to_string(),
        });
    }
    Ok(snapshot)
}

/// Directory/module evolution: current top-level structure plus modules added
/// within the recent window (docs/10 §10 architectural milestones at dir level).
pub fn architecture(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let analysis = analyze_repository_with(&repo, weights(cli))?;

    let mut dirs: HashMap<String, usize> = HashMap::new();
    for f in &analysis.files {
        let dir = f
            .path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| ".".into());
        *dirs.entry(dir).or_insert(0) += 1;
    }
    let mut dir_list: Vec<(String, usize)> = dirs.into_iter().collect();
    dir_list.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    let recent_dirs: Vec<String> = analysis
        .files
        .iter()
        .filter(|f| {
            f.metrics
                .first_introduced
                .map(|t| t.timestamp() >= chrono::Utc::now().timestamp() - 90 * 86_400)
                .unwrap_or(false)
        })
        .filter_map(|f| {
            f.path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .map(|p| p.display().to_string())
        })
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    if cli.json {
        return print_json(&json!({
            "directories": dir_list.into_iter().map(|(dir, files)| json!({"dir": dir, "files": files})).collect::<Vec<_>>(),
            "directories_added_recently": recent_dirs,
        }));
    }

    println!("Architecture (directory/module evolution)");
    println!(
        "  directories: {}  files analyzed: {}",
        dir_list.len(),
        analysis.files.len()
    );
    println!();
    println!("  current modules (by file count):");
    for (dir, files) in dir_list.iter().take(20) {
        println!(
            "    {:<40} {files}",
            dir.chars().take(40).collect::<String>()
        );
    }
    if !recent_dirs.is_empty() {
        println!();
        println!("  modules added in last 90 days:");
        for dir in recent_dirs {
            println!("    + {dir}");
        }
    }
    Ok(())
}

/// Dependency overview from declared manifests in the HEAD tree.
/// Parsing is shared with the TUI via `gitx_analysis::manifest` (docs/10 §11).
pub fn dependencies(cli: &Cli, action: Option<DependenciesAction>) -> anyhow::Result<()> {
    match action.unwrap_or(DependenciesAction::List) {
        DependenciesAction::List => dependencies_list(cli),
        DependenciesAction::History { max } => dependencies_history(cli, max),
        DependenciesAction::Diff { from, to } => dependencies_diff(cli, &from, &to),
    }
}

/// Dependency overview from declared manifests + lockfiles in the HEAD tree.
/// Parsing is shared with the TUI via `gitx_analysis::manifest` (docs/10 §11).
pub fn dependencies_list(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let head = repo.head_commit_id()?;
    let head_commit = repo.find_commit(head)?;
    let mut found = gitx_analysis::manifest::head_dependencies_at(&repo, head_commit.tree_id)?;
    found.extend(gitx_analysis::manifest::lockfile_dependencies_at(
        &repo,
        head_commit.tree_id,
    )?);

    if cli.json {
        return print_json(&json!(found
            .iter()
            .map(|(path, deps)| json!({
                "manifest": path.display().to_string(),
                "dependencies": deps.iter().map(|d| json!({"name": d.name, "version": d.version})).collect::<Vec<_>>(),
            }))
            .collect::<Vec<_>>()));
    }

    if found.is_empty() {
        println!("No supported dependency manifests found in HEAD.");
        return Ok(());
    }
    for (path, deps) in &found {
        println!("{}", path.display());
        for dep in deps {
            match &dep.version {
                Some(v) => println!("    {} {v}", dep.name),
                None => println!("    {}", dep.name),
            }
        }
    }
    Ok(())
}

/// Dependency history (docs/10 §11): walk the mainline, sample manifests at
/// each commit, and report add/remove/version-change events.
pub fn dependencies_history(cli: &Cli, max: usize) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let head = repo.head_commit_id()?;

    // Each manifest path keeps its own ordered event stream.
    let mut streams: HashMap<String, Vec<serde_json::Value>> = HashMap::new();
    let mut prior: HashMap<String, Vec<gitx_analysis::manifest::Dependency>> = HashMap::new();

    // Take the newest `max` commits, then walk oldest → newest so events
    // read chronologically.
    let walk: Vec<gitx_git::models::ObjectId> = repo
        .rev_walk(head)?
        .collect::<gitx_git::Result<Vec<_>>>()?
        .into_iter()
        .take(max)
        .rev()
        .collect();
    for id in walk {
        let commit = repo.find_commit(id)?;
        let deps = gitx_analysis::manifest::head_dependencies_at(&repo, commit.tree_id)?;

        for (path, current) in deps {
            let key = path.display().to_string();
            let before = prior.get(&key).cloned().unwrap_or_default();
            let (added, removed, changed) =
                gitx_analysis::manifest::diff_dependencies(&before, &current);
            for d in added {
                streams.entry(key.clone()).or_default().push(json!({
                    "commit": commit.id.to_string(),
                    "event": "added",
                    "name": d.name,
                    "version": d.version,
                }));
            }
            for d in removed {
                streams.entry(key.clone()).or_default().push(json!({
                    "commit": commit.id.to_string(),
                    "event": "removed",
                    "name": d.name,
                    "version": d.version,
                }));
            }
            for (b, a) in changed {
                streams.entry(key.clone()).or_default().push(json!({
                    "commit": commit.id.to_string(),
                    "event": "changed",
                    "name": a.name,
                    "from": b.version,
                    "to": a.version,
                }));
            }
            prior.insert(key, current);
        }
    }

    if cli.json {
        let mut out = Vec::new();
        for (path, events) in &streams {
            out.push(json!({ "manifest": path, "events": events }));
        }
        return print_json(&json!(out));
    }

    if streams.is_empty() {
        println!("No dependency changes found in the last {max} commits.");
        return Ok(());
    }
    for (path, events) in &streams {
        println!("{path}");
        for ev in events {
            let event = ev["event"].as_str().unwrap_or("");
            let name = ev["name"].as_str().unwrap_or("");
            let commit = ev["commit"].as_str().unwrap_or("");
            let short: String = commit.chars().take(7).collect();
            match event {
                "added" => println!("    + {name} {}", ev["version"].as_str().unwrap_or("")),
                "removed" => println!("    - {name} {}", ev["version"].as_str().unwrap_or("")),
                _ => println!(
                    "    ~ {name} {} → {}",
                    ev["from"].as_str().unwrap_or(""),
                    ev["to"].as_str().unwrap_or("")
                ),
            }
            println!("        {short}");
        }
    }
    Ok(())
}

/// Dependency diff between two refs: which dependencies were added, removed,
/// or changed in the manifests/lockfiles between them (docs/10 §11).
pub fn dependencies_diff(cli: &Cli, from: &str, to: &str) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let from_id = crate::commands::resolve_ref(&repo, from)?;
    let to_id = crate::commands::resolve_ref(&repo, to)?;
    let from_commit = repo.find_commit(from_id)?;
    let to_commit = repo.find_commit(to_id)?;

    // Manifest constraints + lockfile-precise versions (docs/10 §11).
    let mut before = gitx_analysis::manifest::head_dependencies_at(&repo, from_commit.tree_id)?;
    let mut after = gitx_analysis::manifest::head_dependencies_at(&repo, to_commit.tree_id)?;
    before.extend(gitx_analysis::manifest::lockfile_dependencies_at(
        &repo,
        from_commit.tree_id,
    )?);
    after.extend(gitx_analysis::manifest::lockfile_dependencies_at(
        &repo,
        to_commit.tree_id,
    )?);

    let (added, removed, changed) = gitx_analysis::manifest::diff_dependency_sets(&before, &after);

    if cli.json {
        return print_json(&json!({
            "from": from,
            "to": to,
            "added": added.iter().map(|d| json!({"name": d.name, "version": d.version})).collect::<Vec<_>>(),
            "removed": removed.iter().map(|d| json!({"name": d.name, "version": d.version})).collect::<Vec<_>>(),
            "changed": changed.iter().map(|(b, a)| json!({"name": a.name, "from": b.version, "to": a.version})).collect::<Vec<_>>(),
        }));
    }

    println!("dependency diff {from} → {to}");
    for d in &added {
        println!("    + {} {}", d.name, d.version.as_deref().unwrap_or(""));
    }
    for d in &removed {
        println!("    - {} {}", d.name, d.version.as_deref().unwrap_or(""));
    }
    for (b, a) in &changed {
        println!(
            "    ~ {} {} → {}",
            a.name,
            b.version.as_deref().unwrap_or(""),
            a.version.as_deref().unwrap_or("")
        );
    }
    if added.is_empty() && removed.is_empty() && changed.is_empty() {
        println!("    (no dependency changes)");
    }
    Ok(())
}

fn file_json(f: &FileAnalysis) -> serde_json::Value {
    json!({
        "file": f.path.display().to_string(),
        "change_frequency": f.metrics.change_frequency,
        "lines_added": f.metrics.lines_added,
        "lines_deleted": f.metrics.lines_deleted,
        "bug_fixes": f.metrics.bug_fix_count,
        "contributors": f.metrics.unique_contributors,
        "ownership_concentration": f.ownership_concentration,
        "hotspot_score": f.hotspot,
        "classification": f.classification,
        "risk_score": f.risk,
    })
}
