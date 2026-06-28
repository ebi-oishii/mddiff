use std::path::Path;

use mddiff_core::git::{list_bases, BaseKind, BaseOption, DiffMarker};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState};
use ratatui::Frame;

pub struct BasePicker {
    pub options: Vec<BaseOption>,
    pub state: ListState,
}

impl BasePicker {
    pub fn open(file: &Path, current: &str) -> Self {
        // Match the GUI's default: only commits that actually changed this
        // file. Special / Branch / Tag pass through (they're not filtered
        // out by `file_changed`).
        let options = list_bases(file, Some(current))
            .unwrap_or_default()
            .into_iter()
            .filter(|o| o.file_changed)
            .collect::<Vec<_>>();
        let mut state = ListState::default();
        if !options.is_empty() {
            state.select(Some(0));
        }
        Self { options, state }
    }

    pub fn up(&mut self) {
        if self.options.is_empty() {
            return;
        }
        let i = self.state.selected().unwrap_or(0);
        let new = if i == 0 { self.options.len() - 1 } else { i - 1 };
        self.state.select(Some(new));
    }

    pub fn down(&mut self) {
        if self.options.is_empty() {
            return;
        }
        let i = self.state.selected().unwrap_or(0);
        let new = (i + 1) % self.options.len();
        self.state.select(Some(new));
    }

    pub fn current(&self) -> Option<&BaseOption> {
        self.state.selected().and_then(|i| self.options.get(i))
    }

    pub fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let popup = centered_rect(80, 70, area);
        frame.render_widget(Clear, popup);

        let items: Vec<ListItem<'_>> = self
            .options
            .iter()
            .map(|opt| {
                let (tag, tag_color) = match opt.kind {
                    BaseKind::Special => (" special ", Color::Cyan),
                    BaseKind::Branch => (" branch  ", Color::Green),
                    BaseKind::Tag => (" tag     ", Color::Magenta),
                    BaseKind::Commit => (" commit  ", Color::Yellow),
                };
                let marker = match opt.marker {
                    DiffMarker::Differs => {
                        Span::styled("● ", Style::default().fg(Color::Yellow))
                    }
                    DiffMarker::Identical => {
                        Span::styled("○ ", Style::default().fg(Color::DarkGray))
                    }
                    DiffMarker::Redundant | DiffMarker::Unknown => Span::raw("  "),
                };
                let mut spans = vec![
                    marker,
                    Span::styled(
                        format!("[{}]", tag),
                        Style::default().fg(tag_color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
                    Span::raw(opt.label.clone()),
                ];
                if let Some(detail) = &opt.detail {
                    spans.push(Span::styled(
                        format!("   {}", detail),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
                ListItem::new(Line::from(spans))
            })
            .collect();

        let title = " Pick comparison base · ↑↓ / j k navigate · Enter select · Esc cancel ";
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .style(Style::default().bg(Color::Reset)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, popup, &mut self.state);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vert[1])[1]
}
