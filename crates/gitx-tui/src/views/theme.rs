//! Shared visual helpers (docs/25 UX guidelines, docs/16 §[ui] theme):
//! severity/health colors, styled lines, horizontal bars, vertical bar
//! charts, and the loadable color theme.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::sync::OnceLock;

/// Risk/hotspot severity color (docs/10 §2 bands: 0–30, 31–60, 61–80, 81–100).
pub fn severity_color(score: f64) -> Color {
    if score >= 80.0 {
        Color::Red
    } else if score >= 60.0 {
        Color::Yellow
    } else if score >= 30.0 {
        Color::Blue
    } else {
        Color::Green
    }
}

/// Classification → color (docs/08 Hotspots severity badges).
pub fn class_color(class: &str) -> Color {
    match class {
        "CRITICAL" => Color::Red,
        "HIGH" => Color::Yellow,
        "MEDIUM" => Color::Blue,
        _ => Color::Green,
    }
}

/// Health sub-score color: ≥70 green, 40–69 yellow, <40 red.
pub fn health_color(score: f64) -> Color {
    if score >= 70.0 {
        Color::Green
    } else if score >= 40.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

/// Activity recency color: <30d green, 30–90d yellow, >90d red.
pub fn recency_color(days: u64) -> Color {
    if days < 30 {
        Color::Green
    } else if days < 90 {
        Color::Yellow
    } else {
        Color::Red
    }
}

pub fn plain(s: impl Into<String>) -> Line<'static> {
    Line::from(s.into())
}

pub fn dim(s: impl Into<String>) -> Line<'static> {
    Line::from(Span::styled(s.into(), Style::default().fg(Color::DarkGray)))
}

pub fn colored(s: impl Into<String>, color: Color) -> Line<'static> {
    Line::from(Span::styled(s.into(), Style::default().fg(color)))
}

pub fn strong(s: impl Into<String>, color: Color) -> Line<'static> {
    Line::from(Span::styled(
        s.into(),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    ))
}

pub fn heading(s: impl Into<String>) -> Line<'static> {
    Line::from(Span::styled(
        s.into(),
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ))
}

/// Styled line mixing a plain label and a colored value (e.g. `Commits  1284`).
pub fn kv(label: &str, value: String, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<22} "), Style::default()),
        Span::styled(
            value,
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
    ])
}

/// Horizontal bar: `label [██████░░░░]   87%`, bar colored by `color`.
pub fn hbar(label: String, pct: f64, width: usize, color: Color) -> Line<'static> {
    let pct = pct.clamp(0.0, 100.0);
    let filled = ((pct / 100.0) * width as f64).round() as usize;
    let bar: String = "█".repeat(filled) + &"░".repeat(width.saturating_sub(filled));
    Line::from(vec![
        Span::styled(format!("{label:<26} "), Style::default()),
        Span::styled(bar, Style::default().fg(color)),
        Span::styled(format!(" {pct:>5.0}%"), Style::default().fg(color)),
    ])
}

/// Vertical bar chart from `(label, value)` pairs: `height` rows of block
/// glyphs plus a bottom label row (last character of each label).
pub fn vchart(data: &[(String, u32)], height: usize) -> Vec<Line<'static>> {
    let max = data.iter().map(|(_, v)| *v).max().unwrap_or(0).max(1);
    let mut rows = Vec::new();
    for level in (0..height).rev() {
        let mut line = String::from(" ");
        for (_, v) in data {
            let h = (v * height as u32).div_ceil(max) as usize;
            line.push(if h > level { '█' } else { ' ' });
        }
        rows.push(Line::from(line));
    }
    let mut label = String::from(" ");
    for (l, _) in data {
        label.push(
            l.rsplit('-')
                .next()
                .and_then(|w| w.chars().last())
                .unwrap_or(' '),
        );
    }
    rows.push(Line::from(Span::styled(
        label,
        Style::default().fg(Color::DarkGray),
    )));
    rows
}

/// A color theme (docs/16 `[ui] theme`, `GITX_THEME` env). Content stays
/// semantic; the theme drives chrome (header, navigation highlight, status).
pub struct Theme {
    pub name: String,
    pub accent: Color,
    pub fg: Color,
    pub status_bg: Color,
    pub sel_bg: Color,
}

impl Theme {
    pub fn load() -> Self {
        let name = std::env::var("GITX_THEME")
            .ok()
            .or_else(|| {
                gitx_core::config::Config::default_path()
                    .and_then(|p| gitx_core::config::Config::load(&p).ok())
                    .map(|c| c.ui.theme)
            })
            .unwrap_or_else(|| "default".into());
        Self::named(&name)
    }

    pub fn named(name: &str) -> Self {
        match name {
            "light" => Theme {
                name: "light".into(),
                accent: Color::Blue,
                fg: Color::Black,
                status_bg: Color::Blue,
                sel_bg: Color::Cyan,
            },
            _ => Theme {
                name: "default".into(),
                accent: Color::Yellow,
                fg: Color::White,
                status_bg: Color::Blue,
                // High-contrast selection (docs/25): white-on-blue reads
                // clearly even in long lists; DarkGray was too subtle.
                sel_bg: Color::Blue,
            },
        }
    }
}

/// The process-wide theme, loaded once from `GITX_THEME` or `[ui] theme`
/// (docs/16 §[ui]). Content widgets stay semantic; callers fetch it for the
/// chrome colors (selection, header, status bar).
pub fn global() -> &'static Theme {
    static GLOBAL: OnceLock<Theme> = OnceLock::new();
    GLOBAL.get_or_init(Theme::load)
}
