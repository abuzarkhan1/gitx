use crate::app::Contributor;
use crate::views::{common, theme};
use ratatui::{Frame, layout::Rect, style::Color, text::Line};

pub fn render(
    f: &mut Frame,
    area: Rect,
    contributors: Option<&[Contributor]>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<Line<'static>> = match contributors {
        None => common::empty_rows("repository"),
        Some([]) => vec![theme::plain("No contributors found.")],
        Some(list) => {
            let total: u64 = list.iter().map(|c| c.commits).sum::<u64>().max(1);
            let mut out = Vec::new();
            for c in list {
                let weight = c.commits as f64 / total as f64 * 100.0;
                let files = if c.files_touched > 0 {
                    format!("  {:>3} files", c.files_touched)
                } else {
                    String::new()
                };
                let areas = if c.areas.is_empty() {
                    String::new()
                } else {
                    format!("  areas: {}", c.areas.join(", "))
                };
                // Contribution-weight bar (docs/08 Contributors): bar length =
                // share of commits; activity dates + areas dimmed below.
                out.push(theme::hbar(
                    format!("{}  ({} commits)", c.name, c.commits),
                    weight,
                    24,
                    Color::Magenta,
                ));
                out.push(theme::dim(format!(
                    "         {files}{areas}  first {} → last {}",
                    common::ts(c.first_activity),
                    common::ts(c.last_activity)
                )));
            }
            out
        }
    };
    common::render_scrollable(
        f,
        area,
        " Contributors — share of commits ",
        &rows,
        scroll,
        selected,
    )
}
