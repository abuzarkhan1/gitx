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
    /// Module graph summary (docs/21 Stage 6): per-directory file/import/call
    /// counts from the shared HEAD-graph builder.
    Graph,
    /// Commit or file detail opened from a list row (docs/08 drill-down).
    Detail,
}

/// What the detail view is showing.
#[derive(Debug, Clone)]
pub enum Detail {
    Commit { oid: String },
    File { path: std::path::PathBuf },
}

/// Progress messages from the background repository loader (docs/08 §6:
/// operation name, processed/total, cancellation). The worker reports each
/// stage as it completes; a partial dataset lands first (Overview
/// essentials, docs/13 §7 lazy loading) and the final message carries the
/// complete data.
pub enum LoadMsg {
    /// A stage completed: human-readable name + step index + total steps.
    Progress {
        phase: &'static str,
        step: usize,
        total: usize,
    },
    /// A partial dataset: the Overview essentials, sent before the heavy
    /// panels finish so the UI paints immediately. `loading` stays true.
    Phase { data: Box<AppData> },
    /// The full repository data set landed. Boxed so the tiny `Progress`
    /// variant stays small (clippy::large_enum_variant).
    Done(Box<AppData>),
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
    /// Search results: FTS hits across commits/files/authors/branches/tags
    /// (docs/11), produced by `SearchService`.
    pub search_results: Option<Vec<gitx_services::SearchHit>>,
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
    /// Ahead/behind intelligence per branch, aligned with `branches` (docs/08
    /// Branch view: ahead/behind bar; docs/10 §5).
    pub branch_intel: Option<Vec<Option<gitx_analysis::branch::BranchIntelligence>>>,
    /// Repository state string from gix (clean/merge/rebase...).
    pub repo_state: Option<String>,
    /// Per-sub-score evidence lines for the Health view (docs/08 §3: selecting
    /// a sub-score reveals its evidence).
    pub health_evidence: Vec<Vec<String>>,
    /// Hotspots sort mode (docs/08 Hotspots sortable table): 0=score,
    /// 1=change frequency, 2=churn. Toggled with `s`.
    pub hotspot_sort: u8,
    /// Row count of the current view's last render (docs/08: scroll-position
    /// indicator in the status bar — “showing x–y of N”).
    pub last_row_count: usize,
    /// Visible row count of the current view's last render (used to keep the
    /// cursor highlight inside the window while j/k move the selection).
    pub visible: usize,
    /// Changed-file path strings per timeline commit, aligned with `timeline`
    /// (docs/08 Commit view: related-commits panel).
    pub commit_files: Option<Vec<Vec<String>>>,
    /// Architecture before/after comparison (docs/08 Architecture view,
    /// docs/10 §10): HEAD vs the newest commit ≥30 days old.
    pub arch_diff: Option<ArchDiff>,
    /// Graph view rows: (directory, file count, import edges, call edges).
    pub graph_summary: Option<Vec<(String, usize, usize, usize)>>,
    /// Spinner frame while the background loader runs (docs/08 loading
    /// progress: a real animated indicator, not just text).
    pub load_frame: u8,
    /// True once the user has navigated at all — the first-run onboarding
    /// hint shows until then (docs/08 #31).
    pub nav_used: bool,
    /// True while an FTS search is running on a worker thread (docs/08: the
    /// UI never freezes on large repositories).
    pub search_pending: bool,
    /// Terminal width from the last render (docs/08 §5 responsive layout):
    /// lets the mouse handler compute the sidebar width for click targets.
    pub width: u16,
    /// Current loader stage name (docs/08 §6 loading progress), while
    /// `loading` is true: e.g. "Analyzing hotspots & health".
    pub load_phase: &'static str,
    /// Completed loader steps out of `load_total` (docs/08 §6 processed/total).
    pub load_step: usize,
    pub load_total: usize,
    /// Shared cancellation flag for the background loader (docs/08 §6): Esc
    /// sets it and the worker stops between stages; results are discarded.
    cancel_load: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// Receiver for the background data load (docs/08 loading progress): the
    /// repo data is computed on a worker thread so the UI renders immediately
    /// and the status bar shows progress while it loads.
    data_rx: Option<std::sync::mpsc::Receiver<LoadMsg>>,
    /// Receiver for async FTS search results (docs/08 Search: the query runs
    /// on a worker thread; results land here on the next tick).
    search_rx: Option<std::sync::mpsc::Receiver<Vec<gitx_services::SearchHit>>>,
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
        let (tx, rx) = std::sync::mpsc::channel::<LoadMsg>();
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        std::thread::spawn(move || {
            load_repo_stats(&tx, &worker_cancel);
        });
        Self {
            current_view: View::Overview,
            nav_index: 0,
            running: true,
            error: None,
            loading: true,
            load_phase: "Discovering repository",
            load_step: 0,
            load_total: LOAD_STAGES,
            cancel_load: cancel,
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
            timeline_areas: None,
            branch_tips: None,
            branch_intel: None,
            repo_state: None,
            health_evidence: Vec::new(),
            hotspot_sort: 0,
            last_row_count: 0,
            visible: 1,
            commit_files: None,
            arch_diff: None,
            graph_summary: None,
            load_frame: 0,
            nav_used: false,
            search_pending: false,
            width: 0,
            data_rx: Some(rx),
            search_rx: None,
        }
    }

    /// Merge a partial or final dataset into the app state (docs/13 §7 lazy
    /// loading). `apply_phase` keeps `loading` true; [`App::apply_data`]
    /// marks the load complete.
    fn merge_data(&mut self, data: AppData) {
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
        self.timeline_areas = data.timeline_areas;
        self.branch_tips = data.branch_tips;
        self.branch_intel = data.branch_intel;
        self.repo_state = data.repo_state;
        self.health_evidence = data.health_evidence;
        self.commit_files = data.commit_files;
        self.arch_diff = data.arch_diff;
        self.graph_summary = data.graph_summary;
    }

    /// Apply a partial dataset (Overview essentials). `loading` stays true
    /// until the final [`App::apply_data`] lands (docs/13 §7).
    pub fn apply_phase(&mut self, data: AppData) {
        self.merge_data(data);
    }

    /// Apply the final background-loaded repository data (docs/08 loading
    /// progress): merge everything and mark the load complete.
    pub fn apply_data(&mut self, data: AppData) {
        self.merge_data(data);
        self.loading = false;
        // Data changed: invalidate stale search results.
        self.search_results = None;
        self.search_pending = false;
    }

    /// Reload all repository data on a worker thread (docs/08: `r` refresh).
    /// The status bar shows the stage and step/total while it loads (docs/08
    /// §6), and Esc cancels the load.
    pub fn reload(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel::<LoadMsg>();
        self.cancel_load
            .store(false, std::sync::atomic::Ordering::Relaxed);
        let cancel = self.cancel_load.clone();
        std::thread::spawn(move || {
            load_repo_stats(&tx, &cancel);
        });
        self.data_rx = Some(rx);
        self.loading = true;
        self.load_phase = "Discovering repository";
        self.load_step = 0;
        self.load_total = 7;
    }

    /// Cancel the in-flight background load (docs/08 §6 cancellation hint):
    /// Esc stops the spinner and discards any results that still land.
    pub fn cancel_loading(&mut self) {
        self.cancel_load
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.loading = false;
        self.data_rx = None;
    }

    /// Poll the background loader; returns true when new data landed.
    pub fn poll_load(&mut self) -> bool {
        let Some(rx) = &self.data_rx else {
            return false;
        };
        match rx.try_recv() {
            Ok(LoadMsg::Progress { phase, step, total }) => {
                self.load_phase = phase;
                self.load_step = step;
                self.load_total = total;
                false
            }
            Ok(LoadMsg::Phase { data }) => {
                self.apply_phase(*data);
                false
            }
            Ok(LoadMsg::Done(data)) => {
                self.apply_data(*data);
                self.data_rx = None;
                true
            }
            Err(_) => false,
        }
    }

    /// Poll the async search worker; returns true when results landed.
    pub fn poll_search(&mut self) -> bool {
        let Some(rx) = &self.search_rx else {
            return false;
        };
        match rx.try_recv() {
            Ok(hits) => {
                self.search_results = Some(hits);
                self.search_pending = false;
                self.search_rx = None;
                true
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                self.search_pending = false;
                self.search_rx = None;
                true
            }
            Err(_) => false,
        }
    }

    /// Toggle the Hotspots sort mode (docs/08 sortable table).
    pub fn cycle_hotspot_sort(&mut self) {
        self.hotspot_sort = (self.hotspot_sort + 1) % 3;
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn set_view(&mut self, view: View) {
        self.current_view = view;
    }

    pub fn next_nav(&mut self) {
        if self.nav_index < 14 {
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
            14 => View::Graph,
            _ => View::Overview,
        };
        // Enter opens the view: subsequent j/k scroll its content.
        self.in_content = true;
        self.scroll = 0;
        self.selected = 0;
        self.nav_used = true;
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
            View::Graph => 14,
            View::Detail => 13,
        };
    }

    /// Move the cursor down one row. The selection is what Enter opens and
    /// the status bar reports (docs/08 #20), so j/k move it; the window only
    /// scrolls when the cursor would leave the visible area.
    pub fn cursor_down(&mut self) {
        let max = self.last_row_count.saturating_sub(1);
        if self.selected < max {
            self.selected += 1;
            let visible = self.visible.max(1);
            if self.selected >= self.scroll + visible {
                self.scroll = self.selected.saturating_add(1).saturating_sub(visible);
            }
        }
    }

    /// Move the cursor up one row (see [`App::cursor_down`]).
    pub fn cursor_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        if self.selected < self.scroll {
            self.scroll = self.selected;
        }
    }

    /// Scroll the current view down by one row without moving the cursor
    /// (mouse wheel).
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
                list.get(self.selected)
                    .and_then(|hit| match hit.scope.as_str() {
                        "commit" => Some(Detail::Commit {
                            oid: hit.id.clone(),
                        }),
                        // File, code and symbol hits all resolve to a file
                        // path (docs/08 Search: Enter opens the result).
                        "file" | "code" | "symbol" => Some(Detail::File {
                            path: std::path::PathBuf::from(&hit.id),
                        }),
                        _ => None,
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
        self.nav_used = true;
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
            Detail::Commit { oid } => render_commit_detail(
                &repo,
                oid,
                self.timeline.as_deref(),
                self.commit_files.as_deref(),
            ),
            Detail::File { path } => render_file_detail(&repo, path, self.hotspots.as_ref()),
        }
    }

    /// Run an FTS search through `SearchService` across commits, files,
    /// authors, branches and tags (docs/08 Search, docs/11). Runs on a worker
    /// thread so a big repository never freezes the UI (docs/08 #20); results
    /// land via [`App::poll_search`] on the next tick.
    pub fn run_search(&mut self) {
        let raw = self.search_query.trim();
        if raw.is_empty() {
            self.search_results = None;
            self.search_pending = false;
            self.search_rx = None;
            return;
        }
        let repo_path = match &self.repo_path {
            Some(path) => path.clone(),
            None => {
                self.search_results = Some(Vec::new());
                self.search_pending = false;
                self.search_rx = None;
                return;
            }
        };
        // FTS5-safe phrase query: quote the input (doubling embedded quotes).
        let query = format!("\"{}\"", raw.replace('"', "\"\""));
        // Owned copy for the worker thread (the code-content scope needs the
        // raw term, not the FTS-escaped phrase).
        let raw_owned = raw.to_string();
        let (tx, rx) = std::sync::mpsc::channel();
        self.search_rx = Some(rx);
        self.search_pending = true;
        std::thread::spawn(move || {
            let repo = gitx_git::Repository::discover(&repo_path);
            let hits = repo
                .ok()
                .map(|r| {
                    let service = gitx_services::SearchService::new(&r);
                    let options = gitx_services::SearchOptions {
                        commits: true,
                        files: true,
                        authors: true,
                        branches: true,
                        tags: true,
                        renames: true,
                        symbols: true,
                        directories: true,
                        code: true,
                        ..Default::default()
                    };
                    service
                        .search(&query, &raw_owned, &options)
                        .unwrap_or_default()
                })
                .unwrap_or_default();
            let _ = tx.send(hits);
        });
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
    /// Top directories the author works in, most-touched first (docs/08
    /// Contributors: areas / ownership concentration). Empty when unknown.
    pub areas: Vec<String>,
}

