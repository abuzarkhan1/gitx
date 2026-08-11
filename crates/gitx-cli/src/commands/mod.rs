pub mod analysis;
pub mod completions;
pub mod config;
pub mod history;
pub mod index;
pub mod recovery;
pub mod repo;
pub mod search;

use crate::cli::{Cli, Commands};
use anyhow::Context;

pub fn dispatch(mut cli: Cli) -> anyhow::Result<()> {
    // No subcommand → start the TUI (docs/07 §1). The TUI crate is a separate
    // binary today; surface a clear message instead of a silent no-op.
    if cli.command.is_none() {
        eprintln!(
            "gitx: the interactive TUI is a separate binary (gitx-tui); run `cargo run -p gitx-tui`"
        );
        eprintln!("gitx: use `gitx --help` for the available commands");
        return Ok(());
    }

    match cli.command.take().expect("checked above") {
        Commands::Info => repo::info(&cli),
        Commands::Status => repo::status(&cli),
        Commands::Stats => repo::stats(&cli),
        Commands::Scan => index::scan(&cli),
        Commands::Refresh => index::refresh(&cli),
        Commands::Index { action } => index::index_command(&cli, action),
        Commands::Timeline {
            author,
            since,
            until,
            branch,
            path,
            max,
        } => history::timeline(&cli, author, since, until, branch, path, max),
        Commands::Commit { oid } => history::commit(&cli, &oid),
        Commands::History {
            path,
            follow,
            since,
            lines,
        } => history::file_history(&cli, &path, follow, since, lines),
        Commands::Blame { path } => history::blame(&cli, &path),
        Commands::Branches => history::branches(&cli),
        Commands::Branch { name } => history::branch(&cli, &name),
        Commands::Contributors => analysis::contributors(&cli),
        Commands::Contributor { name } => analysis::contributor(&cli, &name),
        Commands::Ownership { path } => analysis::ownership(&cli, path.as_deref()),
        Commands::Hotspots { limit, path } => analysis::hotspots(&cli, limit, path.as_deref()),
        Commands::Architecture { action } => match action {
            Some(crate::cli::ArchitectureAction::Diff { from, to }) => {
                analysis::architecture_diff(&cli, &from, &to)
            }
            _ => analysis::architecture(&cli),
        },
        Commands::Dependencies { action } => analysis::dependencies(&cli, action),
        Commands::Risk { path } => analysis::risk(&cli, path.as_deref()),
        Commands::Health => analysis::health(&cli),
        Commands::Search {
            query,
            commits,
            files,
            authors,
            branches,
            tags,
            renames,
            code,
            history,
            author,
        } => search::search(
            &cli, &query, commits, files, authors, branches, tags, renames, code, history, author,
        ),
        Commands::Recovery { action } => recovery::recovery(&cli, action),
        Commands::Unreachable => recovery::unreachable(&cli),
        Commands::Release { action } => recovery::release(&cli, action),
        Commands::Config { action } => config::config_command(&cli, action),
        Commands::Completions { shell } => completions::completions(shell),
    }
}

/// Open the repository from `--repo` or discover it from the current directory.
pub fn open_repo(cli: &Cli) -> anyhow::Result<gitx_git::Repository> {
    match &cli.repo {
        Some(path) => gitx_git::Repository::discover(path)
            .with_context(|| format!("cannot open repository at {}", path.display())),
        None => gitx_git::Repository::discover(".")
            .with_context(|| "not inside a Git repository (use --repo <PATH>)"),
    }
}

/// Abbreviated object id (7 hex chars), as shown by git.
pub fn short_oid(id: &gitx_git::models::ObjectId) -> String {
    id.to_string().chars().take(7).collect()
}

/// Format a unix timestamp as a local RFC3339-ish string.
pub fn format_ts(seconds: i64) -> String {
    match chrono::DateTime::from_timestamp(seconds, 0) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        None => seconds.to_string(),
    }
}

/// Parse a user-supplied date: unix seconds or RFC3339.
pub fn parse_ts(input: &str) -> anyhow::Result<i64> {
    if let Ok(secs) = input.parse::<i64>() {
        return Ok(secs);
    }
    let dt = chrono::DateTime::parse_from_rfc3339(input)
        .with_context(|| format!("invalid date `{input}` (use RFC3339 or unix seconds)"))?;
    Ok(dt.timestamp())
}

/// Emit JSON on stdout (docs/07 §18: JSON output must not mix with decorative output).
pub fn print_json(value: &serde_json::Value) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

/// Resolve a commit reference: full/abbreviated hex object id, branch name, or
/// tag name.
pub fn resolve_ref(
    repo: &gitx_git::Repository,
    name: &str,
) -> anyhow::Result<gitx_git::models::ObjectId> {
    if name == "HEAD" {
        return repo
            .head_commit_id()
            .map_err(|e| anyhow::anyhow!(e.to_string()));
    }
    if let Some(id) = gitx_git::models::ObjectId::from_hex(name) {
        return Ok(id);
    }
    if name.chars().all(|c| c.is_ascii_hexdigit()) {
        let mut matches = Vec::new();
        for id_res in repo.all_object_ids()? {
            let id = id_res?;
            if id.to_string().starts_with(name) {
                matches.push(id);
            }
        }
        match matches.len() {
            1 => return Ok(matches[0]),
            0 => {}
            n => anyhow::bail!("`{name}` is ambiguous ({n} candidates)"),
        }
    }
    for branch in repo.branches()? {
        if branch.name == name {
            return Ok(branch.target);
        }
    }
    for tag in repo.tags()? {
        if tag.name == name {
            return Ok(tag.target);
        }
    }
    anyhow::bail!("cannot resolve reference `{name}`")
}
