use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;

use crate::analytics::AnalyticsSnapshot;
use crate::ui::theme::Theme;
use crate::ui::widgets::common::{metric_line, range_selector_line, truncate_label};
use crate::ui::widgets::heatmap::HeatmapWidget;
use crate::utils::formatting::{format_relative_time, format_tokens, format_usd_precise};

const CONTENT_WIDTH: u16 = 68;

pub fn render_overview(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    snapshot: &AnalyticsSnapshot,
    range: crate::utils::time::TimeRange,
    theme: &Theme,
) {
    let content = left_aligned_content(area, CONTENT_WIDTH);
    let [heatmap, legend, spacer, ranges, stats, fun] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(7),
            Constraint::Min(3),
        ])
        .areas(content);

    frame.render_widget(
        HeatmapWidget {
            data: &snapshot.heatmap,
            theme,
        },
        heatmap,
    );

    frame.render_widget(
        Paragraph::new(crate::ui::widgets::heatmap::heatmap_legend_line(theme)),
        legend,
    );
    frame.render_widget(Paragraph::new(""), spacer);
    frame.render_widget(Paragraph::new(range_selector_line(range, theme)), ranges);

    let [left, right] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .areas(stats);

    let favorite_model = snapshot
        .models
        .first()
        .map(|row| truncate_label(&row.model_id, 22))
        .unwrap_or_else(|| "n/a".to_string());

    let left_text = Text::from(vec![
        Line::from(vec![
            Span::styled("Favorite model: ", theme.muted_style()),
            Span::styled(favorite_model, theme.accent_style()),
        ]),
        Line::from(""),
        metric_line(
            "Input: ",
            format_tokens(snapshot.overview.input_tokens),
            theme,
        ),
        metric_line(
            "Output: ",
            format_tokens(snapshot.overview.output_tokens),
            theme,
        ),
        metric_line(
            "Cache: ",
            format_tokens(snapshot.overview.cache_tokens),
            theme,
        ),
        Line::from(""),
        metric_line("Sessions: ", snapshot.overview.sessions.to_string(), theme),
    ]);
    frame.render_widget(Paragraph::new(left_text), left);

    let right_text = Text::from(vec![
        Line::from(vec![
            Span::styled("Total tokens: ", theme.muted_style()),
            Span::styled(
                format_tokens(snapshot.overview.total_tokens),
                theme.title_style(),
            ),
        ]),
        Line::from(""),
        metric_line(
            "Total cost: ",
            format_usd_precise(snapshot.overview.total_cost),
            theme,
        ),
        metric_line(
            "Interactions: ",
            snapshot.overview.interactions.to_string(),
            theme,
        ),
        metric_line(
            "Models used: ",
            snapshot.overview.models_used.to_string(),
            theme,
        ),
        metric_line(
            "Active days: ",
            snapshot.overview.active_days.to_string(),
            theme,
        ),
        metric_line(
            "Avg/day: ",
            format_tokens(snapshot.overview.average_tokens_per_day),
            theme,
        ),
        metric_line(
            "Latest activity: ",
            format_relative_time(snapshot.overview.latest_activity),
            theme,
        ),
    ]);
    frame.render_widget(Paragraph::new(right_text), right);

    let fun_text = Paragraph::new(Text::from(vec![
        Line::from(vec![Span::styled("Usage comparison", theme.accent_style())]),
        Line::from(""),
        Line::from(vec![Span::styled(
            snapshot.overview.fun_comparison.clone(),
            theme.comparison_style().add_modifier(Modifier::BOLD),
        )]),
    ]));
    frame.render_widget(fun_text, fun);
}

fn left_aligned_content(area: Rect, width: u16) -> Rect {
    let width = width.min(area.width);
    Rect {
        x: area.x,
        y: area.y,
        width,
        height: area.height,
    }
}