/// Architecture before/after comparison (docs/08 Architecture view): the
/// structural diff between HEAD and the newest commit ≥30 days old.
#[derive(Debug, Clone)]
pub struct ArchDiff {
    pub from: String,
    pub to: String,
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
    pub added_files: Vec<String>,
    pub removed_files: Vec<String>,
    /// Directories that gained at least one file (docs/10 §10 modules added).
    pub modules_added: Vec<String>,
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
    pub branch_intel: Option<Vec<Option<gitx_analysis::branch::BranchIntelligence>>>,
    pub repo_state: Option<String>,
    pub health_evidence: Vec<Vec<String>>,
    pub commit_files: Option<Vec<Vec<String>>>,
    pub arch_diff: Option<ArchDiff>,
    /// Graph view rows: (directory, file count, import edges, call edges).
    pub graph_summary: Option<Vec<(String, usize, usize, usize)>>,
}

/// Total loader stages reported by [`load_repo_stats`] (docs/08 §6
/// processed/total progress in the status bar).
const LOAD_STAGES: usize = 7;

/// Load repository data eagerly at startup by discovering the repository from
/// the current directory. Blocks briefly; acceptable for V1 (docs/08 notes the
/// overview is the first-render anchor). Reports each stage on `tx` so the
/// status bar can show operation + step/total (docs/08 §6) and stops between
/// stages when `cancel` is set (Esc).
fn load_repo_stats(tx: &std::sync::mpsc::Sender<LoadMsg>, cancel: &std::sync::atomic::AtomicBool) {
    use std::sync::atomic::Ordering;

    // Report a completed stage; returns true when the load was cancelled.
    let report = |tx: &std::sync::mpsc::Sender<LoadMsg>,
                  cancel: &std::sync::atomic::AtomicBool,
                  step: usize,
                  phase: &'static str|
     -> bool {
        if cancel.load(Ordering::Relaxed) {
            return true;
        }
        let _ = tx.send(LoadMsg::Progress {
            phase,
            step,
            total: LOAD_STAGES,
        });
        false
    };

    let repo = match gitx_git::Repository::discover(".") {
        Ok(repo) => repo,
        Err(err) => {
            let _ = tx.send(LoadMsg::Done(Box::new(AppData {
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
                timeline_areas: None,
                branch_tips: None,
                branch_intel: None,
                repo_state: None,
                health_evidence: Vec::new(),
                commit_files: None,
                arch_diff: None,
                graph_summary: None,
            })));
            return;
        }
    };
    let path = repo.work_dir().map(|p| p.display().to_string());
    let repo_state = repo.state();

    // ======================================================================
    // Phase A — Overview essentials (docs/13 §7 lazy loading): stats,
    // timeline, per-commit areas and activity land first so the Overview
    // paints immediately; the heavy panels fill in during Phase B.
    // ======================================================================
    // Services layer (docs/04 §6): statistics and analysis go through
    // `RepositoryService`/`AnalysisService`, which prefer the fresh persisted
    // index and fall back to live Git computation (docs/13 §3).
    if report(tx, cancel, 1, "Repository discovered") {
        return;
    }
    let stats = index_stats_or_live(&repo);
    if report(tx, cancel, 2, "Building timeline") {
        return;
    }
    let service = gitx_history::timeline::HistoryService::new(&repo);
    let timeline = service
        .timeline(gitx_history::timeline::TimelineOptions {
            max_count: Some(500),
            ..Default::default()
        })
        .ok();

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
                let changes = repo
                    .diff_tree_to_tree(parent_tree, c.tree_id)
                    .unwrap_or_default();
                let mut dirs: std::collections::HashMap<String, u32> =
                    std::collections::HashMap::new();
                let mut files: Vec<String> = Vec::new();
                for change in &changes {
                    files.push(change.path.display().to_string());
                    let dir = change
                        .path
                        .parent()
                        .filter(|p| !p.as_os_str().is_empty())
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| ".".to_string());
                    *dirs.entry(dir).or_insert(0) += 1;
                }
                let top = dirs.into_iter().max_by_key(|(_, n)| *n).map(|(d, _)| d);
                (changes.len() as u32, top.unwrap_or_default(), files)
            })
            .collect::<Vec<(u32, String, Vec<String>)>>()
    });
    let timeline_file_counts = per_commit_areas
        .as_ref()
        .map(|v| v.iter().map(|(n, _, _)| *n).collect());
    let timeline_areas = per_commit_areas
        .as_ref()
        .map(|v| v.iter().map(|(_, a, _)| a.clone()).collect());
    // Changed-file sets per commit (docs/08 Commit view related-commits).
    let commit_files = per_commit_areas
        .as_ref()
        .map(|v| v.iter().map(|(_, _, f)| f.clone()).collect());

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

    // Phase A lands now: the Overview, Timeline and Commits views render
    // immediately; `loading` stays true until Phase B completes. The Phase
    // payload carries clones so the originals survive for the final Done
    // (which must contain the complete dataset, never None placeholders).
    let _ = tx.send(LoadMsg::Phase {
        data: Box::new(AppData {
            stats: stats.clone(),
            timeline: timeline.clone(),
            hotspots: None,
            branches: None,
            contributors: None,
            dependencies: None,
            recovery: None,
            repo_path: path.clone(),
            error: None,
            activity: activity.clone(),
            timeline_file_counts: timeline_file_counts.clone(),
            timeline_areas: timeline_areas.clone(),
            branch_tips: None,
            branch_intel: None,
            repo_state: repo_state.clone(),
            health_evidence: Vec::new(),
            commit_files: commit_files.clone(),
            arch_diff: None,
            graph_summary: None,
        }),
    });

    // ======================================================================
    // Phase B — heavy panels (docs/13 §7): hotspots/health, branches,
    // contributors, architecture, dependencies, recovery and the graph
    // summary. Each stage still honors Esc-cancel between stages.
    // ======================================================================
    if report(tx, cancel, 3, "Analyzing hotspots & health") {
        return;
    }
    // Index-backed analysis via AnalysisService (docs/04 §6, docs/13 §3):
    // fresh cache → read; otherwise live computation.
    let hotspots = gitx_services::AnalysisService::new(&repo)
        .analyze(true, gitx_analysis::hotspots::HotspotWeights::default())
        .ok();
    let branches = repo.branches().ok();

    // Contributors from the timeline: commit count + first/last activity +
    // files touched + top areas (from live analysis author_lines, keyed by
    // the raw author name; empty from the cache).
    // Author identity → files they added lines to (keyed like the pipeline's
    // author_lines: `Name <email>`). Used for files_touched + top areas.
    let author_files = hotspots.as_ref().map(|a| {
        let mut map: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for file in &a.files {
            for author in file.author_lines.keys() {
                map.entry(author.clone())
                    .or_default()
                    .push(file.path.display().to_string());
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
                areas: Vec::new(),
            });
            entry.commits += 1;
            entry.first_activity = entry.first_activity.min(c.author.time);
            entry.last_activity = entry.last_activity.max(c.author.time);
            // Match on the full identity — author_lines is keyed by
            // `Name <email>` (docs/05 identity normalization).
            let identity = format!("{} <{}>", c.author.name, c.author.email);
            if let Some(map) = &author_files
                && let Some(files) = map.get(&identity)
            {
                entry.files_touched = files.len() as u64;
                // Top areas: directories the author touches most.
                let mut dirs: std::collections::HashMap<String, u32> =
                    std::collections::HashMap::new();
                for f in files {
                    let dir = std::path::Path::new(f)
                        .parent()
                        .filter(|p| !p.as_os_str().is_empty())
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| ".".into());
                    *dirs.entry(dir).or_insert(0) += 1;
                }
                let mut areas: Vec<(String, u32)> = dirs.into_iter().collect();
                areas.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
                entry.areas = areas.into_iter().take(3).map(|(d, _)| d).collect();
            }
        }
        let mut list: Vec<Contributor> = counts.into_values().collect();
        list.sort_by_key(|c| std::cmp::Reverse(c.commits));
        list
    });

    if report(tx, cancel, 4, "Reading branches & contributors") {
        return;
    }
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

    // Ahead/behind intelligence vs the current branch (docs/08 Branch view
    // ahead/behind bar; docs/10 §5). Local branches only, so remote refs
    // (which don't compare cleanly) keep a None entry.
    let branch_intel = branches.as_ref().map(|list| {
        let current = repo.head_commit_id().ok().and_then(|id| {
            list.iter()
                .find(|b| !b.is_remote && b.target == id)
                .cloned()
        });
        list.iter()
            .map(|b| {
                if b.is_remote {
                    None
                } else {
                    gitx_analysis::branch::branch_intelligence(&repo, b, current.as_ref())
                        .ok()
                        .flatten()
                }
            })
            .collect()
    });

    if report(tx, cancel, 5, "Architecture & activity") {
        return;
    }
    // Architecture before/after (docs/08 Architecture view, docs/10 §10):
    // HEAD vs the newest commit ≥30 days old (falling back to the oldest
    // commit in the window).
    let arch_diff = compute_arch_diff(&repo, timeline.as_deref()).ok().flatten();
    // Module graph summary (docs/21 Stage 6): per-directory file/import/call
    // counts for the Graph view, computed once here and cached.
    let graph_summary = gitx_graph::graph::module_summary(&repo).ok();

    if report(tx, cancel, 6, "Dependencies & recovery") {
        return;
    }
    let dependencies = gitx_analysis::manifest::head_dependencies(&repo).ok();
    // Cap the object-database scan so TUI startup stays fast on large repos
    // (the CLI `gitx recovery` does the full scan on demand).
    let recovery = gitx_analysis::recovery::analyze_recovery_capped(&repo).ok();

    if report(tx, cancel, 7, "Finishing") {
        return;
    }
    // Per-sub-score evidence for the Health view (docs/08 §3: never just a
    // number). Index order matches the six sub-scores.
    let health_evidence = build_health_evidence(hotspots.as_ref());

    // The final message carries the complete dataset: the Phase A values
    // (kept alive by the clones in the Phase send) plus the Phase B panels.
    let _ = tx.send(LoadMsg::Done(Box::new(AppData {
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
        timeline_areas,
        branch_tips,
        branch_intel,
        repo_state,
        health_evidence,
        commit_files,
        arch_diff,
        graph_summary,
    })));
}

