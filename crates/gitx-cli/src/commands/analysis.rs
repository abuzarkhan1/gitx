use crate::cli::{Cli, DependenciesAction};
use crate::commands::config::{load_config, load_config_for};
use crate::commands::{format_ts, open_repo, print_json, short_oid};
use gitx_analysis::{FileAnalysis, HotspotWeights, analyze_repository_with};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

/// Analysis weights from the effective configuration (docs/16 §3–§5): global
/// config overlaid by a repository `gitx.toml` when present, falling back to
/// the documented defaults.
fn weights(cli: &Cli) -> HotspotWeights {
    let config = match open_repo(cli)
        .ok()
        .map(|repo| load_config_for(cli, &repo))
        .transpose()
    {
        Ok(Some(c)) => c,
        _ => load_config(cli).unwrap_or_default(),
    };
    HotspotWeights {
        change_frequency: config.analysis.hotspot_change_frequency_weight,
        recent_churn: config.analysis.hotspot_recent_churn_weight,
        bug_fix: config.analysis.hotspot_bug_fix_weight,
        ownership: config.analysis.hotspot_ownership_weight,
        complexity: config.analysis.hotspot_complexity_weight,
    }
}

pub fn contributors(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let head = repo.head_commit_id()?;
    let config = crate::commands::config::load_config_for(cli, &repo)?;
    let mappings = &config.identity.mappings;

    // Group by the canonical identity key (docs/05 §3): lowercased email, with
    // the display name resolved through explicit user mappings when configured.
    let mut stats: HashMap<String, (String, u64, i64, i64)> = HashMap::new();
    for id_res in repo.rev_walk(head)? {
        let commit = repo.find_commit(id_res?)?;
        let id = gitx_core::identity::resolve(mappings, &commit.author.name, &commit.author.email);
        let entry = stats
            .entry(id.key)
            .or_insert((id.display_name, 0, i64::MAX, i64::MIN));
        entry.1 += 1;
        entry.2 = entry.2.min(commit.author.time);
        entry.3 = entry.3.max(commit.author.time);
    }

    let mut list: Vec<(String, String, u64, i64, i64)> = stats
        .into_iter()
        .map(|(key, (name, commits, first, last))| (key, name, commits, first, last))
        .collect();
    list.sort_by_key(|(_, _, commits, _, _)| std::cmp::Reverse(*commits));

    if cli.json {
        return print_json(&json!(
            list.iter()
                .map(|(key, name, commits, first, last)| json!({
                    "key": key,
                    "author": name,
                    "commits": commits,
                    "first_activity": first,
                    "last_activity": last,
                }))
                .collect::<Vec<_>>()
        ));
    }
    if cli.csv {
        let headers = [
            "key",
            "author",
            "commits",
            "first_activity",
            "last_activity",
        ];
        let rows: Vec<Vec<String>> = list
            .iter()
            .map(|(key, name, commits, first, last)| {
                vec![
                    key.clone(),
                    name.clone(),
                    commits.to_string(),
                    format_ts(*first),
                    format_ts(*last),
                ]
            })
            .collect();
        return crate::commands::emit_csv(cli, &headers, &rows);
    }

    println!("Contributors");
    for (key, name, commits, first, last) in list {
        let mut label = format!("{name} <{key}>");
        if label.len() > 46 {
            label.truncate(46);
        }
        println!(
            "  {label:<46} {:>6} commits   {} → {}",
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
    if cli.csv {
        let headers = [
            "file",
            "ownership_concentration",
            "contributors",
            "top_author",
            "author_lines",
        ];
        let rows: Vec<Vec<String>> = files
            .iter()
            .take(50)
            .map(|f| {
                let mut owners: Vec<(String, u64)> = f.author_lines.clone().into_iter().collect();
                owners.sort_by_key(|(_, lines)| std::cmp::Reverse(*lines));
                vec![
                    f.path.display().to_string(),
                    format!("{:.1}", f.ownership_concentration),
                    f.metrics.unique_contributors.to_string(),
                    owners.first().map(|(a, _)| a.clone()).unwrap_or_default(),
                    owners
                        .iter()
                        .map(|(a, l)| format!("{a}:{l}"))
                        .collect::<Vec<_>>()
                        .join(" | "),
                ]
            })
            .collect();
        return crate::commands::emit_csv(cli, &headers, &rows);
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

    // Subsystem ownership (docs/10 §4): aggregate ownership per directory.
    let mut author_lines_by_path: std::collections::HashMap<
        std::path::PathBuf,
        std::collections::HashMap<String, u64>,
    > = std::collections::HashMap::new();
    for f in &analysis.files {
        author_lines_by_path.insert(f.path.clone(), f.author_lines.clone());
    }
    let subsystems = gitx_analysis::branch::subsystem_ownership(&author_lines_by_path);
    println!("\nSubsystem ownership (per directory)");
    for (dir, total, top, concentration) in subsystems.iter().take(10) {
        println!(
            "  {concentration:>5.1}%  {:<40} {total:>8} lines  top: {top}",
            dir.chars().take(40).collect::<String>()
        );
    }

    // Knowledge concentration + inactive ownership (docs/10 §4): per-author
    // last activity from a single history walk.
    let mut author_last: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    if let Ok(head) = repo.head_commit_id() {
        for id_res in repo.rev_walk(head)? {
            if let Ok(commit) = repo.find_commit(id_res?) {
                let key = format!("{} <{}>", commit.author.name, commit.author.email);
                let entry = author_last.entry(key).or_insert(commit.author.time);
                *entry = (*entry).max(commit.author.time);
            }
        }
    }
    let cutoff = chrono::Utc::now().timestamp() - 30 * 86_400;

    let knowledge: Vec<&FileAnalysis> = files
        .iter()
        .copied()
        .filter(|f| f.ownership_concentration >= 85.0 && f.metrics.unique_contributors > 1)
        .collect();
    println!("\nKnowledge concentration (bus-factor risk, ≥85% single owner)");
    for f in knowledge.iter().take(10) {
        let top = f
            .author_lines
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(a, _)| a);
        println!(
            "  {:.0}%  {}  owned by {}",
            f.ownership_concentration,
            f.path.display(),
            top.map(String::as_str).unwrap_or("-")
        );
    }

    let inactive: Vec<&FileAnalysis> = files
        .iter()
        .copied()
        .filter(|f| {
            f.author_lines
                .iter()
                .max_by_key(|(_, v)| *v)
                .and_then(|(a, _)| author_last.get(a))
                .map(|&t| t < cutoff)
                .unwrap_or(false)
        })
        .collect();
    println!("\nInactive ownership (top owner idle >30d)");
    for f in inactive.iter().take(10) {
        let top = f
            .author_lines
            .iter()
            .max_by_key(|(_, v)| *v)
            .map(|(a, _)| a);
        println!(
            "  {}  top owner {}",
            f.path.display(),
            top.map(String::as_str).unwrap_or("-")
        );
    }
    Ok(())
}

/// Run the analysis through `AnalysisService` (docs/04 §6): index-backed
/// when fresh, live otherwise. `--no-cache` forces the live path.
fn analyze(cli: &Cli, repo: &gitx_git::Repository) -> anyhow::Result<gitx_analysis::RepoAnalysis> {
    gitx_services::AnalysisService::new(repo).analyze(!cli.no_cache, weights(cli))
}

pub fn hotspots(cli: &Cli, limit: usize, path: Option<&str>) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let analysis = analyze(cli, &repo)?;

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
    if cli.csv {
        let headers = [
            "file",
            "hotspot_score",
            "classification",
            "changes",
            "churn_30d",
            "bug_fixes",
            "contributors",
            "ownership_pct",
        ];
        let rows: Vec<Vec<String>> = files
            .iter()
            .map(|f| {
                vec![
                    f.path.display().to_string(),
                    format!("{:.1}", f.hotspot),
                    f.classification.to_string(),
                    f.metrics.change_frequency.to_string(),
                    (f.metrics.lines_added + f.metrics.lines_deleted).to_string(),
                    f.metrics.bug_fix_count.to_string(),
                    f.metrics.unique_contributors.to_string(),
                    format!("{:.1}", f.ownership_concentration),
                ]
            })
            .collect();
        return crate::commands::emit_csv(cli, &headers, &rows);
    }

    println!("Hotspots (change/maintenance risk, 0–100)");
    for f in &files {
        let cx = if f.fn_count > 0 {
            format!("{} ({} fns)", f.complexity_source, f.fn_count)
        } else {
            f.complexity_source.to_string()
        };
        println!(
            "  {:>5.1}  {:<8}  {}",
            f.hotspot,
            f.classification,
            f.path.display()
        );
        println!(
            "         changes {} | churn 30d {} | fixes {} | contributors {} | ownership {:.0}% | LOC {} | complexity {}",
            f.metrics.change_frequency,
            f.metrics.lines_added + f.metrics.lines_deleted,
            f.metrics.bug_fix_count,
            f.metrics.unique_contributors,
            f.ownership_concentration,
            f.metrics
                .lines_added
                .saturating_sub(f.metrics.lines_deleted),
            cx,
        );
    }
    Ok(())
}

pub fn risk(cli: &Cli, path: Option<&str>) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let analysis = analyze(cli, &repo)?;

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
    if cli.csv {
        let headers = [
            "file",
            "risk_score",
            "change_frequency",
            "churn_30d",
            "bug_fixes",
            "contributors",
            "ownership_pct",
            "complexity_source",
            "function_count",
        ];
        let rows: Vec<Vec<String>> = files
            .iter()
            .map(|f| {
                vec![
                    f.path.display().to_string(),
                    format!("{:.1}", f.risk),
                    f.metrics.change_frequency.to_string(),
                    (f.metrics.lines_added + f.metrics.lines_deleted).to_string(),
                    f.metrics.bug_fix_count.to_string(),
                    f.metrics.unique_contributors.to_string(),
                    format!("{:.1}", f.ownership_concentration),
                    f.complexity_source.to_string(),
                    f.fn_count.to_string(),
                ]
            })
            .collect();
        return crate::commands::emit_csv(cli, &headers, &rows);
    }

    // Docs/10 §3 + §13: risk output must show evidence and its formula and
    // time window, never a bare number.
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
        if f.fn_count > 0 {
            println!(
                "   complexity           {} ({} functions)",
                f.complexity_source, f.fn_count
            );
        } else {
            println!("   complexity           {}", f.complexity_source);
        }
        println!(
            "   formula              risk = (hotspot + ownership + churn30d + complexity) / 4"
        );
        println!("   time window          full history; churn over last 30 days");
    }
    Ok(())
}

