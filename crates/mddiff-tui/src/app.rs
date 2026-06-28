use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use mddiff_core::diff::DiffLine;
use mddiff_core::git::SideBySidePayload;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use ratatui::Terminal;

use crate::picker::BasePicker;
use crate::views::diff::DiffView;
use crate::views::preview::PreviewView;
use crate::views::source::SourceView;

/// Per-(text, base) memoization of the Diff backends. Without this the
/// terminal redraws every keystroke would re-run git2 + similar on every
/// frame; for non-trivial docs that adds visible lag.
#[derive(Default)]
struct DiffCache {
    text_hash: u64,
    base: String,
    sbs: Option<Result<SideBySidePayload, String>>,
    full: Option<Result<Vec<DiffLine>, String>>,
}

impl DiffCache {
    fn invalidate_if_stale(&mut self, text_hash: u64, base: &str) {
        if self.text_hash != text_hash || self.base != base {
            self.text_hash = text_hash;
            self.base = base.to_string();
            self.sbs = None;
            self.full = None;
        }
    }
}

fn hash_str(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Source,
    Preview,
    Diff,
}

impl Mode {
    fn label(self) -> &'static str {
        match self {
            Mode::Source => "Source",
            Mode::Preview => "Preview",
            Mode::Diff => "Diff",
        }
    }

    fn next(self, git_available: bool) -> Self {
        match self {
            Mode::Source => Mode::Preview,
            Mode::Preview => {
                if git_available {
                    Mode::Diff
                } else {
                    Mode::Source
                }
            }
            Mode::Diff => Mode::Source,
        }
    }
}

pub struct App {
    pub mode: Mode,
    pub path: Option<PathBuf>,
    pub source: SourceView,
    pub preview: PreviewView,
    pub diff: DiffView,
    pub read_only: bool,
    pub git_available: bool,
    pub diff_base: String,
    pub saved_text: String,
    pub status: Option<String>,
    pub picker: Option<BasePicker>,
    diff_cache: DiffCache,
    /// `Some(buf)` while the user is typing a `:command`. `buf` doesn't
    /// include the leading colon.
    pub command: Option<String>,
}

impl App {
    pub fn new(
        initial_text: String,
        path: Option<PathBuf>,
        mode: Mode,
        read_only: bool,
        git_available: bool,
        diff_base: String,
    ) -> Self {
        Self {
            mode: if mode == Mode::Diff && !git_available {
                Mode::Source
            } else {
                mode
            },
            path,
            source: SourceView::new(&initial_text, read_only),
            preview: PreviewView::new(),
            diff: DiffView::new(),
            read_only,
            git_available,
            diff_base,
            saved_text: initial_text,
            status: None,
            picker: None,
            diff_cache: DiffCache::default(),
            command: None,
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
            // draw mutated picker state (ListState); nothing else to do.

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
        // Picker captures all input while open.
        if self.picker.is_some() {
            let picker = self.picker.as_mut().unwrap();
            match key.code {
                KeyCode::Esc => self.picker = None,
                KeyCode::Up | KeyCode::Char('k') => picker.up(),
                KeyCode::Down | KeyCode::Char('j') => picker.down(),
                KeyCode::Enter => {
                    if let Some(opt) = picker.current() {
                        self.diff_base = opt.revspec.clone();
                        self.status = Some(format!("compare base: {}", self.diff_base));
                        if self.mode != Mode::Diff && self.git_available {
                            self.mode = Mode::Diff;
                        }
                    }
                    self.picker = None;
                }
                _ => {}
            }
            return Ok(false);
        }

        // Command mode (`:w` / `:q` / `:wq` / `:q!`) captures all input while open.
        if self.command.is_some() {
            return self.handle_command_key(key);
        }

        // Esc enters command mode from any view (Source/Preview/Diff). tui-textarea
        // ignores Esc by default, so this doesn't conflict with editing.
        if key.code == KeyCode::Esc && !key.modifiers.contains(KeyModifiers::SHIFT) {
            self.command = Some(String::new());
            self.status = None;
            return Ok(false);
        }

        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        if ctrl {
            match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('s') => {
                    self.save();
                    return Ok(false);
                }
                KeyCode::Char('e') => {
                    self.mode = self.mode.next(self.git_available);
                    return Ok(false);
                }
                KeyCode::Char('d') if self.mode == Mode::Diff => {
                    self.diff.toggle_submode();
                    return Ok(false);
                }
                KeyCode::Char('b') if self.git_available => {
                    self.open_picker();
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
                KeyCode::Tab => self.mode = self.mode.next(self.git_available),
                KeyCode::Char('b') if self.git_available => self.open_picker(),
                _ => {}
            },
            Mode::Diff => match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('j') | KeyCode::Down => self.diff.scroll_down(1),
                KeyCode::Char('k') | KeyCode::Up => self.diff.scroll_up(1),
                KeyCode::PageDown | KeyCode::Char(' ') => self.diff.scroll_down(10),
                KeyCode::PageUp => self.diff.scroll_up(10),
                KeyCode::Char('d') | KeyCode::Tab => self.diff.toggle_submode(),
                KeyCode::Char('e') => self.mode = self.mode.next(self.git_available),
                KeyCode::Char('b') => self.open_picker(),
                _ => {}
            },
        }
        Ok(false)
    }

    fn open_picker(&mut self) {
        if let Some(path) = &self.path {
            self.picker = Some(BasePicker::open(path, &self.source.text()));
        }
    }