/// Structural before/after comparison (docs/08 Architecture view): snapshot
/// the HEAD tree and the newest commit ≥30 days old, then diff them with
/// `gitx_graph::compare`. Returns None when there is nothing to compare.
fn compute_arch_diff(
    repo: &gitx_git::Repository,
    timeline: Option<&[gitx_git::models::Commit]>,
) -> anyhow::Result<Option<ArchDiff>> {
    let head_id = repo.head_commit_id()?;
    let head = repo.find_commit(head_id)?;
    let cutoff = chrono::Utc::now().timestamp() - 30 * 86_400;
    let from_commit = timeline
        .and_then(|list| {
            // Newest-first: first commit at least 30 days old, else the oldest.
            list.iter()
                .find(|c| c.author.time <= cutoff)
                .or_else(|| list.last())
                .cloned()
        })
        .filter(|c| c.id != head.id);
    let Some(from) = from_commit else {
        return Ok(None);
    };

    let old = snapshot_from_tree(repo, from.tree_id, &format!("{}", from.id))?;
    let new = snapshot_from_tree(repo, head.tree_id, &format!("{}", head.id))?;
    let diff = gitx_graph::compare::compare_snapshots(&old, &new);

    let added_files: Vec<String> = diff.added.iter().map(|p| p.display().to_string()).collect();
    let removed_files: Vec<String> = diff
        .removed
        .iter()
        .map(|p| p.display().to_string())
        .collect();
    let mut modules: std::collections::HashSet<String> = std::collections::HashSet::new();
    for path in &diff.added {
        if let Some(dir) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            modules.insert(dir.display().to_string());
        }
    }
    let mut modules_added: Vec<String> = modules.into_iter().collect();
    modules_added.sort();

    Ok(Some(ArchDiff {
        from: format!(
            "{} ({})",
            from.id.to_string().chars().take(7).collect::<String>(),
            ts(from.author.time)
        ),
        to: format!(
            "{} ({})",
            head.id.to_string().chars().take(7).collect::<String>(),
            ts(head.author.time)
        ),
        added: diff.added.len(),
        removed: diff.removed.len(),
        modified: diff.modified.len(),
        added_files,
        removed_files,
        modules_added,
    }))
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