pub fn health(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let analysis = analyze(cli, &repo)?;
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
    if cli.csv {
        let headers = ["metric", "value"];
        let rows = vec![
            vec!["overall".into(), format!("{:.1}", h.overall_score)],
            vec![
                "code_hotspots".into(),
                format!("{:.1}", h.code_hotspots_score),
            ],
            vec![
                "ownership_risk".into(),
                format!("{:.1}", h.ownership_risk_score),
            ],
            vec![
                "branch_hygiene".into(),
                format!("{:.1}", h.branch_hygiene_score),
            ],
            vec![
                "change_volatility".into(),
                format!("{:.1}", h.change_volatility_score),
            ],
            vec![
                "architecture_stability".into(),
                format!("{:.1}", h.architecture_stability_score),
            ],
            vec![
                "recovery_risk".into(),
                format!("{:.1}", h.recovery_risk_score),
            ],
            vec![
                "evidence.commits".into(),
                analysis.total_commits.to_string(),
            ],
            vec![
                "evidence.contributors".into(),
                analysis.total_contributors.to_string(),
            ],
            vec![
                "evidence.current_files".into(),
                analysis.current_files.to_string(),
            ],
        ];
        return crate::commands::emit_csv(cli, &headers, &rows);
    }

    println!(
        "Repository Health  (composite, deterministic — docs/10 §8)  band: {}",
        gitx_analysis::health_band(h.overall_score)
    );
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
    // Explainability contract (docs/10 §13): formula, time window, and the
    // classification bands used for every sub-score. Health bands are
    // health-oriented (higher = healthier) — never the risk CRITICAL labels.
    println!("  Formula: overall = Σ(weight_i × sub_score_i), each sub-score normalized 0–100");
    println!("  Time window: full history; churn/activity signals over the last 30 days");
    println!(
        "  Bands: 0–30 POOR · 31–60 FAIR · 61–80 GOOD · 81–100 EXCELLENT (higher = healthier)"
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

    // Dependency-direction changes between the two snapshots (docs/10 §10):
    // module import edges whose direction flipped or newly appeared.
    let before_edges = gitx_analysis::structure::module_import_edges(&repo, from_commit.tree_id)
        .unwrap_or_default();
    let after_edges =
        gitx_analysis::structure::module_import_edges(&repo, to_commit.tree_id).unwrap_or_default();
    let direction_changes =
        gitx_analysis::structure::direction_changes(&before_edges, &after_edges);

    if cli.json {
        return print_json(&json!({
            "from": from,
            "to": to,
            "added": diff.added.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            "removed": diff.removed.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            "modified": diff.modified.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
            "modules_added": added_dirs.iter().collect::<Vec<_>>(),
            "dependency_direction_changes": direction_changes,
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
    if !direction_changes.is_empty() {
        println!();
        println!("  dependency-direction changes (heuristic import edges):");
        for change in &direction_changes {
            println!("    ~ {change}");
        }
    }
    Ok(())
}

/// Detect architectural milestones from the mainline (docs/10 §10).
pub fn architecture_milestones(cli: &Cli, max: usize) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let milestones = gitx_analysis::structure::architecture_milestones(&repo, max)?;

    if cli.json {
        return print_json(&json!(milestones));
    }

    println!("Architectural milestones (newest → oldest, docs/10 §10)");
    if milestones.is_empty() {
        println!("  (no milestones detected in the last {max} commits)");
        return Ok(());
    }
    for m in milestones.iter().rev() {
        println!(
            "  {:<11} {:<20}  {}  {}",
            m.commit,
            format_ts(m.time),
            m.kind,
            m.description
        );
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
        DependenciesAction::Workspace => dependencies_workspace(cli),
        DependenciesAction::Features => dependencies_features(cli),
        DependenciesAction::Usage { max } => dependencies_usage(cli, max),
    }
}

/// Dependency usage + churn (docs/10 §11): for each declared dependency,
/// count the source files referencing it in HEAD, and count how often the
/// dependency changed across the last `max` commits of the mainline.
pub fn dependencies_usage(cli: &Cli, max: usize) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let head = repo.head_commit_id()?;
    let head_commit = repo.find_commit(head)?;

    let mut declared: Vec<gitx_analysis::manifest::Dependency> = Vec::new();
    for (_, deps) in gitx_analysis::manifest::head_dependencies_at(&repo, head_commit.tree_id)? {
        declared.extend(deps);
    }
    let usage = gitx_analysis::manifest::usage_counts(&repo, head_commit.tree_id, &declared)?;

    // Churn: add/remove/version-change events per dependency across the walk.
    let mut churn: HashMap<String, u64> = HashMap::new();
    let mut prior: HashMap<String, Vec<gitx_analysis::manifest::Dependency>> = HashMap::new();
    let walk: Vec<gitx_git::models::ObjectId> = repo
        .rev_walk(head)?
        .collect::<gitx_git::Result<Vec<_>>>()?
        .into_iter()
        .take(max)
        .rev()
        .collect();
    for id in walk {
        let commit = repo.find_commit(id)?;
        for (path, current) in gitx_analysis::manifest::head_dependencies_at(&repo, commit.tree_id)?
        {
            let key = path.display().to_string();
            let before = prior.get(&key).cloned().unwrap_or_default();
            let (added, removed, changed) =
                gitx_analysis::manifest::diff_dependencies(&before, &current);
            for d in added.into_iter().chain(removed) {
                *churn.entry(d.name.clone()).or_insert(0) += 1;
            }
            for (_, a) in changed {
                *churn.entry(a.name.clone()).or_insert(0) += 1;
            }
            prior.insert(key, current);
        }
    }

    // Merge usage and churn per dependency name.
    let mut rows: Vec<(String, u64, u64)> = usage
        .into_iter()
        .map(|(name, files)| {
            let changes = churn.get(&name).copied().unwrap_or(0);
            (name, files, changes)
        })
        .collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.cmp(&a.2)).then(a.0.cmp(&b.0)));

    if cli.json {
        return print_json(&json!(
            rows.iter()
                .map(|(name, files, changes)| json!({
                    "name": name,
                    "files_referencing": files,
                    "changes": changes,
                }))
                .collect::<Vec<_>>()
        ));
    }

    if rows.is_empty() {
        println!("No supported dependency manifests found in HEAD.");
        return Ok(());
    }
    println!("Dependency usage (files referencing) and churn (changes in last {max} commits):");
    for (name, files, changes) in rows {
        println!("  {:<32} {:>4} files   {:>3} changes", name, files, changes);
    }
    Ok(())
}

/// Workspace layout for monorepos (docs/10 §11 workspace-aware resolution).
pub fn dependencies_workspace(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let head = repo.head_commit_id()?;
    let head_commit = repo.find_commit(head)?;
    let ws = gitx_analysis::manifest::detect_workspace(&repo, head_commit.tree_id)?;

    if cli.json {
        return print_json(&json!({
            "kind": format!("{:?}", ws.kind),
            "root": ws.root.as_ref().map(|p| p.display().to_string()),
            "members": ws.members.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
        }));
    }

    if ws.kind == gitx_analysis::manifest::WorkspaceKind::None {
        println!("No workspace detected (single-package repository).");
        return Ok(());
    }
    println!(
        "Workspace: {:?}  root: {}",
        ws.kind,
        ws.root
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "-".into())
    );
    println!("  members ({}):", ws.members.len());
    for member in &ws.members {
        println!("    {}", member.display());
    }
    // Pnpm catalogs (docs/10 §11): declared versions shared by members.
    let catalogs = gitx_analysis::manifest::pnpm_catalogs_in_tree(&repo, head_commit.tree_id)?;
    if !catalogs.is_empty() {
        println!("  pnpm catalogs ({}):", catalogs.len());
        for c in catalogs {
            println!("    {} {}", c.name, c.version);
        }
    }
    Ok(())
}

