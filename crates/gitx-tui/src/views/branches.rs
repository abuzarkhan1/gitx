use crate::views::common;
use gitx_git::models::Branch;
use ratatui::{Frame, layout::Rect};

pub fn render(
    f: &mut Frame,
    area: Rect,
    branches: Option<&[Branch]>,
    branch_tips: Option<&[i64]>,
    scroll: usize,
    selected: usize,
) -> usize {
    let rows: Vec<String> = match branches {
        None => common::empty_rows("branches"),
        Some([]) => vec!["No branches found.".to_string()],
        Some(branches) => branches
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let mark = if b.is_remote { "[remote]" } else { "[local] " };
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
                format!(
                    "{mark} {:<24} {}  {age:>4}d old  last {activity}",
                    b.name,
                    common::short_oid(&b.target)
                )
            })
            .collect(),
    };
    common::render_scrollable(f, area, " Branches ", &rows, scroll, selected)
}
