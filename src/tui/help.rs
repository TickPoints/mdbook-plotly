//! Full keybinding help overlay, opened with `?`.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Margin, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};

use crate::tui::theme;

const SECTIONS: &[(&str, &[(&str, &str)])] = &[
    (
        "Global",
        &[
            ("q / Ctrl+C", "quit"),
            ("Esc", "back to home / close this help"),
            ("?", "toggle this help"),
        ],
    ),
    (
        "Home",
        &[
            ("↑/↓", "move selection"),
            (
                "1 / 2 / 3",
                "open self-update / book.toml editor / cheat-sheet",
            ),
            ("Enter", "open selected tool"),
        ],
    ),
    (
        "Self-update",
        &[
            ("c", "check for the latest release"),
            ("y / n", "confirm / decline replacing the binary"),
            ("r", "re-check after an error"),
        ],
    ),
    (
        "book.toml editor",
        &[
            ("↑/↓", "select a config item"),
            ("Enter / Space", "edit / toggle the selected item"),
            ("←/→", "cycle enum values"),
            ("a", "show diff and apply"),
            ("y / n", "confirm / cancel writing"),
        ],
    ),
    (
        "Cheat-sheet",
        &[
            ("type", "search as you type"),
            ("↑/↓", "move selection"),
            ("Enter", "copy example to the clipboard"),
            ("p", "print example to stdout (clipboard fallback)"),
            ("Esc", "clear the search box"),
        ],
    ),
];

pub fn render(frame: &mut Frame, area: Rect) {
    let popup = {
        let [_x] = Layout::horizontal([Constraint::Percentage(64)]).areas(area);
        let [y] = Layout::vertical([Constraint::Percentage(58)]).areas(area);
        y
    };
    let inner = popup.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Block::default()
            .title(" Help ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme::border_active()),
        popup,
    );

    let mut lines: Vec<Line> = Vec::new();
    for (section, rows) in SECTIONS {
        lines.push(Line::from(Span::styled(*section, theme::accent_bold())));
        for (keys, desc) in *rows {
            let keys_w = unicode_width::UnicodeWidthStr::width(*keys);
            lines.push(Line::from(vec![
                Span::styled(format!("  {keys}"), theme::fg()),
                Span::styled(" ".repeat(22usize.saturating_sub(keys_w)), theme::dim()),
                Span::styled(format!("  {desc}"), theme::dim()),
            ]));
        }
        lines.push(Line::from(""));
    }
    frame.render_widget(Paragraph::new(lines).style(theme::fg()), inner);
}
