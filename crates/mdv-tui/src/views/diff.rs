use mdv_core::diff::{DiffLine, HunkKind, HunkSummary};
use mdv_core::git::SideBySidePayload;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub const SBS_MIN_WIDTH: u16 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Submode {
    Highlight,
    Full,
    SideBySide,
}

impl Submode {
    pub fn label(self) -> &'static str {
        match self {
            Submode::Highlight => "Highlight Only",
            Submode::Full => "Full",
            Submode::SideBySide => "Side-by-Side",
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            Submode::Highlight => Submode::Full,
            Submode::Full => Submode::SideBySide,
            Submode::SideBySide => Submode::Highlight,
        }
    }
}

pub struct DiffView {
    pub submode: Submode,
    pub scroll: u16,
}

impl DiffView {
    pub fn new() -> Self {
        Self {
            submode: Submode::Highlight,
            scroll: 0,
        }
    }

    pub fn scroll_up(&mut self, n: u16) {
        self.scroll = self.scroll.saturating_sub(n);
    }

    pub fn scroll_down(&mut self, n: u16) {
        self.scroll = self.scroll.saturating_add(n);
    }

    pub fn toggle_submode(&mut self) {
        self.submode = self.submode.toggled();
        self.scroll = 0;
    }

    pub fn render_highlight(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        text: &str,
        hunks: &[HunkSummary],
    ) {
        let lines = highlight_lines_new(text, hunks);
        let title = format!(" Diff · {} ", self.submode.label());
        let para = Paragraph::new(lines)
            .scroll((self.scroll, 0))
            .block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(para, area);
    }

    pub fn render_full(&self, frame: &mut Frame<'_>, area: Rect, diff_lines: &[DiffLine]) {
        let lines = full_lines(diff_lines);
        let title = format!(" Diff · {} ", self.submode.label());
        let para = Paragraph::new(lines)
            .scroll((self.scroll, 0))
            .block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(para, area);
    }

    pub fn render_message(&self, frame: &mut Frame<'_>, area: Rect, msg: &str) {
        let para = Paragraph::new(msg).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Diff "),
        );
        frame.render_widget(para, area);
    }

    pub fn render_sidebyside(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        payload: &SideBySidePayload,
        base_label: &str,
    ) {
        if area.width < SBS_MIN_WIDTH {
            self.render_message(
                frame,
                area,
                &format!(
                    "Terminal too narrow for Side-by-Side\n(need ≥{} cols, got {}).\n\nUse Tab / Ctrl+D to switch back to Highlight Only or Full.",
                    SBS_MIN_WIDTH, area.width
                ),
            );
            return;
        }

        let halves = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let old_para = Paragraph::new(highlight_lines_old(&payload.old_text, &payload.hunks))
            .scroll((self.scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" OLD · {} ", base_label)),
            );
        frame.render_widget(old_para, halves[0]);

        let new_para = Paragraph::new(highlight_lines_new(&payload.new_text, &payload.hunks))
            .scroll((self.scroll, 0))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" NEW · current buffer "),
            );
        frame.render_widget(new_para, halves[1]);
    }
}

fn highlight_lines_new<'a>(text: &'a str, hunks: &[HunkSummary]) -> Vec<Line<'a>> {
    let mut out = Vec::new();

    let removed_at = |line: usize| -> usize {
        hunks
            .iter()
            .filter(|h| h.kind == HunkKind::Removed && h.new_start == line)
            .map(|h| h.removed_count())
            .sum()
    };

    let n = removed_at(0);
    if n > 0 {
        out.push(removal_marker(n));
    }

    for (i, l) in text.lines().enumerate() {
        let no = i + 1;
        let kind = hunks
            .iter()
            .find(|h| h.kind != HunkKind::Removed && no >= h.new_start && no <= h.new_end)
            .map(|h| h.kind);

        let (mark, mark_style) = match kind {
            Some(HunkKind::Added) => ("▎", Style::default().fg(Color::Green)),
            Some(HunkKind::Modified) => ("▎", Style::default().fg(Color::Yellow)),
            _ => (" ", Style::default()),
        };

        let text_style = match kind {
            Some(HunkKind::Added) => Style::default().fg(Color::LightGreen),
            Some(HunkKind::Modified) => Style::default().fg(Color::LightYellow),
            _ => Style::default(),
        };

        out.push(Line::from(vec![
            Span::styled(mark.to_string(), mark_style),
            Span::styled(format!(" {:>4} ", no), Style::default().fg(Color::DarkGray)),
            Span::styled(l.to_string(), text_style),
        ]));

        let n = removed_at(no);
        if n > 0 {
            out.push(removal_marker(n));
        }
    }

    out
}

