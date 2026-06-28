use crossterm::event::KeyEvent;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;
use tui_textarea::{Input, TextArea};

pub struct SourceView {
    textarea: TextArea<'static>,
    read_only: bool,
}

impl SourceView {
    pub fn new(initial: &str, read_only: bool) -> Self {
        let mut textarea = TextArea::from(initial.lines());
        textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
        if read_only {
            textarea.set_cursor_line_style(Style::default());
        }
        Self {
            textarea,
            read_only,
        }
    }

    pub fn text(&self) -> String {
        self.textarea.lines().join("\n")
    }

    pub fn cursor(&self) -> (usize, usize) {
        let (row, col) = self.textarea.cursor();
        (row + 1, col + 1)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.read_only {
            return false;
        }
        let input: Input = key.into();
        self.textarea.input(input)
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect) {
        let mut ta = self.textarea.clone();
        let title = if self.read_only {
            " Source [read-only] "
        } else {
            " Source "
        };
        ta.set_block(Block::default().borders(Borders::ALL).title(title));
        frame.render_widget(&ta, area);
    }
}
