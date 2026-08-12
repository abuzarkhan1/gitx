use crate::views::{common, theme};
use gitx_analysis::branch::BranchIntelligence;
use gitx_git::models::Branch;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

pub fn render(
    f: &mut Frame,
    area: Rect,
    branches: Option<&[Branch]>,
    branch_tips: Option<&[i64]>,
    intel: Option<&[Option<BranchIntelligence>]>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<Line<'static>> = match branches {
        None => common::empty_rows("branches"),
        Some([]) => vec![theme::plain("No branches found.")],
        Some(branches) => branches
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let mark = if b.is_remote {
                    "[remote] "
                } else {
                    "[local]  "
                };
                // Age + last activity from the tip commit (docs/08 Branch view).
                let (age, activity) = branch_tips
                    .and_then(|tips| tips.get(i))
                    .map(|tip| {
                        let now = chrono::Utc::now().timestamp();
                        let days = if *tip > 0 {
                            ((now - *tip).max(0) / 86_400) as u64
                        } else {
                            0
                        };
                        (days, common::ts(*tip))
                    })
                    .unwrap_or((0, "-".to_string()));

                let mut spans: Vec<Span<'static>> = vec![Span::styled(
                    format!("{mark} {:<24} {}", b.name, common::short_oid(&b.target)),
                    Style::default().fg(theme::global().fg),
                )];

                // Ahead/behind vs the current branch, with a visual bar
                // (docs/08 Branch view), plus divergence and shared files
                // (docs/10 §5): how far the branch has split from the base
                // and how many files both sides have changed.
                if let Some(Some(bi)) = intel.and_then(|list| list.get(i)) {
                    let total = (bi.ahead + bi.behind).max(1);
                    let width = 12usize;
                    let ahead_filled =
                        ((bi.ahead as f64 / total as f64) * width as f64).round() as usize;
                    let bar: String =
                        "█".repeat(ahead_filled) + &"░".repeat(width.saturating_sub(ahead_filled));
                    let color = if bi.ahead > 0 && bi.behind > 0 {
                        Color::Yellow
                    } else if bi.ahead > 0 {
                        Color::Green
                    } else {
                        Color::DarkGray
                    };
                    spans.push(Span::styled(
                        format!(
                            " [{bar}] ahead {} behind {} | diverged {} | {} shared file{}",
                            bi.ahead,
                            bi.behind,
                            bi.diverged_commits,
                            bi.shared_files,
                            if bi.shared_files == 1 { "" } else { "s" }
                        ),
                        Style::default().fg(color),
                    ));
                    // Staleness marker (docs/10 §5): stale branches stand out.
                    if bi.is_stale {
                        spans.push(Span::styled(
                            "  [stale]",
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        ));
                    }
                }

                // Staleness color (docs/10 §5): green <30d, yellow 30–90d, red >90d.
                spans.push(Span::styled(
                    format!("  {age:>4}d old  last {activity}"),
                    Style::default().fg(theme::recency_color(age)),
                ));
                Line::from(spans)
            })
            .collect(),
    };
    common::render_scrollable(
        f,
        area,
        " Branches — age · activity · ahead/behind · divergence · shared files ",
        &rows,
        scroll,
        selected,
    )
}
