//! Visual vocabulary for the "biomorphic organic" paper look.
//!
//! A single low-saturation green/cream palette drives every view. The
//! palette is adapted once at startup to the terminal's colour capability
//! (TrueColor → 256 → 16 → none), so a paper background never garbles an
//! older terminal. Layers and density characters express hierarchy:
//! [`Layer`] maps to background shades, [`density_char`] to fill levels.

use std::sync::OnceLock;

use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, BorderType, Borders};

/// TrueColor palette used when the terminal supports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub fg_primary: Color,
    pub fg_secondary: Color,
    pub fg_decorative: Color,
    pub bg_base: Color,
    pub bg_highlight: Color,
    pub bg_dark: Color,
    pub accent_dark: Color,
    pub accent_moss: Color,
    pub ink_line: Color,
    pub border: Color,
}

/// The full-resolution palette.
pub const PALETTE: Palette = Palette {
    fg_primary: Color::Rgb(44, 58, 34),
    fg_secondary: Color::Rgb(90, 107, 78),
    fg_decorative: Color::Rgb(141, 154, 130),
    bg_base: Color::Rgb(227, 233, 220),
    bg_highlight: Color::Rgb(244, 246, 239),
    bg_dark: Color::Rgb(216, 224, 208),
    accent_dark: Color::Rgb(61, 82, 48),
    accent_moss: Color::Rgb(107, 127, 94),
    ink_line: Color::Rgb(201, 211, 196),
    border: Color::Rgb(82, 96, 71),
};

/// Vertical hierarchy of a surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    /// The page itself (lightest background).
    Base,
    /// A raised panel / card.
    Raised,
    /// A floating modal above everything else.
    Floating,
}

impl Layer {
    /// The background colour for this layer, taken from the active palette.
    pub fn bg(self) -> Color {
        let p = active();
        match self {
            Layer::Base => p.bg_base,
            Layer::Raised => p.bg_highlight,
            Layer::Floating => p.bg_dark,
        }
    }
}

/// Map an emphasis `0.0..=1.0` onto the density ramp `· ░ ▓ █`.
pub fn density_char(level: f32) -> char {
    match level {
        l if l <= 0.25 => '·',
        l if l <= 0.5 => '░',
        l if l <= 0.75 => '▓',
        _ => '█',
    }
}

/// A `width`-wide line of a single density character.
pub fn density_line(level: f32, width: usize) -> String {
    density_char(level).to_string().repeat(width)
}

/// A rounded panel block with the layer's background and an optional
/// active (accent) border.
pub fn block(title: &str, layer: Layer, is_active: bool) -> Block<'static> {
    let p = active();
    let border_style = if is_active {
        Style::new().fg(p.accent_dark)
    } else {
        Style::new().fg(p.border)
    };
    Block::default()
        .title(title.to_string())
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::new().bg(layer.bg()))
        .border_style(border_style)
}

/// Terminal colour capability, detected once at startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorLevel {
    TrueColor,
    Ansi256,
    Ansi16,
    NoColor,
}

/// Detect the terminal's colour capability from the environment and the
/// attached stream. `NO_COLOR` is respected by `supports-color`.
pub fn detect_color_level() -> ColorLevel {
    use supports_color::Stream;
    match supports_color::on_cached(Stream::Stdout) {
        Some(support) if support.has_16m => ColorLevel::TrueColor,
        Some(support) if support.has_256 => ColorLevel::Ansi256,
        Some(_) => ColorLevel::Ansi16,
        None => ColorLevel::NoColor,
    }
}

/// Adapt a single colour to a terminal capability.
pub fn adapt(color: Color, level: ColorLevel) -> Color {
    match (color, level) {
        (Color::Rgb(r, g, b), ColorLevel::Ansi256) => Color::Indexed(nearest_ansi256(r, g, b)),
        (Color::Rgb(r, g, b), ColorLevel::Ansi16) => ansi16_color(nearest_ansi16(r, g, b)),
        (_, ColorLevel::NoColor) => Color::Reset,
        (other, _) => other,
    }
}

fn adapt_palette(p: Palette, level: ColorLevel) -> Palette {
    Palette {
        fg_primary: adapt(p.fg_primary, level),
        fg_secondary: adapt(p.fg_secondary, level),
        fg_decorative: adapt(p.fg_decorative, level),
        bg_base: adapt(p.bg_base, level),
        bg_highlight: adapt(p.bg_highlight, level),
        bg_dark: adapt(p.bg_dark, level),
        accent_dark: adapt(p.accent_dark, level),
        accent_moss: adapt(p.accent_moss, level),
        ink_line: adapt(p.ink_line, level),
        border: adapt(p.border, level),
    }
}

