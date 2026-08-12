use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "gitx",
    author,
    version,
    about = "GitX — local-first repository intelligence and code archaeology"
)]
pub struct Cli {
    /// Path to the repository (discovered from the current directory by default).
    #[arg(long, global = true)]
    pub repo: Option<PathBuf>,

    /// Emit machine-readable JSON on stdout (analytical commands).
    #[arg(long, global = true)]
    pub json: bool,

    /// Disable ANSI colors in human output.
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Suppress non-essential output.
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Enable verbose diagnostics on stderr.
    #[arg(long, global = true)]
    pub verbose: bool,

    /// Path to a configuration file.
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// Do not use the persisted index.
    #[arg(long, global = true)]
    pub no_cache: bool,

    /// Force a refresh of the index before running.
    #[arg(long, global = true)]
    pub refresh: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Repository identity and location.
    Info,
    /// Repository state (merging, rebasing, ...) and HEAD.
    Status,
    /// High-level repository statistics.
    Stats,
    /// Build the local repository index.
    Scan,
    /// Rebuild the local repository index from scratch.
    Refresh,
    /// Inspect and manage the local index.
    Index {
        #[command(subcommand)]
        action: IndexAction,
    },
    /// Line-level attribution for a file.
    Blame {
        path: String,
        /// Maximum number of lines to attribute (paginated; docs/07 §7).
        #[arg(long, default_value_t = 500)]
        limit: usize,
    },
    /// Commit timeline.
    Timeline {
        /// Only commits whose author name/email contains this.
        #[arg(long)]
        author: Option<String>,
        /// Only commits whose committer name/email contains this.
        #[arg(long)]
        committer: Option<String>,
        /// Only commits at or after this date (RFC3339 or unix seconds).
        #[arg(long)]
        since: Option<String>,
        /// Only commits at or before this date (RFC3339 or unix seconds).
        #[arg(long)]
        until: Option<String>,
        /// Only commits reachable from this branch.
        #[arg(long)]
        branch: Option<String>,
        /// Only commits touching this path.
        #[arg(long)]
        path: Option<String>,
        /// Only merge commits (2+ parents).
        #[arg(long)]
        merges: bool,
        /// Exclude merge commits.
        #[arg(long)]
        no_merges: bool,
        /// Maximum number of commits to show.
        #[arg(long)]
        max: Option<usize>,
    },
    /// Show a single commit in detail.
    Commit { oid: String },
    /// History of a file (or directory).
    History {
        path: String,
        /// Follow renames (accepted; renames are resolved along the mainline).
        #[arg(long)]
        follow: bool,
        /// Only commits at or after this date.
        #[arg(long)]
        since: Option<String>,
        /// Show line-level history (which commit introduced each line).
        #[arg(long)]
        lines: bool,
    },
    /// Life of a file: every commit that touched it, following renames
    /// (docs/10 file archaeology).
    Lineage { path: String },
    /// List branches.
    Branches,
    /// Show a single branch.
    Branch { name: String },
    /// Contributor statistics.
    Contributors,
    /// A single contributor's activity.
    Contributor { name: String },
    /// Per-file ownership concentration.
    Ownership {
        /// Restrict to a path prefix.
        path: Option<String>,
    },
    /// Change/maintenance hotspots (highest-risk files first).
    Hotspots {
        /// Show at most N files.
        #[arg(long, default_value_t = 20)]
        limit: usize,
        /// Restrict to a path prefix.
        #[arg(long)]
        path: Option<String>,
    },
    /// Directory/module evolution overview and structural diffs.
    Architecture {
        /// Diff from this ref (implies `--to`; equivalent to
        /// `architecture diff <from> <to>`).
        #[arg(long)]
        from: Option<String>,
        /// Diff to this ref (implies `--from`).
        #[arg(long)]
        to: Option<String>,
        #[command(subcommand)]
        action: Option<ArchitectureAction>,
    },
    /// Dependency overview and history from declared manifests.
    Dependencies {
        #[command(subcommand)]
        action: Option<DependenciesAction>,
    },
    /// Evidence-backed risk score for a file (or all files).
    Risk { path: Option<String> },
    /// Composite repository health with all six sub-scores.
    Health,
    /// Recurring bug-fix and regression areas (docs/10 §9).
    Regressions {
        /// Maximum number of commits to analyze.
        #[arg(long, default_value_t = 2000)]
        max: usize,
    },
    /// Source symbols extracted from HEAD (heuristic; docs/21 Stage 6).
    Symbols {
        /// Restrict to a path prefix.
        #[arg(long)]
        path: Option<String>,
        #[command(subcommand)]
        action: Option<SymbolsAction>,
    },
    /// Module/file dependency graph of the HEAD tree (docs/21 Stage 6).
    Graph,
    /// Search commits, files, authors, branches and tags.
    Search {
        query: String,
        /// Only commits at or after this date (RFC3339 or unix seconds).
        #[arg(long)]
        since: Option<String>,
        /// Only commits at or before this date (RFC3339 or unix seconds).
        #[arg(long)]
        until: Option<String>,
        /// Restrict file/symbol/directory results to this path prefix.
        #[arg(long)]
        path: Option<String>,
        /// Search commit messages only.
        #[arg(long)]
        commits: bool,
        /// Search file paths only.
        #[arg(long)]
        files: bool,
        /// Search authors only.
        #[arg(long)]
        authors: bool,
        /// Search branches only.
        #[arg(long)]
        branches: bool,
        /// Search tags only.
        #[arg(long)]
        tags: bool,
        /// Include rename history in file search.
        #[arg(long)]
        renames: bool,
        /// Include file content in search (bounded).
        #[arg(long)]
        code: bool,
        /// Include history (commits + files).
        #[arg(long)]
        history: bool,
        /// Search symbols extracted from source (docs/11 §2).
        #[arg(long)]
        symbols: bool,
        /// Search directories containing matching paths (docs/11 §2).
        #[arg(long)]
        directories: bool,
        /// Restrict commit search to an author.
        #[arg(long)]
        author: Option<String>,
    },
    /// Recovery report: reflogs and unreachable commits (read-only).
    Recovery {
        #[command(subcommand)]
        action: Option<RecoveryAction>,
    },
    /// Commits present in the object database but unreachable from any ref.
    Unreachable,
    /// Release information: a tag's commits, or the diff between two refs
    /// (docs/07 §17). `gitx release <TAG>` is shorthand for
    /// `gitx release show <TAG>`.
    Release {
        /// Tag to inspect (equivalent to `release show <TAG>`).
        tag: Option<String>,
        #[command(subcommand)]
        action: Option<ReleaseAction>,
    },
    /// Show or initialize the configuration (docs/16).
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Generate shell completions (docs/07).
    Completions {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum DependenciesAction {
    /// Current declared dependencies in HEAD (default).
    List,
    /// Dependency history: added/removed/version-changed across commits.
    /// Scans the manifest at each commit on the mainline (docs/10 §11).
    History {
        /// Maximum number of commits to scan.
        #[arg(long, default_value_t = 500)]
        max: usize,
    },
    /// Dependency diff between two refs (branch, tag, or commit id).
    Diff { from: String, to: String },
    /// Show the workspace layout (root + members) for monorepos.
    Workspace,
    /// Cargo feature flags + pnpm catalogs declared in HEAD (docs/10 §11).
    Features,
    /// Dependency usage + churn: how many files reference each dependency,
    /// and how often it was added/removed/version-changed (docs/10 §11).
    Usage {
        /// Maximum number of commits to scan for churn.
        #[arg(long, default_value_t = 500)]
        max: usize,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum ArchitectureAction {
    /// Directory/module evolution overview (default).
    Overview,
    /// Structural diff between two commits (docs/07 §11, docs/10 §10).
    Diff { from: String, to: String },
    /// Detect architectural milestones from history (docs/10 §10): initial
    /// commit, first release tag, module additions, structural refactors,
    /// and dependency-direction changes.
    Milestones {
        /// Maximum number of commits to walk (newest first).
        #[arg(long, default_value_t = 1000)]
        max: usize,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum SymbolsAction {
    /// Life of a symbol: when it was added, moved, or removed along the
    /// mainline (docs/21 Stage 6).
    History { name: String },
}

#[derive(Subcommand, Debug, Clone, Copy)]
pub enum ConfigAction {
    /// Show the effective configuration (defaults overlaid by the file).
    Show,
    /// Write an example configuration file.
    Init,
}

#[derive(Subcommand, Debug)]
pub enum IndexAction {
    /// Show index location and commit count.
    Status,
    /// Rebuild the index from scratch.
    Rebuild,
    /// Delete the persisted index.
    Clear,
}

#[derive(Subcommand, Debug)]
pub enum ReleaseAction {
    /// Show a tag and the commits it points at.
    Show { tag: String },
    /// Summary of commits and file changes between two refs.
    Diff { from: String, to: String },
}

#[derive(Subcommand, Debug)]
pub enum RecoveryAction {
    /// Show reflog entries (default).
    Reflog,
    /// Show commits unreachable from any ref.
    Unreachable,
    /// Show a commit (or any object) by id.
    Show { oid: String },
    /// Export a commit as a unified patch (read-only; docs/12 §6).
    Export {
        oid: String,
        /// Write the patch to this file (default: `gitx-recovery-<oid>.patch`).
        #[arg(long, value_name = "PATH")]
        output: Option<std::path::PathBuf>,
    },
}
