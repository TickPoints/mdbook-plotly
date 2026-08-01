//! The TUI application: a single `mdbook-plotly tui` entry point hosting
//! three tools (self-update, book.toml editor, plot generator) plus a
//! welcome view and a help overlay.
//!
//! Network work runs on background threads and reports progress over
//! channels, so the UI never blocks. Effects (fade-in on view change, a
//! short sweep on update completion) come from `tachyonfx` and can be
//! disabled with `--no-effects`.

pub mod app;
pub mod book_toml;
pub mod book_toml_view;
pub mod github;
pub mod help;
pub mod home;
pub mod locale;
pub mod plotgen;
pub mod plotgen_view;
pub mod settings;
pub mod term;
pub mod theme;
pub mod update;
pub mod update_view;
pub mod widget;

use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

/// Event poll interval in milliseconds.
pub const EVENT_POLL_MS: u64 = 50;

/// View-transition fade-in duration in milliseconds.
pub const VIEW_FADE_MS: u32 = 160;
/// Update-complete sweep duration in milliseconds.
pub const UPDATE_SWEEP_MS: u32 = 180;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Home,
    Update,
    Config,
    PlotGen,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TuiOptions {
    pub dry_run: bool,
    pub no_effects: bool,
    pub no_preview: bool,
}

/// Run the TUI until the user quits.
pub fn run(opts: TuiOptions) -> anyhow::Result<()> {
    term::install_panic_hook();
    let mut terminal = term::setup()?;
    let mut app = app::App::new(opts);
    let result = app.run(&mut terminal);
    let _ = term::teardown(&mut terminal);
    if let Some(text) = app.take_gen_output() {
        println!("{text}");
    }
    result.map_err(Into::into)
}

/// Helper type alias for the terminal used across the app.
pub type TerminalBackend = CrosstermBackend<std::io::Stdout>;
pub type TerminalType = Terminal<TerminalBackend>;