/// Render a commit's full detail: metadata, classification, message, changed
/// files with diff stats, related history, and affected contributors
/// (mirrors `gitx commit <oid>`, docs/07 §6, docs/08 §3).
fn render_commit_detail(
    repo: &gitx_git::Repository,
    oid: &str,
    timeline: Option<&[gitx_git::models::Commit]>,
    commit_files: Option<&[Vec<String>]>,
) -> Vec<String> {
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
    // Classification (docs/07 §6, docs/10 §7 — heuristic, labeled as such).
    use gitx_core::types::CommitClassification as CC;
    let class = match gitx_analysis::classify_commit_message(&commit.message) {
        CC::Feature => "feature",
        CC::Fix => "fix",
        CC::Refactor => "refactor",
        CC::Docs => "docs",
        CC::Test => "test",
        CC::Chore => "chore",
        CC::Revert => "revert",
        CC::Merge => "merge",
        CC::Unknown => "unknown",
    };
    out.push(format!("Classification: {class} (heuristic)"));
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

    // Related history + affected contributors (docs/08 §3, docs/07 §6):
    // computed from the per-commit changed-file sets loaded with the
    // timeline, when available.
    if let Some((related, contributors)) = related_and_contributors(&commit, timeline, commit_files)
    {
        if !related.is_empty() {
            out.push(String::new());
            out.push(" Related history (commits touching the same files):".to_string());
            for (short_oid, overlap) in related.iter().take(6) {
                out.push(format!(
                    "  {short_oid}  ({} shared file{})",
                    overlap,
                    if *overlap == 1 { "" } else { "s" }
                ));
            }
        }
        if !contributors.is_empty() {
            out.push(String::new());
            out.push(" Affected contributors:".to_string());
            for name in contributors.iter().take(6) {
                out.push(format!("  {name}"));
            }
        }
    }
    out
}