/// Cargo feature flags + pnpm catalogs declared in HEAD (docs/10 §11).
pub fn dependencies_features(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let head = repo.head_commit_id()?;
    let head_commit = repo.find_commit(head)?;
    let features = gitx_analysis::manifest::cargo_features_in_tree(&repo, head_commit.tree_id)?;
    let catalogs = gitx_analysis::manifest::pnpm_catalogs_in_tree(&repo, head_commit.tree_id)?;

    if cli.json {
        return print_json(&json!({
            "cargo_features": features.iter().map(|(path, fs)| json!({
                "manifest": path.display().to_string(),
                "features": fs,
            })).collect::<Vec<_>>(),
            "pnpm_catalogs": catalogs,
        }));
    }

    if features.is_empty() && catalogs.is_empty() {
        println!("No cargo features or pnpm catalogs declared in HEAD.");
        return Ok(());
    }
    for (path, fs) in &features {
        println!("{}", path.display());
        for f in fs {
            if f.enables.is_empty() {
                println!("    feature {:<20} (empty)", f.name);
            } else {
                println!(
                    "    feature {:<20} enables: {}",
                    f.name,
                    f.enables.join(", ")
                );
            }
        }
    }
    if !catalogs.is_empty() {
        println!("pnpm-workspace.yaml catalogs:");
        for c in &catalogs {
            println!("    {} {}", c.name, c.version);
        }
    }
    Ok(())
}

