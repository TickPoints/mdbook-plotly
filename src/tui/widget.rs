//! Small reusable widgets: spinner, key-hint bar, centered confirm
//! dialog, text input, and a scrollable paragraph helper.

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};

use crate::tui::theme;

pub const KEY_SPACING: &str = "  ";

/// A small text input buffer with cursor tracking (CJK-aware widths are
/// handled by the renderer via `unicode-width`).
#[derive(Debug, Clone, Default)]
pub struct Input {
    pub text: String,
    pub cursor: usize,
}

impl Input {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
        }
    }

    pub fn insert(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.text.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    pub fn delete(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
        }
    }

    pub fn left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.text.len());
    }

    pub fn reset(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }
}

/// A lightweight rotating spinner for "in progress" states.
#[derive(Debug, Clone)]
pub struct Spinner {
    frames: &'static [char],
    idx: usize,
    acc: Duration,
    interval: Duration,
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            frames: &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'],
            idx: 0,
            acc: Duration::ZERO,
            interval: Duration::from_millis(80),
        }
    }

    pub fn tick(&mut self, dt: Duration) {
        self.acc += dt;
        while self.acc >= self.interval {
            self.acc -= self.interval;
            self.idx = (self.idx + 1) % self.frames.len();
        }
    }

    pub fn ch(&self) -> char {
        self.frames[self.idx]
    }
}

/// Render the persistent bottom key-hint bar.
pub fn key_bar(frame: &mut Frame, area: Rect, hints: &[(&str, &str)]) {
    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, label)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(KEY_SPACING, theme::dim()));
        }
        spans.push(Span::styled(
            format!(" {key} "),
            theme::accent().add_modifier(Modifier::REVERSED),
        ));
        spans.push(Span::styled(format!(" {label}"), theme::dim()));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// A centered dialog with a title, message body, and a prompt line.
pub fn centered_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let [_x] = Layout::horizontal([Constraint::Percentage(percent_x)]).areas(area);
    let [y] = Layout::vertical([Constraint::Percentage(percent_y)]).areas(area);
    y
}

/// Render a modal confirm dialog over the whole screen.
pub fn confirm_dialog(frame: &mut Frame, area: Rect, title: &str, message: &str, prompt: &str) {
    let popup = centered_area(area, 60, 40);
    let inner = popup.inner(ratatui::layout::Margin {
        horizontal: 1,
        vertical: 1,
    });
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Block::default()
            .title(title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme::border_active()),
        popup,
    );
    let lines: Vec<Line> = message
        .lines()
        .map(|l| Line::from(Span::styled(l.to_string(), theme::fg())))
        .collect();
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .style(theme::fg()),
        inner,
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(prompt, theme::accent_bold())))
            .alignment(Alignment::Center)
            .style(theme::fg()),
        Rect {
            x: inner.x,
            y: inner.bottom().saturating_sub(2),
            width: inner.width,
            height: 1,
        },
    );
}

/// Render a single-line text input inside a titled rounded box.
pub fn text_input(frame: &mut Frame, area: Rect, title: &str, value: &str, cursor: usize) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme::border_active());
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let text = if value.is_empty() {
        Line::from("")
    } else {
        Line::from(Span::raw(value.to_string()))
    };
    frame.render_widget(Paragraph::new(text).style(theme::fg()), inner);
    let cursor_x = inner.x + cursor as u16;
    frame.set_cursor_position(ratatui::layout::Position::new(
        cursor_x.min(inner.right().saturating_sub(1)),
        inner.y,
    ));
}

/// Render a header line for non-home views: view title + current version.
pub fn view_header(frame: &mut Frame, area: Rect, title: &str, right: &str) {
    let left_w = unicode_width::UnicodeWidthStr::width(title);
    let right_w = unicode_width::UnicodeWidthStr::width(right);
    let pad = area
        .width
        .saturating_sub(left_w as u16 + right_w as u16 + 1);
    let line = Line::from(vec![
        Span::styled(title.to_string(), theme::accent_bold()),
        Span::raw(" ".repeat(pad as usize)),
        Span::styled(right.to_string(), theme::dim()),
    ]);
    frame.render_widget(Paragraph::new(line).style(theme::fg()), area);
}
