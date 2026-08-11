#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Overview,
    Timeline,
    Commits,
    Branches,
    Files,
    Contributors,
    Hotspots,
    Ownership,
    Architecture,
    Dependencies,
    Risk,
    Health,
    Recovery,
    Search,
    /// Commit or file detail opened from a list row (docs/08 drill-down).
    Detail,
}

/// What the detail view is showing.
#[derive(Debug, Clone)]
pub enum Detail {
    Commit { oid: String },
    File { path: std::path::PathBuf },
}

pub struct App {
    pub current_view: View,
    pub nav_index: usize,
    pub running: bool,
    pub error: Option<String>,
    pub loading: bool,
    /// Repository statistics shown by the Overview view (docs/08).
    pub stats: Option<gitx_analysis::RepoStats>,
    /// Commit timeline (newest first, capped) for the Timeline/Commits views.
    pub timeline: Option<Vec<gitx_git::models::Commit>>,
    /// Full hotspot/risk analysis for the Hotspots/Risk/Files/Ownership views.
    pub hotspots: Option<gitx_analysis::RepoAnalysis>,
    /// Branches (docs/08 Branches panel).
    pub branches: Option<Vec<gitx_git::models::Branch>>,
    /// Contributors: author key → commit count, sorted descending.
    pub contributors: Option<Vec<(String, u64)>>,
    /// Declared dependencies in HEAD (docs/08 Dependencies panel).
    pub dependencies: Option<Vec<(std::path::PathBuf, Vec<gitx_analysis::manifest::Dependency>)>>,
    /// Recovery data: reflog entries + unreachable commits (docs/08 Recovery).
    pub recovery: Option<gitx_analysis::RecoveryReport>,
    /// Search query being typed (docs/08 Search panel).
    pub search_query: String,
    /// Search results: matching timeline commits.
    pub search_results: Option<Vec<gitx_git::models::Commit>>,
    /// The repository work directory (when discovered).
    pub repo_path: Option<String>,
    /// Scroll offset for the current content view (docs/08: j/k scrolls).
    pub scroll: usize,
    /// Whether keyboard input scrolls the content view (true) or navigates
    /// the sidebar (false). Enter opens a view; Esc/← returns to navigation.
    pub in_content: bool,
    /// Selected row index within the current view (when the view supports
    /// selection).
    pub selected: usize,
    /// The currently open detail (commit or file), if any.
    pub detail: Option<Detail>,
    /// Pre-rendered detail content (computed once at open time so rendering
    /// stays fast and non-blocking).
    pub detail_text: Option<Vec<String>>,
    /// Scroll offset within the detail view.
    pub detail_scroll: usize,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let (
            stats,
            timeline,
            hotspots,
            branches,
            contributors,
            dependencies,
            recovery,
            repo_path,
            error,
        ) = load_repo_stats();
        Self {
            current_view: View::Overview,
            nav_index: 0,
            running: true,
            error,
            loading: false,
            stats,
            timeline,
            hotspots,
            branches,
            contributors,
            dependencies,
            recovery,
            search_query: String::new(),
            search_results: None,
            repo_path,
            scroll: 0,
            in_content: false,
            selected: 0,
            detail: None,
            detail_text: None,
            detail_scroll: 0,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn set_view(&mut self, view: View) {
        self.current_view = view;
    }

    pub fn next_nav(&mut self) {
        if self.nav_index < 13 {
            self.nav_index += 1;
        }
    }

    pub fn prev_nav(&mut self) {
        if self.nav_index > 0 {
            self.nav_index -= 1;
        }
    }

    pub fn select_nav(&mut self) {
        self.current_view = match self.nav_index {
            0 => View::Overview,
            1 => View::Timeline,
            2 => View::Commits,
            3 => View::Branches,
            4 => View::Files,
            5 => View::Contributors,
            6 => View::Hotspots,
            7 => View::Ownership,
            8 => View::Architecture,
            9 => View::Dependencies,
            10 => View::Risk,
            11 => View::Health,
            12 => View::Recovery,
            13 => View::Search,
            _ => View::Overview,
        };
        // Enter opens the view: subsequent j/k scroll its content.
        self.in_content = true;
        self.scroll = 0;
        self.selected = 0;
    }

    /// The nav index for the current view (used to keep the sidebar highlight
    /// in sync when a view is opened via the search path).
    pub fn sync_nav(&mut self) {
        self.nav_index = match self.current_view {
            View::Overview => 0,
            View::Timeline => 1,
            View::Commits => 2,
            View::Branches => 3,
            View::Files => 4,
            View::Contributors => 5,
            View::Hotspots => 6,
            View::Ownership => 7,
            View::Architecture => 8,
            View::Dependencies => 9,
            View::Risk => 10,
            View::Health => 11,
            View::Recovery => 12,
            View::Search => 13,
            View::Detail => 13,
        };
    }

    /// Scroll the current view down by one row. The renderer clamps the offset
    /// to the actual row count, so an unbounded increment is safe.
    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    /// Scroll the current view up by one row.
    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    /// Open the detail view for the currently selected row (docs/08 drill-down):
    /// a commit on Timeline/Commits/Search, a file on Files/Hotspots/Risk/
    /// Ownership. The content is computed once and cached.
    pub fn open_detail(&mut self) {
        let detail = match self.current_view {
            View::Timeline | View::Commits => self.timeline.as_ref().and_then(|list| {
                list.get(self.selected).map(|c| Detail::Commit {
                    oid: c.id.to_string(),
                })
            }),
            View::Search => self.search_results.as_ref().and_then(|list| {
                list.get(self.selected).map(|c| Detail::Commit {
                    oid: c.id.to_string(),
                })
            }),
            View::Files | View::Hotspots | View::Risk | View::Ownership => self
                .hotspots
                .as_ref()
                .and_then(|a| a.files.get(self.selected))
                .map(|f| Detail::File {
                    path: f.path.clone(),
                }),
            _ => None,
        };
        let Some(detail) = detail else { return };

        // Remember which panel opened the detail so Esc returns there.
        self.sync_nav();
        let text = self.render_detail(&detail);
        self.detail = Some(detail);
        self.detail_text = Some(text);
        self.detail_scroll = 0;
        self.current_view = View::Detail;
        self.in_content = true;
    }

    /// Close the detail view back to the panel it was opened from.
    pub fn close_detail(&mut self) {
        self.detail = None;
        self.detail_text = None;
        self.detail_scroll = 0;
        self.current_view = self.panel_for_nav(self.nav_index);
        self.in_content = true;
    }

    fn panel_for_nav(&self, index: usize) -> View {
        match index {
            0 => View::Overview,
            1 => View::Timeline,
            2 => View::Commits,
            3 => View::Branches,
            4 => View::Files,
            5 => View::Contributors,
            6 => View::Hotspots,
            7 => View::Ownership,
            8 => View::Architecture,
            9 => View::Dependencies,
            10 => View::Risk,
            11 => View::Health,
            12 => View::Recovery,
            _ => View::Search,
        }
    }

    /// Build the detail text for a commit or file. Discovers the repository
    /// from the stored path; falls back to a helpful message.
    fn render_detail(&self, detail: &Detail) -> Vec<String> {
        let repo = match &self.repo_path {
            Some(path) => match gitx_git::Repository::discover(path) {
                Ok(r) => r,
                Err(e) => return vec![format!("cannot open repository: {e}")],
            },
            None => return vec!["No repository loaded.".to_string()],
        };
        match detail {
            Detail::Commit { oid } => render_commit_detail(&repo, oid),
            Detail::File { path } => render_file_detail(&repo, path),
        }
    }

    /// Re-run the in-memory search over the loaded timeline (docs/08 Search).
    pub fn run_search(&mut self) {
        let query = self.search_query.trim().to_lowercase();
        if query.is_empty() {
            self.search_results = None;
            return;
        }
        let results = self
            .timeline
            .iter()
            .flatten()
            .filter(|c| {
                c.message.to_lowercase().contains(&query)
                    || c.author.name.to_lowercase().contains(&query)
                    || c.author.email.to_lowercase().contains(&query)
                    || c.id.to_string().contains(&query)
            })
            .cloned()
            .collect();
        self.search_results = Some(results);
    }
}

/// All repository data loaded eagerly at startup, keyed by the panels that
/// consume it (docs/08).
type LoadedData = (
    Option<gitx_analysis::RepoStats>,
    Option<Vec<gitx_git::models::Commit>>,
    Option<gitx_analysis::RepoAnalysis>,
    Option<Vec<gitx_git::models::Branch>>,
    Option<Vec<(String, u64)>>,
    Option<Vec<(std::path::PathBuf, Vec<gitx_analysis::manifest::Dependency>)>>,
    Option<gitx_analysis::RecoveryReport>,
    Option<String>,
    Option<String>,
);

/// Load repository data eagerly at startup by discovering the repository from
/// the current directory. Blocks briefly; acceptable for V1 (docs/08 notes the
/// overview is the first-render anchor).
fn load_repo_stats() -> LoadedData {
    let repo = match gitx_git::Repository::discover(".") {
        Ok(repo) => repo,
        Err(err) => {
            return (
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                Some(format!("not inside a Git repository: {err}")),
            );
        }
    };
    let path = repo.work_dir().map(|p| p.display().to_string());

    let stats = gitx_analysis::repository_stats(&repo).ok();
    let service = gitx_history::timeline::HistoryService::new(&repo);
    let timeline = service
        .timeline(gitx_history::timeline::TimelineOptions {
            max_count: Some(500),
            ..Default::default()
        })
        .ok();
    let hotspots = gitx_analysis::analyze_repository(&repo).ok();
    let branches = repo.branches().ok();

    // Contributors from the timeline (author key → commit count).
    let contributors = timeline.as_ref().map(|commits| {
        let mut counts: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        for c in commits {
            let key = format!("{} <{}>", c.author.name, c.author.email);
            *counts.entry(key).or_insert(0) += 1;
        }
        let mut list: Vec<(String, u64)> = counts.into_iter().collect();
        list.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        list
    });

    let dependencies = gitx_analysis::manifest::head_dependencies(&repo).ok();
    // Cap the object-database scan so TUI startup stays fast on large repos
    // (the CLI `gitx recovery` does the full scan on demand).
    let recovery = gitx_analysis::recovery::analyze_recovery_capped(&repo).ok();

    (
        stats,
        timeline,
        hotspots,
        branches,
        contributors,
        dependencies,
        recovery,
        path,
        None,
    )
}

/// Render a commit's full detail: metadata, message, and changed files with
/// diff stats (mirrors `gitx commit <oid>`).
fn render_commit_detail(repo: &gitx_git::Repository, oid: &str) -> Vec<String> {
    let Some(id) = gitx_git::models::ObjectId::from_hex(oid) else {
        return vec![format!("invalid object id `{oid}`")];
    };
    let commit = match repo.find_commit(id) {
        Ok(c) => c,
        Err(e) => return vec![format!("cannot read commit {oid}: {e}")],
    };
    let parent_tree = match commit.parents.first() {
        Some(parent) => repo.find_commit(*parent).ok().map(|p| p.tree_id),
        None => None,
    };
    let changes = repo
        .diff_tree_to_tree(parent_tree, commit.tree_id)
        .unwrap_or_default();
    let insertions: u32 = changes.iter().map(|c| c.insertions).sum();
    let deletions: u32 = changes.iter().map(|c| c.deletions).sum();

    let mut out = Vec::new();
    out.push(format!("commit {}", commit.id));
    out.push(format!(
        "Author: {} <{}>",
        commit.author.name, commit.author.email
    ));
    out.push(format!("Date:   {}", ts(commit.author.time)));
    if commit.committer.email != commit.author.email {
        out.push(format!(
            "Committer: {} <{}>",
            commit.committer.name, commit.committer.email
        ));
    }
    if !commit.parents.is_empty() {
        out.push(format!(
            "Parents: {}",
            commit
                .parents
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        ));
    }
    out.push(String::new());
    for line in commit.message.lines() {
        out.push(format!("    {line}"));
    }
    out.push(String::new());
    out.push(format!(
        " {} files changed, {} insertions(+), {} deletions(-)",
        changes.len(),
        insertions,
        deletions
    ));
    for change in changes.iter().take(60) {
        out.push(format!(
            "  {:?} {:>5} {:>5}  {}",
            change.change_type,
            change.insertions,
            change.deletions,
            change.path.display()
        ));
    }
    out
}

/// Render a file's history: every commit that touched it (mirrors
/// `gitx history <path>`).
fn render_file_detail(repo: &gitx_git::Repository, path: &std::path::Path) -> Vec<String> {
    let service = gitx_history::timeline::HistoryService::new(repo);
    let commits = match service.timeline(gitx_history::timeline::TimelineOptions {
        max_count: Some(200),
        from: None,
        path: Some(path.to_path_buf()),
        author: None,
        since: None,
        until: None,
    }) {
        Ok(c) => c,
        Err(e) => return vec![format!("cannot read history for {}: {e}", path.display())],
    };
    let mut out = vec![format!("History of {}", path.display())];
    if commits.is_empty() {
        out.push("  (no commits touch this path)".to_string());
        return out;
    }
    for commit in &commits {
        out.push(format!(
            "{}  {}  {}  {}",
            short(&commit.id),
            ts(commit.author.time),
            commit.author.name,
            one_line(&commit.message)
        ));
    }
    out
}

fn ts(seconds: i64) -> String {
    match chrono::DateTime::from_timestamp(seconds, 0) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        None => seconds.to_string(),
    }
}

fn short(id: &gitx_git::models::ObjectId) -> String {
    id.to_string().chars().take(7).collect()
}

fn one_line(message: &str) -> String {
    message.lines().next().unwrap_or("").to_string()
}