/// (related commits with shared-file counts, contributor names) — the
/// commit-detail enrichment result (docs/07 §6, docs/08 §3).
type RelatedAndContributors = (Vec<(String, usize)>, Vec<String>);

/// Related commits (by shared-file overlap) and their authors, from the
/// precomputed per-commit changed-file sets. `None` when the sets are
/// unavailable (cache path).
fn related_and_contributors(
    commit: &gitx_git::models::Commit,
    timeline: Option<&[gitx_git::models::Commit]>,
    commit_files: Option<&[Vec<String>]>,
) -> Option<RelatedAndContributors> {
    let files = commit_files?;
    let timeline = timeline?;
    let index = timeline.iter().position(|c| c.id == commit.id)?;
    let selected = files.get(index)?;
    if selected.is_empty() {
        return Some((Vec::new(), Vec::new()));
    }
    let selected_set: std::collections::HashSet<&String> = selected.iter().collect();
    let mut related: Vec<(String, usize)> = Vec::new();
    let mut contributors: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (i, c) in timeline.iter().enumerate() {
        if i == index {
            continue;
        }
        let Some(other) = files.get(i) else { continue };
        let overlap = other.iter().filter(|f| selected_set.contains(f)).count();
        if overlap > 0 {
            related.push((short(&c.id), overlap));
            let key = format!("{} <{}>", c.author.name, c.author.email);
            if seen.insert(key) {
                contributors.push(c.author.name.clone());
            }
        }
    }
    related.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    Some((related.into_iter().take(6).collect(), contributors))
}

