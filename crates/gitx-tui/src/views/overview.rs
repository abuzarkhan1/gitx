use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn render(f: &mut Frame, area: Rect, stats: Option<&gitx_analysis::RepoStats>) {
    let block = Block::default()
        .title(" Repository Overview ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let content = match stats {
        None => "No repository loaded.\n\nRun gitx-tui from inside a Git repository.".to_string(),
        Some(s) => format_stats(s),
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White));
    f.render_widget(paragraph, area);
}

fn format_stats(s: &gitx_analysis::RepoStats) -> String {
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

    let mut out = format!(
        "HEAD      {}  {}\n\n\
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
        s.commits,
        s.contributors,
        s.files,
        s.branches,
        s.tags,
        s.age_days,
        first,
        last,
    );

    if !s.languages.is_empty() {
        out.push_str("\nLanguages\n");
        for (ext, count) in s.languages.iter().take(8) {
            out.push_str(&format!("  {ext:<14} {count}\n"));
        }
    }

    out.push_str("\nPress ? for help, q to quit.");
    out
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

/// Health panel (docs/08): the six deterministic sub-scores + overall.
pub fn health_panel(f: &mut Frame, area: Rect, analysis: Option<&gitx_analysis::RepoAnalysis>) {
    let content = match analysis {
        None => "No repository loaded.".to_string(),
        Some(a) => {
            let h = &a.health;
            format!(
                "Repository Health  (composite, deterministic)\n\n\
                 Code Hotspots           {:>5.0}/100\n\
                 Ownership Risk          {:>5.0}/100\n\
                 Branch Hygiene          {:>5.0}/100\n\
                 Change Volatility       {:>5.0}/100\n\
                 Architecture Stability  {:>5.0}/100\n\
                 Recovery Risk           {:>5.0}/100\n\n\
                 Overall                 {:>5.0}/100\n\n\
                 Evidence: {} commits, {} contributors, {} files ({} analyzed) in {} ms",
                h.code_hotspots_score,
                h.ownership_risk_score,
                h.branch_hygiene_score,
                h.change_volatility_score,
                h.architecture_stability_score,
                h.recovery_risk_score,
                h.overall_score,
                a.total_commits,
                a.total_contributors,
                a.current_files,
                a.files.len(),
                a.analysis_duration_ms
            )
        }
    };
    let paragraph = Paragraph::new(content)
        .block(Block::default().title(" Health ").borders(Borders::ALL))
        .wrap(Wrap { trim: false })
        .style(Style::default().fg(Color::White));
    f.render_widget(paragraph, area);
}
