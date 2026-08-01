//! Visual vocabulary shared across views.
//!
//! Restraint-first: one accent color from the built-in Tailwind palette,
//! a gray scale for everything else, rounded borders only where a visual
//! grouping is actually needed, and no hard background color so the
//! terminal theme is respected (readable on both light and dark).

use ratatui::style::palette::tailwind::{INDIGO, SLATE};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::BorderType;

/// Single accent color.
pub const ACCENT: Color = INDIGO.c500;
/// Default foreground (mid-gray, readable on light and dark terminals).
pub const FG: Color = SLATE.c600;
/// De-emphasized foreground.
pub const FG_DIM: Color = SLATE.c500;

/// Unified rounded border everywhere we draw a frame.
pub const BORDER: BorderType = BorderType::Rounded;

pub fn accent() -> Style {
    Style::new().fg(ACCENT)
}

pub fn fg() -> Style {
    Style::new().fg(FG)
}

pub fn dim() -> Style {
    Style::new().fg(FG_DIM)
}

pub fn bold() -> Style {
    Style::new().add_modifier(Modifier::BOLD)
}

pub fn accent_bold() -> Style {
    Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)
}

/// Selected/active row: reverse video keeps contrast on any theme.
pub fn selected() -> Style {
    Style::new().fg(ACCENT).add_modifier(Modifier::REVERSED)
}

pub fn border() -> Style {
    Style::new().fg(FG_DIM)
}

pub fn border_active() -> Style {
    Style::new().fg(ACCENT)
}

pub fn success() -> Style {
    Style::new().fg(Color::Green)
}

pub fn error() -> Style {
    Style::new().fg(Color::Red).add_modifier(Modifier::BOLD)
}
