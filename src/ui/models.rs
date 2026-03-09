use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;

use crate::analytics::model_stats::{chart_with_focus, ModelUsageRow};
use crate::analytics::AnalyticsSnapshot;
use crate::ui::theme::Theme;
use crate::ui::widgets::common::{range_selector_line, truncate_label};
use crate::ui::widgets::linechart::build_chart;
use crate::utils::formatting::{format_tokens, format_usd_precise};
use crate::utils::time::TimeRange;

const CONTENT_WIDTH: u16 = 68;

pub fn render_models(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    snapshot: &AnalyticsSnapshot,
    range: TimeRange,
    focused_model_index: usize,
    theme: &Theme,
) {
    let content = left_aligned_content(area, CONTENT_WIDTH);
    let [chart_area, controls_area, detail_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(13),
            Constraint::Length(2),
            Constraint::Length(5),
        ])
        .areas(content);

    let focused_row = snapshot.models.get(focused_model_index);
    let chart_data = chart_with_focus(
        &snapshot.chart,
        focused_row.map(|row| row.model_id.as_str()),
    );
    frame.render_widget(build_chart(&chart_data, theme), chart_area);
    frame.render_widget(
        Paragraph::new(range_and_focus_line(
            range,
            focused_model_index,
            &snapshot.models,
            theme,
        )),
        controls_area,
    );

    if let Some(row) = focused_row {
        frame.render_widget(
            Paragraph::new(model_detail_text(row, focused_model_index, theme)),
            detail_area,
        );
    } else {
        frame.render_widget(
            Paragraph::new("No model activity in this time range.").style(theme.muted_style()),
            detail_area,
        );
    }
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

fn range_and_focus_line(
    range: TimeRange,
    focused_model_index: usize,
    models: &[ModelUsageRow],
    theme: &Theme,
) -> Text<'static> {
    let total = models.len().max(1);
    let focused_name = models
        .get(focused_model_index)
        .map(|row| truncate_label(&row.model_id, 18))
        .unwrap_or_else(|| "n/a".to_string());

    Text::from(vec![
        range_selector_line(range, theme),
        Line::from(vec![
            Span::styled(
                format!("Focus {}/{}", focused_model_index.min(total - 1) + 1, total),
                theme.muted_style(),
            ),
            Span::styled("  |  ", theme.muted_style()),
            Span::styled(focused_name, theme.accent_style()),
            Span::styled("  |  ", theme.muted_style()),
            Span::styled("j/k or up/down to cycle", theme.muted_style()),
        ]),
    ])
}

fn model_detail_text(
    row: &ModelUsageRow,
    focused_model_index: usize,
    theme: &Theme,
) -> Text<'static> {
    let color = theme.series_color(focused_model_index);

    Text::from(vec![
        Line::from(vec![
            Span::styled("● ", Style::default().fg(color)),
            Span::styled(
                truncate_label(&row.model_id, 42),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(format!("({:.1}%)", row.percentage), theme.muted_style()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Tokens ", theme.muted_style()),
            Span::raw(format_tokens(row.total_tokens)),
            Span::raw("  "),
            Span::styled("Cost ", theme.muted_style()),
            Span::raw(format_usd_precise(row.cost)),
        ]),
        Line::from(vec![
            Span::styled("Sessions ", theme.muted_style()),
            Span::raw(row.sessions.to_string()),
            Span::raw("  "),
            Span::styled("Interactions ", theme.muted_style()),
            Span::raw(row.interactions.to_string()),
            Span::raw("  "),
            Span::styled("Days ", theme.muted_style()),
            Span::raw(row.active_days.to_string()),
        ]),
        Line::from(vec![
            Span::styled("p50 output rate ", theme.muted_style()),
            Span::raw(format!("{:.1} tok/s", row.p50_output_tokens_per_second)),
        ]),
    ])
}
