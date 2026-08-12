use crate::app::App;
use crate::views::{common, theme};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

/// Repository Health (docs/08 §3): the six deterministic sub-scores as colored
/// gauges + an overall gauge, with the selected sub-score's evidence below —
/// never just a number (docs/10 §8, docs/25).
pub fn render(f: &mut Frame, area: Rect, app: &App) -> usize {
    let Some(analysis) = &app.hotspots else {
        common::render_scrollable(f, area, " Health ", &common::empty_rows("health"), 0, 0);
        return 1;
    };
    let h = &analysis.health;

    let mut rows: Vec<ratatui::text::Line<'static>> = vec![
        theme::hbar(
            "Code Hotspots".into(),
            h.code_hotspots_score,
            24,
            theme::health_color(h.code_hotspots_score),
        ),
        theme::hbar(
            "Ownership Risk".into(),
            h.ownership_risk_score,
            24,
            theme::health_color(h.ownership_risk_score),
        ),
        theme::hbar(
            "Branch Hygiene".into(),
            h.branch_hygiene_score,
            24,
            theme::health_color(h.branch_hygiene_score),
        ),
        theme::hbar(
            "Change Volatility".into(),
            h.change_volatility_score,
            24,
            theme::health_color(h.change_volatility_score),
        ),
        theme::hbar(
            "Architecture Stability".into(),
            h.architecture_stability_score,
            24,
            theme::health_color(h.architecture_stability_score),
        ),
        theme::hbar(
            "Recovery Risk".into(),
            h.recovery_risk_score,
            24,
            theme::health_color(h.recovery_risk_score),
        ),
    ];
    rows.push(ratatui::text::Line::default());
    rows.push(theme::hbar(
        "Overall".into(),
        h.overall_score,
        24,
        theme::health_color(h.overall_score),
    ));
    let verdict = if h.overall_score >= 70.0 {
        "mostly healthy — a few files may need attention"
    } else if h.overall_score >= 40.0 {
        "mixed signals — worth reviewing the red areas"
    } else {
        "needs attention — several signals are weak"
    };
    rows.push(theme::dim(format!(
        "  Plain language: {verdict} (higher = healthier)."
    )));

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
        " Health (sub-scores, 0 = bad · 100 = good) ",
        &rows,
        app.scroll,
        app.selected,
    );

    // Evidence for the selected row (docs/08: selecting reveals evidence).
    let selected = app.selected.min(labels.len().saturating_sub(1));
    let mut evidence: Vec<ratatui::text::Line<'static>> = vec![theme::strong(
        format!("{} — why this score", labels[selected]),
        theme::global().accent,
    )];
    match app.health_evidence.get(selected) {
        Some(lines) if !lines.is_empty() => {
            for line in lines {
                evidence.push(theme::dim(format!("  {line}")));
            }
        }
        _ => evidence.push(theme::dim("  (analysis unavailable — run `gitx refresh`)")),
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
