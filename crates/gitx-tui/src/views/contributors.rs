use crate::app::Contributor;
use crate::views::common;
use ratatui::{Frame, layout::Rect};

pub fn render(
    f: &mut Frame,
    area: Rect,
    contributors: Option<&[Contributor]>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<String> = match contributors {
        None => vec!["No repository loaded.".to_string()],
        Some([]) => vec!["No contributors found.".to_string()],
        Some(list) => {
            let total: u64 = list.iter().map(|c| c.commits).sum::<u64>().max(1);
            list.iter()
                .map(|c| {
                    let weight = c.commits as f64 / total as f64 * 100.0;
                    let files = if c.files_touched > 0 {
                        format!("  {:>3} files", c.files_touched)
                    } else {
                        String::new()
                    };
                    format!(
                        "{:>6} commits ({:>4.1}%){files}  {:<22} {} → {}",
                        c.commits,
                        weight,
                        c.name,
                        common::ts(c.first_activity),
                        common::ts(c.last_activity)
                    )
                })
                .collect()
        }
    };
    common::render_scrollable(f, area, " Contributors ", &rows, scroll, selected)
}
