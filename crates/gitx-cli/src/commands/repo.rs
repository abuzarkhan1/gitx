use crate::cli::Cli;
use crate::commands::{format_ts, open_repo, print_json, short_oid};
use serde_json::json;

pub fn info(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let head = repo.head_commit_id().ok();
    let head_commit = head.and_then(|id| repo.find_commit(id).ok());
    let branches = repo.branches().unwrap_or_default();
    let tags = repo.tags().unwrap_or_default();

    let work_dir = repo
        .work_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "<bare>".to_string());

    if cli.json {
        return print_json(&json!({
            "work_dir": work_dir,
            "git_dir": repo.git_dir().display().to_string(),
            "head": head.map(|id| id.to_string()),
            "head_message": head_commit.as_ref().map(|c| c.message.clone()),
            "state": repo.state(),
            "branches": branches.len(),
            "tags": tags.len(),
        }));
    }

    println!("GitX repository");
    println!("  work dir : {work_dir}");
    println!("  git dir  : {}", repo.git_dir().display());
    if let Some(commit) = &head_commit {
        println!("  head     : {} {}", short_oid(&commit.id), commit.message);
    } else {
        println!("  head     : (no commits yet)");
    }
    println!(
        "  state    : {}",
        repo.state().unwrap_or_else(|| "clean".into())
    );
    println!("  branches : {}", branches.len());
    println!("  tags     : {}", tags.len());
    Ok(())
}

pub fn status(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let state = repo.state();
    let head = repo.head_commit_id().ok();

    if cli.json {
        return print_json(&json!({
            "state": state,
            "head": head.map(|id| id.to_string()),
        }));
    }

    match state.as_deref() {
        Some("Clean") | None => println!("clean"),
        Some(other) => println!("{other} (merge/rebase in progress)"),
    }
    match head {
        Some(id) => {
            let commit = repo.find_commit(id)?;
            println!("HEAD at {} — {}", short_oid(&id), commit.message);
        }
        None => println!("HEAD unborn (no commits)"),
    }
    Ok(())
}

pub fn stats(cli: &Cli) -> anyhow::Result<()> {
    let repo = open_repo(cli)?;
    let stats = gitx_analysis::repository_stats(&repo)?;

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

    println!("Repository statistics");
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
