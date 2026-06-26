use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};

pub fn render<'a>(source: &'a str) -> Text<'a> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let mut lines: Vec<Line<'a>> = Vec::new();
    let mut current: Vec<Span<'a>> = Vec::new();
    let mut style = Style::default();
    let mut list_depth: usize = 0;
    let mut in_code_block = false;
    let mut blockquote = false;

    fn flush<'a>(lines: &mut Vec<Line<'a>>, current: &mut Vec<Span<'a>>) {
        let spans = std::mem::take(current);
        lines.push(Line::from(spans));
    }

    for event in Parser::new_ext(source, options) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                if !current.is_empty() {
                    flush(&mut lines, &mut current);
                }
                if !lines.is_empty() {
                    lines.push(Line::from(""));
                }
                style = heading_style(level);
            }
            Event::End(TagEnd::Heading(_)) => {
                flush(&mut lines, &mut current);
                lines.push(Line::from(""));
                style = Style::default();
            }
            Event::Start(Tag::Paragraph) => {
                if !current.is_empty() {
                    flush(&mut lines, &mut current);
                }
            }
            Event::End(TagEnd::Paragraph) => {
                flush(&mut lines, &mut current);
                lines.push(Line::from(""));
            }
            Event::Start(Tag::Emphasis) => style = style.add_modifier(Modifier::ITALIC),
            Event::End(TagEnd::Emphasis) => style = style.remove_modifier(Modifier::ITALIC),
            Event::Start(Tag::Strong) => style = style.add_modifier(Modifier::BOLD),
            Event::End(TagEnd::Strong) => style = style.remove_modifier(Modifier::BOLD),
            Event::Start(Tag::Strikethrough) => {
                style = style.add_modifier(Modifier::CROSSED_OUT);
            }
            Event::End(TagEnd::Strikethrough) => {
                style = style.remove_modifier(Modifier::CROSSED_OUT);
            }
            Event::Start(Tag::BlockQuote(_)) => blockquote = true,
            Event::End(TagEnd::BlockQuote(_)) => blockquote = false,
            Event::Start(Tag::CodeBlock(kind)) => {
                if !current.is_empty() {
                    flush(&mut lines, &mut current);
                }
                in_code_block = true;
                let label = match kind {
                    CodeBlockKind::Fenced(s) if !s.is_empty() => format!("─── {} ───", s),
                    _ => "──────".to_string(),
                };
                lines.push(Line::from(Span::styled(
                    label,
                    Style::default().fg(Color::DarkGray),
                )));
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                lines.push(Line::from(Span::styled(
                    "──────",
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from(""));
            }
            Event::Start(Tag::List(_)) => {
                list_depth += 1;
                if !current.is_empty() {
                    flush(&mut lines, &mut current);
                }
            }
            Event::End(TagEnd::List(_)) => {
                if list_depth > 0 {
                    list_depth -= 1;
                }
                if list_depth == 0 {
                    lines.push(Line::from(""));
                }
            }
            Event::Start(Tag::Item) => {
                let indent = "  ".repeat(list_depth.saturating_sub(1));
                current.push(Span::raw(format!("{}• ", indent)));
            }
            Event::End(TagEnd::Item) => {
                flush(&mut lines, &mut current);
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                let link_style = Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::UNDERLINED);
                style = link_style;
                let _ = dest_url;
            }
            Event::End(TagEnd::Link) => {
                style = Style::default();
            }
            Event::Code(t) => {
                let s = Style::default().fg(Color::LightYellow);
                current.push(Span::styled(t.to_string(), s));
            }
            Event::Text(t) => {
                let prefix = if blockquote && current.is_empty() {
                    "│ "
                } else {
                    ""
                };
                if in_code_block {
                    for line in t.lines() {
                        lines.push(Line::from(Span::styled(
                            format!("  {}", line),
                            Style::default().fg(Color::Gray),
                        )));
                    }
                    if t.ends_with('\n') {
                        // already flushed per-line
                    }
                } else {
                    if !prefix.is_empty() {
                        current.push(Span::styled(
                            prefix.to_string(),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }
                    current.push(Span::styled(t.to_string(), style));
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                flush(&mut lines, &mut current);
            }
            Event::Rule => {
                if !current.is_empty() {
                    flush(&mut lines, &mut current);
                }
                lines.push(Line::from(Span::styled(
                    "──────────",
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from(""));
            }
            _ => {}
        }
    }

    if !current.is_empty() {
        flush(&mut lines, &mut current);
    }

    Text::from(lines)
}

fn heading_style(level: HeadingLevel) -> Style {
    let base = Style::default().add_modifier(Modifier::BOLD);
    match level {
        HeadingLevel::H1 => base.fg(Color::Cyan),
        HeadingLevel::H2 => base.fg(Color::LightCyan),
        HeadingLevel::H3 => base.fg(Color::Yellow),
        _ => base.fg(Color::LightYellow),
    }
}
