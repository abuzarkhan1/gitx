use crate::cli::Cli;
use crate::commands::{format_ts, open_repo, print_json, short_oid};
use gitx_services::RepositoryService;
use serde_json::json;

pub fn info(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let service = RepositoryService::new(&repo);
    let data = service.info();

    if cli.json {
        return print_json(&data);
    }

    println!("GitX repository");
    println!(
        "  work dir : {}",
        data["work_dir"].as_str().unwrap_or("<bare>")
    );
    println!("  git dir  : {}", data["git_dir"].as_str().unwrap_or("?"));
    if let Some(head) = data["head"].as_str() {
        println!(
            "  head     : {} {}",
            short_oid(&gitx_git::models::ObjectId::from_hex(head).expect("head oid")),
            data["head_message"].as_str().unwrap_or("")
        );
    } else {
        println!("  head     : (no commits yet)");
    }
    println!("  state    : {}", data["state"].as_str().unwrap_or("clean"));
    let index_state: gitx_services::IndexState =
        serde_json::from_value(data["index_state"].clone())
            .unwrap_or(gitx_services::IndexState::Unsupported);
    println!("  index    : {}", index_state_label(index_state));
    println!("  branches : {}", data["branches"].as_u64().unwrap_or(0));
    println!("  tags     : {}", data["tags"].as_u64().unwrap_or(0));
    Ok(())
}

fn index_state_label(state: gitx_services::IndexState) -> String {
    use gitx_services::IndexState;
    match state {
        IndexState::Indexed => "Indexed (fresh)".into(),
        IndexState::PartiallyIndexed => "PartiallyIndexed (stale — run `gitx refresh`)".into(),
        IndexState::Failed => "Failed (corrupt — run `gitx index rebuild`)".into(),
        IndexState::Unsupported => "Unsupported (no index — analysis computed live)".into(),
    }
}

pub fn status(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let service = RepositoryService::new(&repo);
    let state = service.state();
    let head = repo.head_commit_id().ok();

    if cli.json {
        return print_json(&json!({
            "state": state.git,
            "index_state": state.index,
            "head": head.map(|id| id.to_string()),
        }));
    }

    match state.git.to_lowercase().as_str() {
        "clean" | "" => println!("clean"),
        other => println!("{other} (merge/rebase in progress)"),
    }
    println!("  index    : {}", index_state_label(state.index));
    match head {
        Some(id) => {
            let commit = repo.find_commit(id)?;
            println!("  HEAD     : {} — {}", short_oid(&id), commit.message);
        }
        None => println!("  HEAD     : unborn (no commits)"),
    }
    Ok(())
}

pub fn stats(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let service = RepositoryService::new(&repo);
    // Index-backed fast path (docs/13 §3): with a fresh index the statistics
    // come from SQLite in milliseconds instead of recomputing from Git.
    let from_index = service.stats_from_index().ok().flatten();
    let (stats, source) = match from_index {
        Some(s) => (
            gitx_analysis::RepoStats {
                commits: s.commits,
                contributors: s.contributors as usize,
                files: s.files as usize,
                branches: s.branches as usize,
                tags: s.tags as usize,
                age_days: s.age_days as i64,
                first_commit: s.first_commit,
                last_commit: s.latest_commit,
                head_oid: repo.head_commit_id().ok().map(|id| id.to_string()),
                head_message: repo
                    .head_commit_id()
                    .ok()
                    .and_then(|id| repo.find_commit(id).ok())
                    .map(|c| c.message),
                languages: s
                    .languages
                    .into_iter()
                    .map(|(ext, count)| (ext, count as usize))
                    .collect(),
            },
            "index",
        ),
        None => (gitx_analysis::repository_stats(&repo)?, "live"),
    };

    if cli.json {
        return print_json(&json!({
            "commits": stats.commits,
            "contributors": stats.contributors,
            "files": stats.files,
            "branches": stats.branches,
            "tags": stats.tags,
            "age_days": stats.age_days,
            "first_commit": stats.first_commit,
            "head": stats.head_oid,
            "languages": stats.languages.iter().map(|(ext, count)| json!({"extension": ext, "files": count})).collect::<Vec<_>>(),
        }));
    }

    println!("Repository statistics (source: {source})");
    println!("  commits      : {}", stats.commits);
    println!("  contributors : {}", stats.contributors);
    println!("  files        : {}", stats.files);
    println!("  branches     : {}", stats.branches);
    println!("  tags         : {}", stats.tags);
    println!(
        "  age          : {} days{}",
        stats.age_days,
        stats
            .first_commit
            .map(|t| format!(" (first commit {})", format_ts(t)))
            .unwrap_or_default()
    );
    println!("  languages    :");
    for (ext, count) in stats.languages.iter().take(10) {
        println!("    {ext:<12} {count}");
    }
    Ok(())
}
