use crate::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

/// Repository Overview (docs/08 §3): stats, state, activity chart, top
/// hotspots, recent commits, and the health summary.
pub fn render(f: &mut Frame, area: Rect, app: &App) -> usize {
    let block = Block::default()
        .title(" Repository Overview ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let content = match &app.stats {
        None => "No repository loaded.\n\nRun gitx-tui from inside a Git repository.".to_string(),
        Some(s) => format_stats(s, app),
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White));
    f.render_widget(paragraph, area);
    0
}

fn format_stats(s: &gitx_analysis::RepoStats, app: &App) -> String {
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
    let first = s.first_commit.map(ts).unwrap_or_else(|| "-".to_string());
    let last = s.last_commit.map(ts).unwrap_or_else(|| "-".to_string());
    let state = app.repo_state.as_deref().unwrap_or("clean").to_lowercase();

    let mut out = format!(
        "HEAD      {}  {}\n\
         State     {}\n\n\
         Commits              {}\n\
         Contributors          {}\n\
         Files                 {}\n\
         Branches              {}\n\
         Tags                  {}\n\
         Repository age        {} days\n\
         First commit          {}\n\
         Last commit           {}\n",
        bold(&head),
        head_msg,
        state,
        s.commits,
        s.contributors,
        s.files,
        s.branches,
        s.tags,
        s.age_days,
        first,
        last,
    );

    // Activity chart (docs/08 §3) — last 12 weeks, oldest → newest.
    if let Some(activity) = &app.activity {
        let max = activity.iter().map(|(_, c)| *c).max().unwrap_or(0).max(1);
        let bars: String = activity
            .iter()
            .map(|(_, c)| {
                let idx = (c * 8 / max).min(7);
                "▁▂▃▄▅▆▇█".chars().nth(idx as usize).unwrap_or('▁')
            })
            .collect();
        out.push_str(&format!("\nActivity (12 weeks)\n  {bars}\n"));
    }

    // Top hotspots (docs/08 §3).
    if let Some(a) = &app.hotspots
        && !a.files.is_empty()
    {
        out.push_str("\nTop hotspots (maintenance risk)\n");
        for file in a.files.iter().take(5) {
            out.push_str(&format!(
                "  {:.0}  {:<4}  {}\n",
                file.hotspot,
                file.classification,
                file.path.display()
            ));
        }
    }

    // Recent commits (docs/08 §3).
    if let Some(timeline) = &app.timeline
        && !timeline.is_empty()
    {
        out.push_str("\nRecent commits\n");
        for c in timeline.iter().take(5) {
            out.push_str(&format!(
                "  {}  {}  {}\n",
                &c.id.to_string()[..7.min(c.id.to_string().len())],
                c.author.name,
                one_line(&c.message)
            ));
        }
    }

    // Health summary (docs/08 §3).
    if let Some(a) = &app.hotspots {
        let h = &a.health;
        out.push_str(&format!(
            "\nHealth overall {:.0}/100  (see Health view for sub-scores)\n",
            h.overall_score
        ));
    }

    if !s.languages.is_empty() {
        out.push_str("\nLanguages\n");
        for (ext, count) in s.languages.iter().take(8) {
            out.push_str(&format!("  {ext:<14} {count}\n"));
        }
    }

    out.push_str("\nPress ? for help, r to refresh, q to quit.");
    out
}

fn one_line(message: &str) -> String {
    message.lines().next().unwrap_or("").to_string()
}

fn ts(seconds: i64) -> String {
    match chrono::DateTime::from_timestamp(seconds, 0) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M").to_string(),
        None => seconds.to_string(),
    }
}

fn bold(s: &str) -> String {
    format!("\u{1b}[1m{s}\u{1b}[0m")
}

/// Architecture panel (docs/08): current modules by file count, plus modules
/// added in the last 90 days.
pub fn architecture_panel(
    f: &mut Frame,
    area: Rect,
    analysis: Option<&gitx_analysis::RepoAnalysis>,
) {
    let content = match analysis {
        None => "No repository loaded.".to_string(),
        Some(a) if a.files.is_empty() => "No files analyzed.".to_string(),
        Some(a) => {
            let mut dirs: std::collections::HashMap<String, usize> =
                std::collections::HashMap::new();
            for file in &a.files {
                let dir = file
                    .path
                    .parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| ".".into());
                *dirs.entry(dir).or_insert(0) += 1;
            }
            let mut list: Vec<(String, usize)> = dirs.into_iter().collect();
            list.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
            let mut out = format!(
                "directories: {}  files analyzed: {}\n\n",
                list.len(),
                a.files.len()
            );
            for (dir, files) in list.iter().take(25) {
                out.push_str(&format!("{:<40} {files}\n", dir));
            }
            out
        }
    };
    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .title(" Architecture (modules) ")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White));
    f.render_widget(paragraph, area);
}