    fn handle_command_key(&mut self, key: KeyEvent) -> Result<bool> {
        let buf = self.command.as_mut().expect("command mode active");
        match key.code {
            KeyCode::Esc => {
                self.command = None;
            }
            KeyCode::Enter => {
                let cmd = std::mem::take(buf);
                self.command = None;
                return Ok(self.execute_command(&cmd));
            }
            KeyCode::Backspace => {
                buf.pop();
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                buf.push(c);
            }
            _ => {}
        }
        Ok(false)
    }

    /// Returns true if the app should exit.
    fn execute_command(&mut self, raw: &str) -> bool {
        match raw.trim() {
            "" => false,
            "w" => {
                self.save();
                false
            }
            "q" => {
                if self.dirty() {
                    self.status = Some(
                        "no write since last change (use :wq to save or :q! to discard)".into(),
                    );
                    false
                } else {
                    true
                }
            }
            "wq" | "x" => {
                self.save();
                true
            }
            "q!" => true,
            other => {
                self.status = Some(format!("unknown command: :{}", other));
                false
            }
        }
    }

    fn save(&mut self) {
        if self.read_only {
            self.status = Some("read-only".into());
            return;
        }
        let Some(path) = self.path.clone() else {
            self.status = Some("no file to save (open with `mddiff-tui <path>`)".into());
            return;
        };
        let text = self.source.text();
        match mddiff_core::fs::write_text_file(&path, &text) {
            Ok(_) => {
                self.saved_text = text;
                self.status = Some(format!("saved: {}", path.display()));
            }
            Err(e) => self.status = Some(format!("save failed: {e}")),
        }
    }

    fn draw(&mut self, frame: &mut ratatui::Frame<'_>) {
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
            Mode::Diff => self.render_diff(frame, chunks[1]),
        }
        self.draw_footer(frame, chunks[2]);

        if let Some(picker) = self.picker.as_mut() {
            picker.render(frame, area);
        }
    }

    fn render_diff(&mut self, frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect) {
        use crate::views::diff::Submode;
        let Some(path) = self.path.clone() else {
            self.diff.render_message(frame, area, "No file open.");
            return;
        };
        if !self.git_available {
            self.diff
                .render_message(frame, area, "This file is not in a Git repository.");
            return;
        }
        let text = self.source.text();
        self.diff_cache
            .invalidate_if_stale(hash_str(&text), &self.diff_base);

        match self.diff.submode {
            // Highlight and SideBySide share the SBS payload (old_text + hunks).
            Submode::Highlight | Submode::SideBySide => {
                if self.diff_cache.sbs.is_none() {
                    self.diff_cache.sbs = Some(
                        mddiff_core::git::side_by_side_against_base(&path, &text, &self.diff_base)
                            .map_err(|e| e.to_string()),
                    );
                }
                match (self.diff.submode, self.diff_cache.sbs.as_ref().unwrap()) {
                    (Submode::Highlight, Ok(sbs)) => {
                        self.diff.render_highlight(frame, area, &text, &sbs.hunks)
                    }
                    (Submode::SideBySide, Ok(sbs)) => {
                        self.diff.render_sidebyside(frame, area, sbs, &self.diff_base)
                    }
                    (_, Err(e)) => self.diff.render_message(frame, area, e),
                    _ => unreachable!(),
                }
            }
            Submode::Full => {
                if self.diff_cache.full.is_none() {
                    self.diff_cache.full = Some(
                        mddiff_core::git::full_diff_against_base(&path, &text, &self.diff_base)
                            .map_err(|e| e.to_string()),
                    );
                }
                match self.diff_cache.full.as_ref().unwrap() {
                    Ok(lines) => self.diff.render_full(frame, area, lines),
                    Err(e) => self.diff.render_message(frame, area, e),
                }
            }
        }
    }

    fn draw_header(&self, frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect) {
        let file = match &self.path {
            Some(p) => p.display().to_string(),
            None => "(untitled)".into(),
        };
        let dirty = if self.dirty() { " ●" } else { "" };
        let header = format!(" mddiff-tui  {}{}", file, dirty);
        frame.render_widget(
            Paragraph::new(header).style(Style::default().fg(Color::Cyan)),
            area,
        );
    }

    fn draw_footer(&self, frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect) {
        // Command mode takes over the footer line with a `:` prompt.
        if let Some(buf) = &self.command {
            let prompt = format!(":{}", buf);
            frame.render_widget(
                Paragraph::new(prompt.clone()).style(Style::default().fg(Color::Yellow)),
                area,
            );
            // Position the terminal cursor right after the typed text so the
            // user can see where they are.
            let x = area.x + 1 + buf.chars().count() as u16;
            frame.set_cursor_position((x.min(area.x + area.width.saturating_sub(1)), area.y));
            return;
        }

        let (row, col) = self.source.cursor();
        let status_str = match (&self.status, self.mode) {
            (Some(s), _) => s.clone(),
            (None, Mode::Source) => format!(
                " [{}] Ln {}, Col {}   ^S save  ^E mode  ^Q quit  Esc :cmd",
                self.mode.label(),
                row,
                col,
            ),
            (None, Mode::Preview) => format!(
                " [{}]   j/k scroll  s save  Tab mode  b base  q quit  Esc :cmd",
                self.mode.label(),
            ),
            (None, Mode::Diff) => format!(
                " [Diff · {}]  vs {}   j/k scroll  Tab/^D submode  b base  q quit  Esc :cmd",
                self.diff.submode.label(),
                self.diff_base,
            ),
        };
        frame.render_widget(
            Paragraph::new(status_str).style(Style::default().fg(Color::DarkGray)),
            area,
        );
    }
}
