use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use ratatui::Terminal;

use crate::views::preview::PreviewView;
use crate::views::source::SourceView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Source,
    Preview,
}

impl Mode {
    fn next(self) -> Self {
        match self {
            Mode::Source => Mode::Preview,
            Mode::Preview => Mode::Source,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Mode::Source => "Source",
            Mode::Preview => "Preview",
        }
    }
}

pub struct App {
    pub mode: Mode,
    pub path: Option<PathBuf>,
    pub source: SourceView,
    pub preview: PreviewView,
    pub read_only: bool,
    pub saved_text: String,
    pub status: Option<String>,
}

impl App {
    pub fn new(initial_text: String, path: Option<PathBuf>, mode: Mode, read_only: bool) -> Self {
        Self {
            mode,
            path,
            source: SourceView::new(&initial_text, read_only),
            preview: PreviewView::new(),
            read_only,
            saved_text: initial_text,
            status: None,
        }
    }

    pub fn dirty(&self) -> bool {
        self.source.text() != self.saved_text
    }

    pub fn run<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if self.handle_key(key)? {
                    break;
                }
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        if ctrl {
            match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('s') => {
                    self.save();
                    return Ok(false);
                }
                KeyCode::Char('e') => {
                    self.mode = self.mode.next();
                    return Ok(false);
                }
                _ => {}
            }
        }

        match self.mode {
            Mode::Source => {
                self.source.handle_key(key);
            }
            Mode::Preview => match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('s') => self.save(),
                KeyCode::Char('j') | KeyCode::Down => self.preview.scroll_down(1),
                KeyCode::Char('k') | KeyCode::Up => self.preview.scroll_up(1),
                KeyCode::PageDown | KeyCode::Char(' ') => self.preview.scroll_down(10),
                KeyCode::PageUp => self.preview.scroll_up(10),
                KeyCode::Tab => self.mode = self.mode.next(),
                _ => {}
            },
        }
        Ok(false)
    }

    fn save(&mut self) {
        if self.read_only {
            self.status = Some("read-only".into());
            return;
        }
        let Some(path) = self.path.clone() else {
            self.status = Some("no file to save (open with `mdv-tui <path>`)".into());
            return;
        };
        let text = self.source.text();
        match mdv_core::fs::write_text_file(&path, &text) {
            Ok(_) => {
                self.saved_text = text;
                self.status = Some(format!("saved: {}", path.display()));
            }
            Err(e) => self.status = Some(format!("save failed: {e}")),
        }
    }

    fn draw(&self, frame: &mut ratatui::Frame<'_>) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

        self.draw_header(frame, chunks[0]);
        match self.mode {
            Mode::Source => self.source.render(frame, chunks[1]),
            Mode::Preview => self.preview.render(frame, chunks[1], &self.source.text()),
        }
        self.draw_footer(frame, chunks[2]);
    }

    fn draw_header(&self, frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect) {
        let file = match &self.path {
            Some(p) => p.display().to_string(),
            None => "(untitled)".into(),
        };
        let dirty = if self.dirty() { " ●" } else { "" };
        let header = format!(" mdv-tui  {}{}", file, dirty);
        frame.render_widget(
            Paragraph::new(header).style(Style::default().fg(Color::Cyan)),
            area,
        );
    }

    fn draw_footer(&self, frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect) {
        let (row, col) = self.source.cursor();
        let status_str = match (&self.status, self.mode) {
            (Some(s), _) => s.clone(),
            (None, Mode::Source) => format!(
                " [{}] Ln {}, Col {}   ^S save  ^E mode  ^Q quit",
                self.mode.label(),
                row,
                col,
            ),
            (None, Mode::Preview) => format!(
                " [{}]   j/k scroll  s save  Tab mode  q quit",
                self.mode.label(),
            ),
        };
        frame.render_widget(
            Paragraph::new(status_str).style(Style::default().fg(Color::DarkGray)),
            area,
        );
    }
}