fn removal_marker<'a>(n: usize) -> Line<'a> {
    let s = format!(
        "        ─── {} line{} removed ───",
        n,
        if n == 1 { "" } else { "s" }
    );
    Line::from(Span::styled(
        s,
        Style::default()
            .fg(Color::Red)
            .add_modifier(Modifier::DIM),
    ))
}

fn highlight_lines_old<'a>(text: &'a str, hunks: &[HunkSummary]) -> Vec<Line<'a>> {
    let mut out = Vec::new();

    let added_at = |line: usize| -> usize {
        hunks
            .iter()
            .filter(|h| h.kind == HunkKind::Added && h.old_start == line)
            .map(|h| h.added_count())
            .sum()
    };

    let n = added_at(0);
    if n > 0 {
        out.push(addition_marker(n));
    }

    for (i, l) in text.lines().enumerate() {
        let no = i + 1;
        let kind = hunks
            .iter()
            .find(|h| h.kind != HunkKind::Added && no >= h.old_start && no <= h.old_end)
            .map(|h| h.kind);

        let (mark, mark_style) = match kind {
            Some(HunkKind::Removed) => ("▎", Style::default().fg(Color::Red)),
            Some(HunkKind::Modified) => ("▎", Style::default().fg(Color::Yellow)),
            _ => (" ", Style::default()),
        };

        let text_style = match kind {
            Some(HunkKind::Removed) => Style::default().fg(Color::LightRed),
            Some(HunkKind::Modified) => Style::default().fg(Color::LightYellow),
            _ => Style::default(),
        };

        out.push(Line::from(vec![
            Span::styled(mark.to_string(), mark_style),
            Span::styled(format!(" {:>4} ", no), Style::default().fg(Color::DarkGray)),
            Span::styled(l.to_string(), text_style),
        ]));

        let n = added_at(no);
        if n > 0 {
            out.push(addition_marker(n));
        }
    }

    out
}

fn addition_marker<'a>(n: usize) -> Line<'a> {
    let s = format!(
        "        ─── {} line{} added ───",
        n,
        if n == 1 { "" } else { "s" }
    );
    Line::from(Span::styled(
        s,
        Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::DIM),
    ))
}

fn full_lines<'a>(diff_lines: &[DiffLine]) -> Vec<Line<'a>> {
    diff_lines
        .iter()
        .map(|dl| match dl {
            DiffLine::Equal {
                old_no,
                new_no,
                text,
            } => Line::from(vec![
                Span::styled(format!("{:>4} ", old_no), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:>4} ", new_no), Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                Span::raw(text.clone()),
            ]),
            DiffLine::Added { new_no, text } => Line::from(vec![
                Span::raw("     "),
                Span::styled(format!("{:>4} ", new_no), Style::default().fg(Color::DarkGray)),
                Span::styled("+ ", Style::default().fg(Color::Green)),
                Span::styled(text.clone(), Style::default().fg(Color::LightGreen)),
            ]),
            DiffLine::Removed { old_no, text } => Line::from(vec![
                Span::styled(format!("{:>4} ", old_no), Style::default().fg(Color::DarkGray)),
                Span::raw("     "),
                Span::styled("- ", Style::default().fg(Color::Red)),
                Span::styled(text.clone(), Style::default().fg(Color::LightRed)),
            ]),
        })
        .collect()
}