/// Dependency overview from declared manifests + lockfiles in the HEAD tree.
/// Parsing is shared with the TUI via `gitx_analysis::manifest` (docs/10 §11).
pub fn dependencies_list(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let head = repo.head_commit_id()?;
    let head_commit = repo.find_commit(head)?;
    let declared = gitx_analysis::manifest::head_dependencies_at(&repo, head_commit.tree_id)?;
    let locked = gitx_analysis::manifest::lockfile_dependencies_at(&repo, head_commit.tree_id)?;

    // Merge per-manifest: direct (declared) + indirect (lockfile-only).
    let mut rows: Vec<(PathBuf, Vec<gitx_analysis::manifest::DependencyDetail>)> = Vec::new();
    for (path, deps) in &declared {
        let locked_here: Vec<_> = locked
            .iter()
            .filter(|(p, _)| p == path)
            .flat_map(|(_, d)| d.iter().cloned())
            .collect();
        rows.push((
            path.clone(),
            gitx_analysis::manifest::classify_directness(deps, &locked_here),
        ));
    }
    // Lockfile-only manifests (e.g. a standalone Cargo.lock) still list their
    // entries as indirect.
    for (path, deps) in &locked {
        if !rows.iter().any(|(p, _)| p == path) {
            let details = gitx_analysis::manifest::classify_directness(&[], deps);
            rows.push((path.clone(), details));
        }
    }

    if cli.json {
        return print_json(&json!(
            rows.iter()
                .map(|(path, deps)| json!({
                    "manifest": path.display().to_string(),
                    "dependencies": deps,
                }))
                .collect::<Vec<_>>()
        ));
    }

    if rows.is_empty() {
        println!("No supported dependency manifests found in HEAD.");
        return Ok(());
    }
    for (path, deps) in &rows {
        println!("{}", path.display());
        for dep in deps {
            let marker = if dep.direct { "direct" } else { "indirect" };
            match &dep.version {
                Some(v) => println!("    [{marker:<8}] {} {v}", dep.name),
                None => println!("    [{marker:<8}] {}", dep.name),
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

/// Recurring bug-fix and regression areas (docs/10 §9).
pub fn regressions(cli: &Cli, max: usize) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let report = gitx_analysis::analyze_regressions(&repo, Some(max))?;

    if cli.json {
        return print_json(&json!(report));
    }

    println!(
        "Regression analysis — {} commits, {} fix-classified, {} reverts (heuristic, docs/10 §9)",
        report.total_commits, report.total_fixes, report.total_reverts
    );
    println!();
    println!("Recurring problem areas (highest fix density first):");
    if report.problem_files.is_empty() {
        println!("  (no fix-classified commits found)");
    }
    for f in report.problem_files.iter().take(25) {
        println!(
            "  {:>5.0}%  {:<44} {} fixes / {} changes  {} reverts",
            f.fix_density * 100.0,
            f.path
                .display()
                .to_string()
                .chars()
                .take(44)
                .collect::<String>(),
            f.fix_commits,
            f.total_changes,
            f.reverts
        );
    }
    if !report.reverts.is_empty() {
        println!();
        println!("Reverts (possible regressions):");
        for r in report.reverts.iter().take(20) {
            let gap = r
                .gap_seconds
                .map(|s| format!("{}s after", s))
                .unwrap_or_else(|| "unknown gap".into());
            println!(
                "  {:<9} reverts {}  ({} files, {gap})",
                short_oid_str(&r.revert_oid),
                r.reverted_oid
                    .as_deref()
                    .map(short_oid_str)
                    .unwrap_or_else(|| "?".into()),
                r.paths.len()
            );
        }
    }
    Ok(())
}

fn short_oid_str(oid: &str) -> String {
    oid.chars().take(7).collect()
}

/// Source symbols from the HEAD tree (docs/21 Stage 6, docs/11 §2).
/// Extraction is line-based and heuristic (see `gitx_analysis::symbols`);
/// output is deterministic and read-only.
pub fn symbols(cli: &Cli, path: Option<&str>) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let head = repo.head_commit_id()?;
    let head_commit = repo.find_commit(head)?;
    let mut found = gitx_analysis::symbols::extract_symbols_from_tree(&repo, head_commit.tree_id)?;
    if let Some(p) = path {
        found.retain(|(path, _)| path.starts_with(p));
    }
    let total: usize = found.iter().map(|(_, s)| s.len()).sum();

    if cli.json {
        return print_json(&json!(found
            .iter()
            .map(|(path, syms)| json!({
                "file": path.display().to_string(),
                "symbols": syms.iter().map(|s| json!({"name": s.name, "kind": s.kind, "line": s.line})).collect::<Vec<_>>(),
            }))
            .collect::<Vec<_>>()));
    }

    println!("Symbols in HEAD (heuristic extraction — docs/21 Stage 6)");
    println!("  {total} symbols in {} file(s)", found.len());
    for (path, syms) in &found {
        println!("{}", path.display());
        for s in syms.iter().take(60) {
            println!("    {:>5}  {:<9}  {}", s.line, s.kind, s.name);
        }
        if syms.len() > 60 {
            println!("    ... {} more", syms.len() - 60);
        }
    }
    Ok(())
}

/// Life of a symbol (docs/21 Stage 6): when it was added, moved, or removed
/// along the mainline, computed from the lineage engine (deterministic,
/// read-only). Evidence-first human output (docs/25).
pub fn symbol_history(cli: &Cli, name: &str, path: Option<&str>) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let events =
        gitx_analysis::symbol_history::symbol_history(&repo, name, path.map(std::path::Path::new))?;

    if cli.json {
        return print_json(&json!({
            "symbol": name,
            "events": events.iter().map(|e| json!({
                "commit": e.commit_id.to_string(),
                "file": e.file.display().to_string(),
                "time": e.time,
                "action": match &e.action {
                    gitx_analysis::symbol_history::SymbolAction::Added { line } =>
                        format!("added:{line}"),
                    gitx_analysis::symbol_history::SymbolAction::Moved { from_line, to_line } =>
                        format!("moved:{from_line}->{to_line}"),
                    gitx_analysis::symbol_history::SymbolAction::Removed { line } =>
                        format!("removed:{line}"),
                },
            })).collect::<Vec<_>>(),
        }));
    }

    if events.is_empty() {
        println!("No history for symbol `{name}` in HEAD files.");
        return Ok(());
    }
    println!("Symbol history: `{name}` (mainline, newest first — docs/21 Stage 6)");
    for e in events {
        let when = format_ts(e.time);
        match &e.action {
            gitx_analysis::symbol_history::SymbolAction::Added { line } => println!(
                "  added   {} {}  {line}  ({})",
                short_oid(&e.commit_id),
                e.file.display(),
                when,
            ),
            gitx_analysis::symbol_history::SymbolAction::Moved { from_line, to_line } => println!(
                "  moved   {} {}  :{from_line} -> :{to_line}  ({})",
                short_oid(&e.commit_id),
                e.file.display(),
                when,
            ),
            gitx_analysis::symbol_history::SymbolAction::Removed { line } => println!(
                "  removed {} {}  :{line}  ({})",
                short_oid(&e.commit_id),
                e.file.display(),
                when,
            ),
        }
    }
    Ok(())
}

/// Module/file dependency graph of the HEAD tree (docs/21 Stage 6): builds a
/// `gitx_graph::CodeGraph` (file + directory nodes, Contains + Imports + Calls
/// edges) through the shared builder.
pub fn graph(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let graph = gitx_graph::graph::build_head_code_graph(&repo)?;

    let mut nodes: Vec<serde_json::Value> = graph
        .graph
        .node_indices()
        .map(|i| {
            let d = &graph.graph[i];
            json!({
                "name": d.name,
                "path": d.path.display().to_string(),
                "type": format!("{:?}", d.node_type).to_lowercase(),
            })
        })
        .collect();
    nodes.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    let mut edges: Vec<serde_json::Value> = graph
        .graph
        .edge_indices()
        .map(|e| {
            let (a, b) = graph.graph.edge_endpoints(e).unwrap();
            let d = &graph.graph[e];
            json!({
                "from": graph.graph[a].path.display().to_string(),
                "to": graph.graph[b].path.display().to_string(),
                "type": format!("{:?}", d.edge_type).to_lowercase(),
                "weight": d.weight,
            })
        })
        .collect();
    edges.sort_by(|a, b| {
        (a["from"].as_str(), a["to"].as_str()).cmp(&(b["from"].as_str(), b["to"].as_str()))
    });

    let import_count = edges.iter().filter(|e| e["type"] == "imports").count();
    let call_count = edges.iter().filter(|e| e["type"] == "calls").count();

    if cli.json {
        return print_json(&json!({"nodes": nodes, "edges": edges}));
    }

    println!("Graph (HEAD tree + heuristic imports/calls — docs/21 Stage 6)");
    println!(
        "  {} nodes ({} files, {} directories), {} import edges, {} call edges",
        nodes.len(),
        nodes.iter().filter(|n| n["type"] == "file").count(),
        nodes.iter().filter(|n| n["type"] == "directory").count(),
        import_count,
        call_count,
    );
    println!();
    println!("  directories:");
    for n in nodes.iter().filter(|n| n["type"] == "directory") {
        println!("    {}", n["path"].as_str().unwrap_or(""));
    }
    println!();
    println!("  edges (resolvable, heuristic):");
    if import_count + call_count == 0 {
        println!("    (none — imports are external or unparsed by the heuristic)");
    }
    for e in edges.iter().take(60) {
        println!(
            "    [{}] {} → {}",
            e["type"].as_str().unwrap_or(""),
            e["from"].as_str().unwrap_or(""),
            e["to"].as_str().unwrap_or("")
        );
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
        "complexity_source": f.complexity_source,
        "function_count": f.fn_count,
    })
}
