use ratatui::text::{Line, Span};

use crate::ui::theme::Theme;
use crate::utils::time::TimeRange;

pub fn range_selector_line(range: TimeRange, theme: &Theme) -> Line<'static> {
    let options = [TimeRange::All, TimeRange::Last7Days, TimeRange::Last30Days];
    let mut spans = Vec::new();

    for (index, option) in options.into_iter().enumerate() {
        if index > 0 {
            spans.push(Span::styled(" | ", theme.muted_style()));
        }

        let style = if option == range {
            theme.accent_style()
        } else {
            theme.muted_style()
        };
        spans.push(Span::styled(option.label().to_string(), style));
    }

    Line::from(spans)
}

pub fn truncate_label(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_string();
    }

    let keep = max_chars.saturating_sub(3);
    let mut truncated = value.chars().take(keep).collect::<String>();
    truncated.push_str("...");
    truncated
}

pub fn metric_line<'a>(label: &'a str, value: String, theme: &Theme) -> Line<'a> {
    Line::from(vec![
        Span::styled(label.to_string(), theme.muted_style()),
        Span::raw(value),
    ])
}
