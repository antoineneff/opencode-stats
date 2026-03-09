use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame, TerminalOptions, Viewport};
use tokio::sync::mpsc;

use crate::analytics::{build_snapshot, AnalyticsSnapshot};
use crate::cache::models_cache::{refresh_remote_models, PricingCatalog};
use crate::db::models::AppData;
use crate::ui::models::render_models;
use crate::ui::overview::render_overview;
use crate::ui::theme::{Theme, ThemeMode};
use crate::utils::time::TimeRange;

const VIEWPORT_HEIGHT: u16 = 27;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Page {
    #[default]
    Overview,
    Models,
}

impl Page {
    pub fn title(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Models => "Models",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Overview => Self::Models,
            Self::Models => Self::Overview,
        }
    }

    pub fn previous(self) -> Self {
        self.next()
    }
}

pub struct App {
    pub data: AppData,
    pub pricing: PricingCatalog,
    pub snapshot: AnalyticsSnapshot,
    pub page: Page,
    pub range: TimeRange,
    pub theme_mode: ThemeMode,
    pub should_quit: bool,
    pub status_message: Option<String>,
    pub focused_model_index: usize,
    pricing_updates: mpsc::UnboundedReceiver<PricingCatalog>,
}

impl App {
    pub fn new(data: AppData, pricing: PricingCatalog, theme_mode: ThemeMode) -> Self {
        let snapshot = build_snapshot(&data, &pricing, TimeRange::All);
        let (sender, receiver) = mpsc::unbounded_channel();
        if pricing.refresh_needed {
            let cache_path = pricing.cache_path.clone();
            tokio::spawn(refresh_remote_models(cache_path, sender));
        }

        Self {
            data,
            pricing,
            snapshot,
            page: Page::Overview,
            range: TimeRange::All,
            theme_mode,
            should_quit: false,
            status_message: None,
            focused_model_index: 0,
            pricing_updates: receiver,
        }
    }

    pub fn run(mut self) -> Result<()> {
        let mut terminal = ratatui::init_with_options(TerminalOptions {
            viewport: Viewport::Inline(VIEWPORT_HEIGHT),
        });

        let app_result = self.run_loop(&mut terminal);
        ratatui::restore();
        app_result
    }

    fn run_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            while let Ok(pricing) = self.pricing_updates.try_recv() {
                self.pricing = pricing;
                self.recompute();
                self.status_message = Some("Pricing cache refreshed from models.dev".to_string());
            }

            terminal.draw(|frame| self.render(frame))?;
            if event::poll(Duration::from_millis(200))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key);
                    }
                }
            }
        }
        Ok(())
    }

    fn recompute(&mut self) {
        self.snapshot = build_snapshot(&self.data, &self.pricing, self.range);
    }

    fn render(&self, frame: &mut Frame<'_>) {
        let theme = Theme::from_mode(self.theme_mode);
        let area = frame.area();
        let vertical = ratatui::layout::Layout::vertical([
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(3),
            ratatui::layout::Constraint::Min(20),
            ratatui::layout::Constraint::Length(2),
        ]);
        let [divider, header, body, footer] = vertical.areas(area);

        frame.render_widget(
            ratatui::widgets::Paragraph::new("─".repeat(divider.width as usize))
                .style(theme.divider_style()),
            divider,
        );

        self.render_header(frame, header, &theme);

        match self.page {
            Page::Overview => render_overview(frame, body, &self.snapshot, self.range, &theme),
            Page::Models => render_models(
                frame,
                body,
                &self.snapshot,
                self.range,
                self.focused_model_index,
                &theme,
            ),
        }

        self.render_footer(frame, footer, &theme);
    }

    fn render_header(&self, frame: &mut Frame<'_>, area: ratatui::layout::Rect, theme: &Theme) {
        let title = format!("oc-stats  {}", self.page.title());
        let tabs = [Page::Overview, Page::Models]
            .into_iter()
            .map(|page| {
                if page == self.page {
                    ratatui::text::Span::styled(
                        format!(" {} ", page.title()),
                        ratatui::style::Style::default()
                            .fg(theme.tab_active_fg)
                            .bg(theme.tab_active_bg)
                            .add_modifier(ratatui::style::Modifier::BOLD),
                    )
                } else {
                    ratatui::text::Span::styled(
                        format!(" {} ", page.title()),
                        ratatui::style::Style::default().fg(theme.muted),
                    )
                }
            })
            .collect::<Vec<_>>();

        let range_text = format!(
            "Range: {}   Source: {:?}",
            self.range.label(),
            self.data.source
        );
        let header = ratatui::widgets::Paragraph::new(ratatui::text::Text::from(vec![
            ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(title, theme.title_style()),
                ratatui::text::Span::raw("    "),
                ratatui::text::Span::styled(range_text, theme.muted_style()),
            ]),
            ratatui::text::Line::from(tabs),
        ]));
        frame.render_widget(header, area);
    }

    fn render_footer(&self, frame: &mut Frame<'_>, area: ratatui::layout::Rect, theme: &Theme) {
        let status = self.status_message.as_deref().unwrap_or(
            "Tab/Left/Right switch pages | r cycle range | 1/2/3 quick pick\nj/k or up/down focus model | Ctrl+S copy | q exit",
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(theme.muted_style()),
            area,
        );
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Tab | KeyCode::Right => self.page = self.page.next(),
            KeyCode::Left => self.page = self.page.previous(),
            KeyCode::Down | KeyCode::Char('j') => self.advance_focused_model(1),
            KeyCode::Up | KeyCode::Char('k') => self.advance_focused_model(-1),
            KeyCode::Char('r') => {
                self.range = self.range.cycle();
                self.focused_model_index = 0;
                self.recompute();
            }
            KeyCode::Char(value)
                if key.modifiers.contains(KeyModifiers::CONTROL) && value == 's' =>
            {
                self.copy_summary();
            }
            KeyCode::Char(value) => {
                if let Some(range) = TimeRange::from_shortcut(value) {
                    self.range = range;
                    self.focused_model_index = 0;
                    self.recompute();
                }
            }
            _ => {}
        }
    }

    fn advance_focused_model(&mut self, delta: isize) {
        if self.page != Page::Models || self.snapshot.models.is_empty() {
            return;
        }

        let current = self.focused_model_index as isize;
        let total = self.snapshot.models.len() as isize;
        let next = (current + delta).rem_euclid(total) as usize;
        self.focused_model_index = next;
    }

    fn copy_summary(&mut self) {
        let summary = match self.page {
            Page::Overview => format!(
                "oc-stats {}\nTokens: {}\nCost: {}\nSessions: {}\nInteractions: {}",
                self.range.label(),
                self.snapshot.overview.total_tokens,
                self.snapshot.overview.total_cost,
                self.snapshot.overview.sessions,
                self.snapshot.overview.interactions,
            ),
            Page::Models => self
                .snapshot
                .models
                .iter()
                .take(8)
                .map(|row| {
                    format!(
                        "{}: {} tokens ({:.1}%)",
                        row.model_id, row.total_tokens, row.percentage
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"),
        };

        match arboard::Clipboard::new().and_then(|mut clipboard| clipboard.set_text(summary)) {
            Ok(()) => {
                self.status_message = Some("Copied current page summary to clipboard".to_string())
            }
            Err(err) => self.status_message = Some(format!("Clipboard unavailable: {err}")),
        }
    }
}