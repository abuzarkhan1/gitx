use crate::app::App;
use crate::views::common;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

/// Repository Health (docs/08 §3): the six deterministic sub-scores + overall.
/// Selecting a row reveals that sub-score's evidence — never just a number.
pub fn render(f: &mut Frame, area: Rect, app: &App) -> usize {
    let Some(analysis) = &app.hotspots else {
        common::render_scrollable(f, area, " Health ", &["No repository loaded.".into()], 0, 0);
        return 1;
    };
    let h = &analysis.health;

    let mut rows: Vec<String> = vec![
        format!("Code Hotspots           {:>5.0}/100", h.code_hotspots_score),
        format!(
            "Ownership Risk          {:>5.0}/100",
            h.ownership_risk_score
        ),
        format!(
            "Branch Hygiene          {:>5.0}/100",
            h.branch_hygiene_score
        ),
        format!(
            "Change Volatility       {:>5.0}/100",
            h.change_volatility_score
        ),
        format!(
            "Architecture Stability  {:>5.0}/100",
            h.architecture_stability_score
        ),
        format!("Recovery Risk           {:>5.0}/100", h.recovery_risk_score),
    ];
    rows.push(String::new());
    rows.push(format!(
        "Overall                 {:>5.0}/100",
        h.overall_score
    ));

    let labels = [
        "Code Hotspots",
        "Ownership Risk",
        "Branch Hygiene",
        "Change Volatility",
        "Architecture Stability",
        "Recovery Risk",
    ];

    // Split: score list on top, selected sub-score's evidence below.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(area);

    let list_count = common::render_scrollable(
        f,
        chunks[0],
        " Health (sub-scores) ",
        &rows,
        app.scroll,
        app.selected,
    );

    // Evidence for the selected row (docs/08: selecting reveals evidence).
    let selected = app.selected.min(labels.len().saturating_sub(1));
    let mut evidence = vec![format!("{} — evidence", labels[selected]), String::new()];
    match app.health_evidence.get(selected) {
        Some(lines) if !lines.is_empty() => evidence.extend(lines.iter().cloned()),
        _ => evidence.push("  (analysis unavailable — run `gitx refresh`)".to_string()),
    }
    common::render_scrollable(
        f,
        chunks[1],
        &format!(" Evidence: {} ", labels[selected]),
        &evidence,
        0,
        0,
    );

    list_count
}
