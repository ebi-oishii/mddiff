mod app;
mod markdown;
mod views;

use std::io;
use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, ValueEnum};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::{App, Mode};

#[derive(Parser, Debug)]
#[command(name = "mdv-tui", version, about = "Terminal UI for mdv")]
struct Args {
    /// Markdown file to open.
    file: Option<PathBuf>,
    /// Initial mode.
    #[arg(long, value_enum, default_value = "source")]
    mode: ModeArg,
    /// Open as read-only.
    #[arg(long)]
    read_only: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ModeArg {
    Source,
    Preview,
}

impl From<ModeArg> for Mode {
    fn from(m: ModeArg) -> Self {
        match m {
            ModeArg::Source => Mode::Source,
            ModeArg::Preview => Mode::Preview,
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let (initial_text, path) = match &args.file {
        Some(p) => {
            let text = mdv_core::fs::read_text_file(p)?;
            (text, Some(p.clone()))
        }
        None => (String::new(), None),
    };

    let mut app = App::new(initial_text, path, args.mode.into(), args.read_only);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}