/// Render a file's lineage (docs/10 file archaeology, docs/08 File view):
/// creation commit, first/last change, every rename event, and each commit
/// that touched it — following renames backward from HEAD. Mirrors
/// `gitx lineage <path>`.
fn render_file_detail(
    repo: &gitx_git::Repository,
    path: &std::path::Path,
    hotspots: Option<&gitx_analysis::RepoAnalysis>,
) -> Vec<String> {
    use gitx_history::lineage::{FileAction, FileLineageNode};

    let service = gitx_history::timeline::HistoryService::new(repo);
    let lineage = match service.get_file_lineage(path.to_path_buf(), None) {
        Ok(l) => l,
        Err(e) => return vec![format!("cannot trace lineage of {}: {e}", path.display())],
    };
    let mut out = vec![format!("Lineage of {}", path.display())];
    if lineage.history.is_empty() {
        out.push("  (no commits touch this path)".to_string());
        return out;
    }

    // First (creation) and last (most recent) nodes, newest-first list.
    let first = lineage.history.last().unwrap();
    let last = lineage.history.first().unwrap();
    out.push(format!(
        "  Created by {} on {} — last change by {} on {}",
        author_of(repo, &first.commit_id),
        ts(commit_time(repo, &first.commit_id)),
        author_of(repo, &last.commit_id),
        ts(commit_time(repo, &last.commit_id))
    ));
    out.push(format!(
        "  {} commit(s) in the file's life{}",
        lineage.history.len(),
        if lineage.history.len() > 1 {
            " (newest first)".to_string()
        } else {
            String::new()
        }
    ));

    // Contributors + churn (docs/08 File view): distinct authors across the
    // lineage and the total insertions/deletions those commits made.
    let mut authors: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut ins: u32 = 0;
    let mut del: u32 = 0;
    for node in &lineage.history {
        if let Ok(c) = repo.find_commit(node.commit_id) {
            let key = format!("{} <{}>", c.author.name, c.author.email);
            if seen.insert(key) {
                authors.push(c.author.name.clone());
            }
            let parent_tree = match c.parents.first() {
                Some(parent) => repo.find_commit(*parent).ok().map(|p| p.tree_id),
                None => None,
            };
            if let Ok(changes) = repo.diff_tree_to_tree(parent_tree, c.tree_id) {
                ins += changes.iter().map(|ch| ch.insertions).sum::<u32>();
                del += changes.iter().map(|ch| ch.deletions).sum::<u32>();
            }
        }
    }
    authors.sort();
    out.push(format!(
        "  Contributors ({})  ·  churn: {ins} insertions, {del} deletions across {} commit(s)",
        authors.len(),
        lineage.history.len()
    ));
    if !authors.is_empty() {
        out.push(format!("    {}", authors.join(", ")));
    }

    // Hotspot metrics (docs/08 File view) from the live analysis, when the
    // path is present in it.
    if let Some(a) = hotspots {
        let canonical = path.to_string_lossy().replace('\\', "/");
        if let Some(f) = a
            .files
            .iter()
            .find(|f| f.path.to_string_lossy().replace('\\', "/") == canonical)
        {
            let churn = f.metrics.lines_added + f.metrics.lines_deleted;
            out.push(format!(
                "  Hotspot: {:.0}/100 ({})  ·  {} changes  ·  {} churn  ·  {} author(s)  ·  {:.0}% owned",
                f.hotspot,
                severity_label(f.hotspot),
                f.metrics.change_frequency,
                churn,
                f.metrics.unique_contributors,
                f.ownership_concentration
            ));
        }
    }
    out.push(String::new());

    let badge = |action: &FileAction| -> String {
        match action {
            FileAction::Added { copy_of: Some(src) } => format!("COPIED from {}", src.display()),
            FileAction::Added { copy_of: None } => "ADDED   ".to_string(),
            FileAction::Modified => "MODIFIED".to_string(),
            FileAction::Deleted => "DELETED ".to_string(),
            FileAction::Renamed { .. } => "RENAMED ".to_string(),
        }
    };
    let describe = |n: &FileLineageNode| -> String {
        match &n.action {
            FileAction::Renamed { from } => format!(
                "{}  {}  {}  renamed from {}",
                badge(&n.action),
                short(&n.commit_id),
                ts(commit_time(repo, &n.commit_id)),
                from.display()
            ),
            _ => format!(
                "{}  {}  {}  {}",
                badge(&n.action),
                short(&n.commit_id),
                ts(commit_time(repo, &n.commit_id)),
                one_line(&commit_message(repo, &n.commit_id))
            ),
        }
    };
    for node in &lineage.history {
        out.push(format!("  {}", describe(node)));
    }
    out
}

fn commit_time(repo: &gitx_git::Repository, id: &gitx_git::models::ObjectId) -> i64 {
    repo.find_commit(*id).map(|c| c.author.time).unwrap_or(0)
}

fn commit_message(repo: &gitx_git::Repository, id: &gitx_git::models::ObjectId) -> String {
    repo.find_commit(*id).map(|c| c.message).unwrap_or_default()
}

fn author_of(repo: &gitx_git::Repository, id: &gitx_git::models::ObjectId) -> String {
    repo.find_commit(*id)
        .map(|c| c.author.name)
        .unwrap_or_else(|_| "-".into())
}

/// Risk band label for a hotspot score (docs/10 §2: 0–30 low, 31–60 medium,
/// 61–80 high, 81–100 critical).
fn severity_label(score: f64) -> &'static str {
    if score >= 80.0 {
        "CRITICAL"
    } else if score >= 60.0 {
        "HIGH"
    } else if score >= 30.0 {
        "MEDIUM"
    } else {
        "LOW"
    }
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