static ACTIVE: OnceLock<Palette> = OnceLock::new();

/// Pin the palette to the detected terminal capability. Called once at
/// TUI startup; tests simply keep the TrueColor palette.
pub fn init_palette(level: ColorLevel) {
    let _ = ACTIVE.set(adapt_palette(PALETTE, level));
}

/// The palette in effect (TrueColor unless [`init_palette`] was called).
pub fn active() -> &'static Palette {
    ACTIVE.get().unwrap_or(&PALETTE)
}

/// The page background, used to paint the whole frame.
pub fn base_bg() -> Color {
    active().bg_base
}

// -- style helpers ---------------------------------------------------------

pub fn fg() -> Style {
    Style::new().fg(active().fg_primary)
}

pub fn dim() -> Style {
    Style::new().fg(active().fg_secondary)
}

pub fn decor() -> Style {
    Style::new().fg(active().fg_decorative)
}

pub fn accent() -> Style {
    Style::new().fg(active().accent_moss)
}

pub fn accent_bold() -> Style {
    Style::new()
        .fg(active().accent_dark)
        .add_modifier(Modifier::BOLD)
}

/// Solid dark-green fill with the light paper text on top.
pub fn on_accent() -> Style {
    Style::new()
        .fg(active().bg_highlight)
        .bg(active().accent_dark)
}

/// Outlined (reverse) accent, for secondary buttons/tabs.
pub fn outlined() -> Style {
    Style::new()
        .fg(active().accent_dark)
        .add_modifier(Modifier::REVERSED)
}

/// The selected/active row.
pub fn selected() -> Style {
    on_accent()
}

pub fn success() -> Style {
    Style::new().fg(active().accent_moss)
}

pub fn error() -> Style {
    Style::new().fg(Color::Red).add_modifier(Modifier::BOLD)
}

pub fn border() -> Style {
    Style::new().fg(active().border)
}

pub fn border_active() -> Style {
    Style::new().fg(active().accent_dark)
}

// -- colour mapping --------------------------------------------------------

fn dist2(r: u8, g: u8, b: u8, cr: u16, cg: u16, cb: u16) -> u32 {
    let dr = r as i32 - cr as i32;
    let dg = g as i32 - cg as i32;
    let db = b as i32 - cb as i32;
    (dr * dr + dg * dg + db * db) as u32
}

/// Nearest index into the xterm 256-colour palette.
fn nearest_ansi256(r: u8, g: u8, b: u8) -> u8 {
    let mut best = 0u8;
    let mut best_d = u32::MAX;
    let mut consider = |index: u8, cr: u16, cg: u16, cb: u16| {
        let d = dist2(r, g, b, cr, cg, cb);
        if d < best_d {
            best_d = d;
            best = index;
        }
    };
    for r6 in 0..6u16 {
        for g6 in 0..6u16 {
            for b6 in 0..6u16 {
                consider(
                    16 + (36 * r6 + 6 * g6 + b6) as u8,
                    r6 * 51,
                    g6 * 51,
                    b6 * 51,
                );
            }
        }
    }
    for i in 0..24u16 {
        let v = 8 + i * 10;
        consider((232 + i) as u8, v, v, v);
    }
    best
}

/// Nearest index into the 16-colour ANSI palette.
fn nearest_ansi16(r: u8, g: u8, b: u8) -> u8 {
    const ANSI: [(u16, u16, u16); 16] = [
        (0, 0, 0),
        (128, 0, 0),
        (0, 128, 0),
        (128, 128, 0),
        (0, 0, 128),
        (128, 0, 128),
        (0, 128, 128),
        (192, 192, 192),
        (128, 128, 128),
        (255, 0, 0),
        (0, 255, 0),
        (255, 255, 0),
        (0, 0, 255),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ];
    ANSI.iter()
        .enumerate()
        .min_by_key(|(_, (cr, cg, cb))| dist2(r, g, b, *cr, *cg, *cb))
        .map(|(i, _)| i as u8)
        .unwrap_or(0)
}

fn ansi16_color(index: u8) -> Color {
    const MAP: [Color; 16] = [
        Color::Black,
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::Gray,
        Color::DarkGray,
        Color::LightRed,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightBlue,
        Color::LightMagenta,
        Color::LightCyan,
        Color::White,
    ];
    MAP[index as usize]
}
