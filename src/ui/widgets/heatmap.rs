use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use crate::analytics::heatmap_data::{weekday_labels, HeatmapData};
use crate::ui::theme::Theme;

pub struct HeatmapWidget<'a> {
    pub data: &'a HeatmapData,
    pub theme: &'a Theme,
}

const ROW_LABEL_WIDTH: usize = 4;

impl Widget for HeatmapWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines = Vec::new();

        lines.push(Line::styled(
            build_month_header(self.data),
            self.theme.muted_style(),
        ));

        for (row_index, weekday) in weekday_labels().iter().enumerate() {
            let name = weekday_label(row_index, weekday);
            let mut spans = vec![Span::styled(name, self.theme.muted_style())];
            for week in &self.data.weeks {
                let cell = &week[row_index];
                let (symbol, style) = if cell.is_future {
                    (' ', Style::default())
                } else {
                    match cell.intensity {
                        0 => ('·', Style::default().fg(self.theme.heat_0)),
                        1 => ('░', Style::default().fg(self.theme.heat_3)),
                        2 => ('▒', Style::default().fg(self.theme.heat_3)),
                        3 => ('▓', Style::default().fg(self.theme.heat_3)),
                        _ => ('█', Style::default().fg(self.theme.heat_3)),
                    }
                };
                spans.push(Span::styled(symbol.to_string(), style));
            }
            lines.push(Line::from(spans));
        }

        Paragraph::new(lines).render(area, buf);
    }
}

pub fn heatmap_legend_line(theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled("    Less ", theme.muted_style()),
        Span::styled("·", Style::default().fg(theme.heat_0)),
        Span::raw(" "),
        Span::styled("░", Style::default().fg(theme.heat_3)),
        Span::raw(" "),
        Span::styled("▒", Style::default().fg(theme.heat_3)),
        Span::raw(" "),
        Span::styled("▓", Style::default().fg(theme.heat_3)),
        Span::raw(" "),
        Span::styled("█", Style::default().fg(theme.heat_3)),
        Span::styled(" More", theme.muted_style()),
    ])
}

fn build_month_header(data: &HeatmapData) -> String {
    let mut header = vec![' '; ROW_LABEL_WIDTH + data.weeks.len()];
    let mut next_free_column = ROW_LABEL_WIDTH;

    for (week_index, label) in &data.month_labels {
        let start = ROW_LABEL_WIDTH + *week_index;
        if start < next_free_column || start >= header.len() {
            continue;
        }

        for (offset, ch) in label.chars().enumerate() {
            let column = start + offset;
            if column >= header.len() {
                break;
            }
            header[column] = ch;
        }

        next_free_column = start + label.chars().count() + 1;
    }

    header.into_iter().collect()
}

fn weekday_label(row_index: usize, weekday: &chrono::Weekday) -> String {
    match row_index {
        0 | 2 | 4 | 6 => format!(
            "{:>3} ",
            format!("{:?}", weekday).chars().take(3).collect::<String>()
        ),
        _ => " ".repeat(ROW_LABEL_WIDTH),
    }
}
