mod app;
mod markdown;
mod picker;
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
    /// Git revision to compare against in Diff mode (e.g. HEAD, HEAD~1, main, <sha>).
    #[arg(long, default_value = "HEAD")]
    diff_base: String,
    /// Open files larger than 5MB anyway. Above 100MB the file is still
    /// refused — the WebView / TUI line wrapping degrade past that point.
    #[arg(long)]
    force: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ModeArg {
    Source,
    Preview,
    Diff,
}

impl From<ModeArg> for Mode {
    fn from(m: ModeArg) -> Self {
        match m {
            ModeArg::Source => Mode::Source,
            ModeArg::Preview => Mode::Preview,
            ModeArg::Diff => Mode::Diff,
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let (initial_text, path) = match &args.file {
        Some(p) => {
            let text = match mdv_core::fs::read_text_file_with(p, args.force) {
                Ok(t) => t,
                Err(mdv_core::fs::ReadError::TooLarge { actual, limit }) => {
                    anyhow::bail!(
                        "{p:?} is {} bytes (limit {limit}); re-run with --force to open anyway.",
                        actual
                    );
                }
                Err(e) => return Err(e.into()),
            };
            // Canonicalize so git2 can correctly relate this path to the discovered repo workdir.
            let abs = std::fs::canonicalize(p).unwrap_or_else(|_| p.clone());
            (text, Some(abs))
        }
        None => (String::new(), None),
    };

    let git_available = path
        .as_ref()
        .map(|p| mdv_core::git::is_in_repo(p))
        .unwrap_or(false);

    let mut app = App::new(
        initial_text,
        path,
        args.mode.into(),
        args.read_only,
        git_available,
        args.diff_base,
    );

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
