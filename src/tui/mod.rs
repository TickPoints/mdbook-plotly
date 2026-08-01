//! The TUI application: a single `mdbook-plotly tui` entry point hosting
//! three tools (self-update, book.toml editor, plot cheat-sheet) plus a
//! welcome view and a help overlay.
//!
//! Network work runs on background threads and reports progress over
//! channels, so the UI never blocks. Effects (fade-in on view change, a
//! short sweep on update completion) come from `tachyonfx` and can be
//! disabled with `--no-effects`.

pub mod github;
pub mod locale;
pub mod settings;
pub mod term;
pub mod theme;
pub mod widget;

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
    CheatSheet,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TuiOptions {
    pub dry_run: bool,
    pub refresh: bool,
    pub no_effects: bool,
}
