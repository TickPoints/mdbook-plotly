//! Programmatic visual regression tests: render key views through
//! `ratatui::TestBackend` and assert structure, labels, and the paper
//! palette, plus direct checks of the color-adaptation helpers.

#![cfg(feature = "tui")]

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;

use mdbook_plotly::tui::home::HomeView;
use mdbook_plotly::tui::locale::DocLang;
use mdbook_plotly::tui::plotgen::PlotGen;
use mdbook_plotly::tui::plotgen_view::PlotGenView;
use mdbook_plotly::tui::theme;

/// Render a view into a fresh buffer.
fn render<F>(width: u16, height: u16, f: F) -> Buffer
where
    F: FnOnce(&mut ratatui::Frame, Rect),
{
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            let area = frame.area();
            f(frame, area);
        })
        .unwrap();
    terminal.backend().buffer().clone()
}

/// True when the buffer contains a row with `needle` as a contiguous run.
fn has_text(buffer: &Buffer, needle: &str) -> bool {
    for y in 0..buffer.area.height {
        let mut row = String::new();
        for x in 0..buffer.area.width {
            row.push_str(buffer[(x, y)].symbol());
        }
        if row.contains(needle) {
            return true;
        }
    }
    false
}

/// True when some cell's symbol is `needle` (wide chars occupy a single
/// cell plus an empty continuation, so this handles CJK).
fn has_cell(buffer: &Buffer, needle: char) -> bool {
    buffer
        .content()
        .iter()
        .any(|cell| cell.symbol() == needle.to_string())
}

/// True when every cell in the row has a background set from the paper
/// palette (the key bar band or one of its accent pills).
fn row_fully_painted(buffer: &Buffer, y: u16) -> bool {
    let p = theme::active();
    (0..buffer.area.width).all(|x| {
        matches!(
            buffer[(x, y)].style().bg,
            Some(color) if color == p.bg_dark || color == p.accent_dark
        )
    })
}

#[test]
fn home_renders_subtitle_and_tools() {
    let buffer = render(100, 40, |frame, area| {
        HomeView::default().render(frame, area);
    });
    assert!(has_text(&buffer, "FULL EDITION"), "subtitle missing");
    assert!(has_text(&buffer, "Self-update"));
    assert!(has_text(&buffer, "Plot generator"));
    assert!(has_text(&buffer, env!("CARGO_PKG_VERSION")));
}

#[test]
fn plotgen_renders_form_preview_and_actions() {
    let view = PlotGenView::new(PlotGen::new(DocLang::English));
    let buffer = render(140, 40, |frame, area| {
        let mut view = view.clone();
        view.render(frame, area);
    });
    assert!(has_text(&buffer, "Plot generator"), "header missing");
    assert!(has_text(&buffer, " Line "), "selected type tab missing");
    assert!(has_text(&buffer, "Chart title"), "field label missing");
    assert!(has_text(&buffer, "preview (JSON)"), "preview block missing");
    assert!(
        has_text(&buffer, "\"title\": \"Basic Line\""),
        "live preview content missing"
    );
    assert!(has_text(&buffer, "copy"), "action bar missing");
}

#[test]
fn plotgen_chinese_view_shows_localized_labels() {
    let view = PlotGenView::new(PlotGen::new(DocLang::Chinese));
    let buffer = render(140, 40, |frame, area| {
        let mut view = view.clone();
        view.render(frame, area);
    });
    assert!(has_cell(&buffer, '折'), "Chinese type label missing");
    assert!(has_cell(&buffer, '图'), "Chinese type label missing");
    assert!(has_cell(&buffer, '题'), "Chinese field label missing");
}

#[test]
fn key_bar_uses_the_paper_band() {
    let buffer = render(80, 10, |frame, area| {
        let band = Rect {
            x: area.x,
            y: area.bottom().saturating_sub(1),
            width: area.width,
            height: 1,
        };
        mdbook_plotly::tui::widget::key_bar(frame, band, &[("Esc", "home"), ("q", "quit")]);
    });
    assert!(
        row_fully_painted(&buffer, 9),
        "status band must be fully painted in the paper palette"
    );
    assert!(has_text(&buffer, "Esc"));
    assert!(has_text(&buffer, "quit"));
    assert!(
        buffer.content().iter().any(
            |cell| cell.symbol() == "E" && cell.style().bg == Some(theme::active().accent_dark)
        ),
        "the 'Esc' pill must use the accent fill"
    );
}

#[test]
fn density_char_maps_emphasis_to_ramp() {
    assert_eq!(theme::density_char(0.0), '·');
    assert_eq!(theme::density_char(0.25), '·');
    assert_eq!(theme::density_char(0.4), '░');
    assert_eq!(theme::density_char(0.6), '▓');
    assert_eq!(theme::density_char(1.0), '█');
    assert_eq!(theme::density_line(1.0, 3), "███");
}

#[test]
fn adapt_preserves_truecolor_and_degrades() {
    let rgb = Color::Rgb(44, 58, 34);
    assert_eq!(theme::adapt(rgb, theme::ColorLevel::TrueColor), rgb);
    assert_eq!(theme::adapt(rgb, theme::ColorLevel::NoColor), Color::Reset);
    let mapped256 = theme::adapt(rgb, theme::ColorLevel::Ansi256);
    assert!(matches!(mapped256, Color::Indexed(_)));
    let mapped16 = theme::adapt(rgb, theme::ColorLevel::Ansi16);
    assert!(
        matches!(
            mapped16,
            Color::Black
                | Color::Red
                | Color::Green
                | Color::Yellow
                | Color::Blue
                | Color::Magenta
                | Color::Cyan
                | Color::Gray
                | Color::DarkGray
                | Color::LightRed
                | Color::LightGreen
                | Color::LightYellow
                | Color::LightBlue
                | Color::LightMagenta
                | Color::LightCyan
                | Color::White
        ),
        "expected a 16-color ANSI color, got {mapped16:?}"
    );
}
