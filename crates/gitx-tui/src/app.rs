use chrono::Datelike;

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
    /// Contributors: key, display name, commit count, first/last activity,
    /// files touched (from live analysis; empty from the cache).
    pub contributors: Option<Vec<Contributor>>,
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
    /// True while the keybinding help overlay is shown (docs/08: `?` help).
    pub show_help: bool,
    /// Weekly commit counts for the Overview activity chart (docs/08 §3),
    /// oldest → newest (last 12 weeks).
    pub activity: Option<Vec<(String, u32)>>,
    /// Changed-file count per timeline commit, aligned with `timeline`
    /// (docs/08 Timeline: changed-file count column).
    pub timeline_file_counts: Option<Vec<u32>>,
    /// Top affected directory per timeline commit (docs/08 Commit view:
    /// affected-areas). Empty string when none.
    pub timeline_areas: Option<Vec<String>>,
    /// Tip-commit timestamp per branch, aligned with `branches` (docs/08
    /// Branch view: age + activity).
    pub branch_tips: Option<Vec<i64>>,
    /// Repository state string from gix (clean/merge/rebase...).
    pub repo_state: Option<String>,
    /// Per-sub-score evidence lines for the Health view (docs/08 §3: selecting
    /// a sub-score reveals its evidence).
    pub health_evidence: Vec<Vec<String>>,
    /// Receiver for the background data load (docs/08 loading progress): the
    /// repo data is computed on a worker thread so the UI renders immediately
    /// and the status bar shows progress while it loads.
    data_rx: Option<std::sync::mpsc::Receiver<AppData>>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Start with no data: the repository load runs on a worker thread and
    /// lands via [`App::apply_data`] once ready (docs/08 loading progress).
    pub fn new() -> Self {
        Self::spawn_load()
    }

    fn spawn_load() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let data = load_repo_stats();
            let _ = tx.send(data);
        });
        Self {
            current_view: View::Overview,
            nav_index: 0,
            running: true,
            error: None,
            loading: true,
            stats: None,
            timeline: None,
            hotspots: None,
            branches: None,
            contributors: None,
            dependencies: None,
            recovery: None,
            search_query: String::new(),
            search_results: None,
            repo_path: None,
            scroll: 0,
            in_content: false,
            selected: 0,
            detail: None,
            detail_text: None,
            detail_scroll: 0,
            show_help: false,
            activity: None,
            timeline_file_counts: None,
            branch_tips: None,
            repo_state: None,
            health_evidence: Vec::new(),
            data_rx: Some(rx),
        }
    }

    /// Apply the background-loaded repository data (docs/08 loading progress).
    pub fn apply_data(&mut self, data: AppData) {
        self.error = data.error;
        self.stats = data.stats;
        self.timeline = data.timeline;
        self.hotspots = data.hotspots;
        self.branches = data.branches;
        self.contributors = data.contributors;
        self.dependencies = data.dependencies;
        self.recovery = data.recovery;
        self.repo_path = data.repo_path;
        self.activity = data.activity;
        self.timeline_file_counts = data.timeline_file_counts;
        self.branch_tips = data.branch_tips;
        self.repo_state = data.repo_state;
        self.health_evidence = data.health_evidence;
        self.loading = false;
        // Data changed: invalidate stale search results.
        self.search_results = None;
    }

    /// Reload all repository data on a worker thread (docs/08: `r` refresh).
    /// The status bar shows progress while the new data loads.
    pub fn reload(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let data = load_repo_stats();
            let _ = tx.send(data);
        });
        self.data_rx = Some(rx);
        self.loading = true;
    }

    /// Poll the background loader; returns true when new data landed.
    pub fn poll_load(&mut self) -> bool {
        let Some(rx) = &self.data_rx else {
            return false;
        };
        match rx.try_recv() {
            Ok(data) => {
                self.apply_data(data);
                self.data_rx = None;
                true
            }
            Err(_) => false,
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
/// A contributor row for the Contributors panel (docs/08 §3).
#[derive(Debug, Clone)]
pub struct Contributor {
    pub key: String,
    pub name: String,
    pub commits: u64,
    pub first_activity: i64,
    pub last_activity: i64,
    /// Number of distinct files the author touched (0 when unknown, e.g.
    /// when reading from the analysis cache).
    pub files_touched: u64,
}

pub struct AppData {
    pub stats: Option<gitx_analysis::RepoStats>,
    pub timeline: Option<Vec<gitx_git::models::Commit>>,
    pub hotspots: Option<gitx_analysis::RepoAnalysis>,
    pub branches: Option<Vec<gitx_git::models::Branch>>,
    pub contributors: Option<Vec<Contributor>>,
    pub dependencies: Option<Vec<(std::path::PathBuf, Vec<gitx_analysis::manifest::Dependency>)>>,
    pub recovery: Option<gitx_analysis::RecoveryReport>,
    pub repo_path: Option<String>,
    pub error: Option<String>,
    pub activity: Option<Vec<(String, u32)>>,
    pub timeline_file_counts: Option<Vec<u32>>,
    pub timeline_areas: Option<Vec<String>>,
    pub branch_tips: Option<Vec<i64>>,
    pub repo_state: Option<String>,
    pub health_evidence: Vec<Vec<String>>,
}

/// Load repository data eagerly at startup by discovering the repository from
/// the current directory. Blocks briefly; acceptable for V1 (docs/08 notes the
/// overview is the first-render anchor).
fn load_repo_stats() -> AppData {
    let repo = match gitx_git::Repository::discover(".") {
        Ok(repo) => repo,
        Err(err) => {
            return AppData {
                stats: None,
                timeline: None,
                hotspots: None,
                branches: None,
                contributors: None,
                dependencies: None,
                recovery: None,
                repo_path: None,
                error: Some(format!("not inside a Git repository: {err}")),
                activity: None,
                timeline_file_counts: None,
                branch_tips: None,
                repo_state: None,
                health_evidence: Vec::new(),
            };
        }
    };
    let path = repo.work_dir().map(|p| p.display().to_string());
    let repo_state = repo.state();

    // Services layer (docs/04 §6): statistics and analysis go through
    // `RepositoryService`/`AnalysisService`, which prefer the fresh persisted
    // index and fall back to live Git computation (docs/13 §3).
    let stats = index_stats_or_live(&repo);
    let service = gitx_history::timeline::HistoryService::new(&repo);
    let timeline = service
        .timeline(gitx_history::timeline::TimelineOptions {
            max_count: Some(500),
            ..Default::default()
        })
        .ok();
    // Index-backed analysis via AnalysisService (docs/04 §6, docs/13 §3):
    // fresh cache → read; otherwise live computation.
    let hotspots = gitx_services::AnalysisService::new(&repo)
        .analyze(true, gitx_analysis::hotspots::HotspotWeights::default())
        .ok();
    let branches = repo.branches().ok();

    // Contributors from the timeline: commit count + first/last activity +
    // files touched (from live analysis author_lines; empty from the cache).
    let files_by_author = hotspots.as_ref().map(|a| {
        let mut map: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        for file in &a.files {
            for (author, _) in &file.author_lines {
                *map.entry(author.clone()).or_insert(0) += 1;
            }
        }
        map
    });
    let contributors = timeline.as_ref().map(|commits| {
        let mut counts: std::collections::HashMap<String, Contributor> =
            std::collections::HashMap::new();
        for c in commits {
            let key = format!("{} <{}>", c.author.name, c.author.email);
            let entry = counts.entry(key.clone()).or_insert(Contributor {
                key: key.clone(),
                name: c.author.name.clone(),
                commits: 0,
                first_activity: i64::MAX,
                last_activity: i64::MIN,
                files_touched: 0,
            });
            entry.commits += 1;
            entry.first_activity = entry.first_activity.min(c.author.time);
            entry.last_activity = entry.last_activity.max(c.author.time);
            if let Some(map) = &files_by_author {
                entry.files_touched = map.get(&key).copied().unwrap_or(0);
            }
        }
        let mut list: Vec<Contributor> = counts.into_values().collect();
        list.sort_by_key(|c| std::cmp::Reverse(c.commits));
        list
    });

    // Changed-file count + top affected directory per timeline commit
    // (docs/08 Timeline column, Commit view affected-areas).
    let per_commit_areas = timeline.as_ref().map(|commits| {
        commits
            .iter()
            .map(|c| {
                let parent_tree = match c.parents.first() {
                    Some(parent) => repo.find_commit(*parent).ok().map(|p| p.tree_id),
                    None => None,
                };
                let changes = repo.diff_tree_to_tree(parent_tree, c.tree_id).unwrap_or_default();
                let mut dirs: std::collections::HashMap<String, u32> =
                    std::collections::HashMap::new();
                for change in &changes {
                    let dir = change
                        .path
                        .parent()
                        .filter(|p| !p.as_os_str().is_empty())
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| ".".to_string());
                    *dirs.entry(dir).or_insert(0) += 1;
                }
                let top = dirs.into_iter().max_by_key(|(_, n)| *n).map(|(d, _)| d);
                (changes.len() as u32, top.unwrap_or_else(|| String::new()))
            })
            .collect::<Vec<(u32, String)>>()
    });
    let timeline_file_counts = per_commit_areas
        .as_ref()
        .map(|v| v.iter().map(|(n, _)| *n).collect());
    let timeline_areas = per_commit_areas
        .as_ref()
        .map(|v| v.iter().map(|(_, a)| a.clone()).collect());

    // Weekly commit counts, last 12 weeks (docs/08 Overview activity chart).
    let activity = timeline.as_ref().map(|commits| {
        let now = chrono::Utc::now();
        let mut buckets: Vec<(String, u32)> = (0..12)
            .map(|i| {
                let week = now - chrono::Duration::weeks(i);
                (
                    format!("{}-W{:02}", week.format("%G"), week.iso_week().week()),
                    0,
                )
            })
            .collect();
        buckets.reverse();
        for c in commits {
            let Some(dt) = chrono::DateTime::from_timestamp(c.author.time, 0) else {
                continue;
            };
            let label = format!("{}-W{:02}", dt.format("%G"), dt.iso_week().week());
            if let Some(bucket) = buckets.iter_mut().find(|(l, _)| *l == label) {
                bucket.1 += 1;
            }
        }
        buckets
    });

    // Tip-commit timestamp per branch (docs/08 Branch view age + activity).
    let branch_tips = branches.as_ref().map(|list| {
        list.iter()
            .map(|b| {
                repo.find_commit(b.target)
                    .map(|c| c.author.time)
                    .unwrap_or(0)
            })
            .collect()
    });

    let dependencies = gitx_analysis::manifest::head_dependencies(&repo).ok();
    // Cap the object-database scan so TUI startup stays fast on large repos
    // (the CLI `gitx recovery` does the full scan on demand).
    let recovery = gitx_analysis::recovery::analyze_recovery_capped(&repo).ok();

    // Per-sub-score evidence for the Health view (docs/08 §3: never just a
    // number). Index order matches the six sub-scores.
    let health_evidence = build_health_evidence(hotspots.as_ref());

    AppData {
        stats,
        timeline,
        hotspots,
        branches,
        contributors,
        dependencies,
        recovery,
        repo_path: path,
        error: None,
        activity,
        timeline_file_counts,
        branch_tips,
        repo_state,
        health_evidence,
    }
}

/// Statistics for the Overview panel: prefer a fresh persisted index, fall
/// back to live Git analysis (docs/13 §3 sub-second startup).
fn index_stats_or_live(repo: &gitx_git::Repository) -> Option<gitx_analysis::RepoStats> {
    if let Some(s) = crate::index_backed::stats_from_index(repo).ok().flatten() {
        return Some(gitx_analysis::RepoStats {
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
        });
    }
    gitx_analysis::repository_stats(repo).ok()
}

/// Evidence lines for each of the six health sub-scores, derived from the
/// analysis (docs/10 §8 explainability). Empty when analysis is unavailable.
fn build_health_evidence(analysis: Option<&gitx_analysis::RepoAnalysis>) -> Vec<Vec<String>> {
    let Some(a) = analysis else { return Vec::new() };
    let total = a.files.len().max(1);

    let high_risk: Vec<&gitx_analysis::pipeline::FileAnalysis> = a
        .files
        .iter()
        .filter(|f| f.classification == "HIGH" || f.classification == "CRITICAL")
        .collect();
    let mut hotspots = Vec::new();
    hotspots.push(format!(
        "{} of {} files are HIGH/CRITICAL hotspots",
        high_risk.len(),
        total
    ));
    for f in high_risk.iter().take(5) {
        hotspots.push(format!("  {} (score {:.0})", f.path.display(), f.hotspot));
    }

    let mut ownership = Vec::new();
    let mut owned: Vec<&gitx_analysis::pipeline::FileAnalysis> = a.files.iter().collect();
    owned.sort_by(|x, y| {
        y.ownership_concentration
            .partial_cmp(&x.ownership_concentration)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for f in owned.iter().take(5) {
        ownership.push(format!(
            "  {:.0}% concentrated  {}",
            f.ownership_concentration,
            f.path.display()
        ));
    }

    let mut volatility = Vec::new();
    let mut churny: Vec<&gitx_analysis::pipeline::FileAnalysis> = a.files.iter().collect();
    // FileMetrics has no dedicated churn field; use total lines touched as the
    // volatility proxy (docs/10 §2 churn = insertions + deletions).
    let churn_of = |f: &gitx_analysis::pipeline::FileAnalysis| {
        f.metrics.lines_added as u64 + f.metrics.lines_deleted as u64
    };
    churny.sort_by_key(|f| std::cmp::Reverse(churn_of(f)));
    for f in churny.iter().take(5) {
        volatility.push(format!(
            "  {} lines touched  {}",
            churn_of(f),
            f.path.display()
        ));
    }

    vec![
        hotspots,
        ownership,
        vec!["Branch hygiene: share of branches with activity in the last 30 days.".to_string()],
        volatility,
        vec![
            "Architecture stability: share of current files not added in the last 30 days."
                .to_string(),
        ],
        vec!["Recovery risk: reflog presence and unreachable-commit volume.".to_string()],
    ]
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
