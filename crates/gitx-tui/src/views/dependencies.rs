use crate::views::{common, theme};
use gitx_analysis::manifest::Dependency;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
};
use std::path::PathBuf;

pub fn render(
    f: &mut Frame,
    area: Rect,
    dependencies: Option<&[(PathBuf, Vec<Dependency>)]>,
    loading: bool,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<Line<'static>> = match dependencies {
        None => common::panel_placeholder(loading, "dependency analysis"),
        Some([]) => vec![
            theme::plain("No supported dependency manifests found in HEAD."),
            theme::dim("Supported: Cargo.toml/lock, package.json/lock, go.mod/sum."),
        ],
        Some(list) => {
            let mut out = Vec::new();
            for (path, deps) in list {
                out.push(theme::strong(path.display().to_string(), Color::Cyan));
                if deps.is_empty() {
                    out.push(theme::dim("    (no dependencies declared)"));
                }
                for dep in deps {
                    let line = match &dep.version {
                        Some(v) => Line::from(vec![
                            Span::raw("    "),
                            Span::styled(dep.name.clone(), Style::default().fg(Color::White)),
                            Span::styled(format!(" {v}"), Style::default().fg(Color::DarkGray)),
                        ]),
                        None => theme::plain(format!("    {}", dep.name)),
                    };
                    out.push(line);
                }
            }
            out
        }
    };
    common::render_scrollable(
        f,
        area,
        " Dependencies — declared in HEAD ",
        &rows,
        scroll,
        selected,
    )
}
