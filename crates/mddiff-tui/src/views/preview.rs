use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::markdown;

pub struct PreviewView {
    scroll: u16,
}

impl PreviewView {
    pub fn new() -> Self {
        Self { scroll: 0 }
    }

    pub fn scroll_up(&mut self, n: u16) {
        self.scroll = self.scroll.saturating_sub(n);
    }

    pub fn scroll_down(&mut self, n: u16) {
        self.scroll = self.scroll.saturating_add(n);
    }

    pub fn render(&self, frame: &mut Frame<'_>, area: Rect, source: &str) {
        let text = markdown::render(source);
        let para = Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0))
            .block(Block::default().borders(Borders::ALL).title(" Preview "));
        frame.render_widget(para, area);
    }
}
