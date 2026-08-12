use crate::app::App;
use crate::views::{common, theme};
use ratatui::{
    Frame,
    layout::Rect,
    style::Color,
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Repository Overview (docs/08 §3): stats with context, activity chart,
/// language breakdown, health gauges, top hotspots, recent commits, and a
/// repository-size gauge — with one-line explanations for every number.
pub fn render(f: &mut Frame, area: Rect, app: &App) -> usize {
    let content: Vec<Line<'static>> = match &app.stats {
        None => {
            if app.loading {
                vec![
                    theme::plain("Loading repository data…"),
                    theme::dim("The Overview appears as soon as the index is read."),
                ]
            } else {
                vec![
                    theme::plain("No repository loaded."),
                    theme::dim("Run gitx-tui from inside a Git repository."),
                ]
            }
        }
        Some(s) => format_stats(s, app),
    };
    let rows: Vec<Line<'static>> = content;
    common::render_scrollable(
        f,
        area,
        " Repository Overview — your repository at a glance ",
        &rows,
        app.scroll,
        app.selected,
    )
}

fn format_stats(s: &gitx_analysis::RepoStats, app: &App) -> Vec<Line<'static>> {
    let head = s
        .head_oid
        .as_ref()
        .map(|oid| oid.chars().take(7).collect::<String>())
        .unwrap_or_else(|| "-".to_string());
    let head_msg = s
        .head_message
        .as_ref()
        .map(|m| m.chars().take(60).collect::<String>())
        .unwrap_or_else(|| "-".to_string());
    let first = s
        .first_commit
        .map(common::ts)
        .unwrap_or_else(|| "-".to_string());
    let last = s
        .last_commit
        .map(common::ts)
        .unwrap_or_else(|| "-".to_string());
    let state = app.repo_state.as_deref().unwrap_or("clean").to_lowercase();
    let state_color = if state == "clean" {
        Color::Green
    } else {
        Color::Yellow
    };

    let mut out: Vec<Line<'static>> = Vec::new();

    // ── First-run onboarding hint (docs/08 #31): shown until the user
    //    navigates once, so a new user knows how to move around. ───────
    if !app.nav_used {
        out.push(theme::strong(
            "Getting started — ↑↓ navigate · Enter open a view · / search · ? help",
            theme::global().accent,
        ));
        out.push(theme::dim(
            "  This hint disappears after your first navigation.",
        ));
        out.push(Line::default());
    }

    // ── Header + numbers (with plain-language context) ──────────────
    out.push(theme::kv(
        "HEAD",
        format!("{head}  {head_msg}"),
        theme::global().accent,
    ));
    out.push(theme::kv("State", state, state_color));
    out.push(theme::dim(
        "  The current commit, and whether the working tree is clean.",
    ));
    out.push(Line::default());

    out.push(theme::kv(
        "Commits",
        format!("{}", s.commits),
        theme::severity_color((s.commits as f64 / 10_000.0).min(100.0)),
    ));
    out.push(theme::kv(
        "Contributors",
        format!("{}", s.contributors),
        Color::Cyan,
    ));
    out.push(theme::kv("Files", format!("{}", s.files), Color::Cyan));
    out.push(theme::kv(
        "Branches",
        format!("{}", s.branches),
        Color::Cyan,
    ));
    out.push(theme::kv("Tags", format!("{}", s.tags), Color::Cyan));
    out.push(theme::kv(
        "Repository age",
        format!("{} days", s.age_days),
        Color::Cyan,
    ));
    out.push(theme::dim(format!(
        "  {}\u{2192} {}  — how much history exists (commits) and how many people/files/branches it has.",
        first, last
    )));
    out.push(Line::default());

    // ── Repository size gauge (docs/08: no bare numbers) ────────────
    // Log scale: a 4-file repo and a 4000-file repo both get a meaningful
    // bar (a linear 0–5000 scale would render small repos at ~0%).
    let files = s.files.max(1) as f64;
    let size_pct = (((files + 1.0).log10() / 5_001.0_f64.log10()) * 100.0).min(100.0);
    let size_label = if files < 100.0 {
        "small (<100 files)"
    } else if files < 1_000.0 {
        "medium (100–1k files)"
    } else {
        "large (>1k files)"
    };
    out.push(theme::heading("Repository size"));
    out.push(theme::hbar(
        format!("{size_label}  ({} files)", s.files),
        size_pct,
        30,
        theme::health_color(size_pct),
    ));
    out.push(Line::default());

    // ── Activity chart (docs/08 §3): real bar chart with week labels ─
    if let Some(activity) = &app.activity {
        out.push(theme::heading(
            "Activity — commits per week (last 12 weeks)",
        ));
        out.extend(theme::vchart(activity, 6));
        out.push(theme::dim(
            "  Taller bars = busier weeks (labels are the last digit of each week).",
        ));
        out.push(Line::default());
    }

    // ── Language breakdown: horizontal bars ─────────────────────────
    if !s.languages.is_empty() {
        let total: usize = s.languages.iter().map(|(_, v)| v).sum::<usize>().max(1);
        let mut langs: Vec<&(String, usize)> = s.languages.iter().collect();
        langs.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        out.push(theme::heading("Languages — share of tracked files"));
        for (ext, count) in langs.iter().take(6) {
            let pct = *count as f64 / total as f64 * 100.0;
            out.push(theme::hbar(
                format!("{ext}  ({count})"),
                pct,
                24,
                Color::Cyan,
            ));
        }
        out.push(Line::default());
    }

    // ── Health gauges (docs/08: sub-scores, never just one number) ──
    if let Some(a) = &app.hotspots {
        let h = &a.health;
        out.push(theme::heading(
            "Health — six measured signals (0 = bad, 100 = good)",
        ));
        out.push(theme::hbar(
            "Code hotspots".into(),
            h.code_hotspots_score,
            24,
            theme::health_color(h.code_hotspots_score),
        ));
        out.push(theme::hbar(
            "Ownership risk".into(),
            h.ownership_risk_score,
            24,
            theme::health_color(h.ownership_risk_score),
        ));
        out.push(theme::hbar(
            "Branch hygiene".into(),
            h.branch_hygiene_score,
            24,
            theme::health_color(h.branch_hygiene_score),
        ));
        out.push(theme::hbar(
            "Change volatility".into(),
            h.change_volatility_score,
            24,
            theme::health_color(h.change_volatility_score),
        ));
        out.push(theme::hbar(
            "Architecture stability".into(),
            h.architecture_stability_score,
            24,
            theme::health_color(h.architecture_stability_score),
        ));
        out.push(theme::hbar(
            "Recovery risk".into(),
            h.recovery_risk_score,
            24,
            theme::health_color(h.recovery_risk_score),
        ));
        out.push(theme::hbar(
            format!("Overall  {:.0}/100", h.overall_score),
            h.overall_score,
            24,
            theme::health_color(h.overall_score),
        ));
        // Plain-language verdict (docs/25: never just a number).
        let verdict = if h.overall_score >= 70.0 {
            "Your repository is mostly healthy — a few files may need attention."
        } else if h.overall_score >= 40.0 {
            "Your repository has mixed signals — worth reviewing the red areas."
        } else {
            "Your repository needs attention — several signals are weak."
        };
        out.push(theme::dim(format!(
            "  {verdict} Open the Health view (e) for the evidence behind each score."
        )));
        out.push(Line::default());
    }

    // ── Top hotspots with score bars (docs/08 §3) ───────────────────
    if let Some(a) = &app.hotspots
        && !a.files.is_empty()
    {
        let mut top: Vec<&gitx_analysis::FileAnalysis> = a.files.iter().collect();
        top.sort_by(|x, y| {
            y.hotspot
                .partial_cmp(&x.hotspot)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        out.push(theme::heading(
            "Top hotspots — files that change most (maintenance risk)",
        ));
        for file in top.iter().take(5) {
            out.push(theme::hbar(
                file.path.display().to_string(),
                file.hotspot,
                24,
                theme::severity_color(file.hotspot),
            ));
        }
        out.push(theme::dim(
            "  Higher = more churn, more contributors, more bug-fixes.",
        ));
        out.push(Line::default());
    }

    // ── Contributors: relative share (docs/08 §3) ───────────────────
    if let Some(list) = &app.contributors
        && !list.is_empty()
    {
        let total: u64 = list.iter().map(|c| c.commits).sum::<u64>().max(1);
        out.push(theme::heading("Contributors — share of commits"));
        for c in list.iter().take(5) {
            let pct = c.commits as f64 / total as f64 * 100.0;
            out.push(theme::hbar(
                format!("{}  ({} commits)", c.name, c.commits),
                pct,
                24,
                Color::Magenta,
            ));
        }
        out.push(Line::default());
    }

    // ── Recent commits (docs/08 §3) ─────────────────────────────────
    if let Some(timeline) = &app.timeline
        && !timeline.is_empty()
    {
        out.push(theme::heading("Recent commits"));
        for c in timeline.iter().take(5) {
            out.push(theme::plain(format!(
                "  {}  {}  {}",
                &c.id.to_string()[..7.min(c.id.to_string().len())],
                c.author.name,
                common::one_line(&c.message, 60)
            )));
        }
        out.push(Line::default());
    }

    out.push(theme::dim(
        "Press ? for help, r to refresh, o/t/c/b/f/u/s/w/a/d/x/e/v to jump to a view, q to quit.",
    ));
    out
}

/// Architecture panel (docs/08): current modules by file count + activity,
/// with a per-module size bar and files added in the last 90 days.
pub fn architecture_panel(
    f: &mut Frame,
    area: Rect,
    analysis: Option<&gitx_analysis::RepoAnalysis>,
) {
    let content = match analysis {
        None => "No repository loaded.".to_string(),
        Some(a) if a.files.is_empty() => "No files analyzed.".to_string(),
        Some(a) => {
            let cutoff = chrono::Utc::now().timestamp() - 90 * 86_400;
            let mut dirs: std::collections::HashMap<String, (usize, u64, usize)> =
                std::collections::HashMap::new();
            for file in &a.files {
                let dir = file
                    .path
                    .parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| ".".into());
                let entry = dirs.entry(dir).or_insert((0, 0, 0));
                entry.0 += 1;
                entry.1 += file.metrics.lines_added as u64 + file.metrics.lines_deleted as u64;
                if file
                    .metrics
                    .first_introduced
                    .map(|t| t.timestamp() >= cutoff)
                    .unwrap_or(false)
                {
                    entry.2 += 1;
                }
            }
            let mut list: Vec<(String, usize, u64, usize)> = dirs
                .into_iter()
                .map(|(dir, (files, churn, recent))| (dir, files, churn, recent))
                .collect();
            list.sort_by_key(|(_, files, _, _)| std::cmp::Reverse(*files));
            let max_files = list
                .iter()
                .map(|(_, files, _, _)| *files)
                .max()
                .unwrap_or(1)
                .max(1);
            let mut out = format!(
                "directories: {}  files analyzed: {}\n\n",
                list.len(),
                a.files.len()
            );
            for (dir, files, churn, recent) in list.iter().take(25) {
                let width = 20;
                let filled = (files * width).div_ceil(max_files);
                let bar: String = "█".repeat(filled) + &"░".repeat(width - filled);
                let recent_mark = if *recent > 0 {
                    format!("  \u{1b}[36m+{recent} new(90d)\u{1b}[0m")
                } else {
                    String::new()
                };
                out.push_str(&format!(
                    "{dir:<36} {bar} {files:>3} files {churn:>7} churn{recent_mark}\n"
                ));
            }
            out
        }
    };
    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .title(" Architecture (modules — size + churn + new files) ")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false })
        .style(ratatui::style::Style::default().fg(theme::global().fg));
    f.render_widget(paragraph, area);
}
