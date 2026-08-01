//! book.toml editor view: item list with per-field guidance, a diff modal,
//! and status reporting.

use std::path::PathBuf;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};
use toml_edit::DocumentMut;

use crate::tui::book_toml::{
    ConfigItem, DiffLine, ItemKind, apply_item, build_items, diff, find_book_toml,
};
use crate::tui::theme;
use crate::tui::widget::{self, Input};

/// Padding used to align list values after the dotted path column.
const LIST_VALUE_PAD: usize = 52;
/// Diff modal size (percent of the terminal).
const DIFF_MODAL_WIDTH_PERCENT: u16 = 70;
const DIFF_MODAL_HEIGHT_PERCENT: u16 = 60;

pub struct ConfigView {
    pub book_toml: Option<PathBuf>,
    pub not_found_error: bool,
    pub doc: Option<DocumentMut>,
    pub original: String,
    pub items: Vec<ConfigItem>,
    pub selected: usize,
    pub editing: bool,
    pub input: Input,
    pub confirm_apply: bool,
    pub status: Option<ConfigStatus>,
}

pub enum ConfigStatus {
    Written,
    NoChanges,
    Error(String),
}

impl Default for ConfigView {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigView {
    pub fn new() -> Self {
        let mut view = Self {
            book_toml: find_book_toml(&std::env::current_dir().unwrap_or_default()),
            not_found_error: false,
            doc: None,
            original: String::new(),
            items: Vec::new(),
            selected: 0,
            editing: false,
            input: Input::new(),
            confirm_apply: false,
            status: None,
        };
        view.reload();
        view
    }

    fn reload(&mut self) {
        let Some(path) = self.book_toml.clone() else {
            self.not_found_error = true;
            return;
        };
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                let doc: DocumentMut = match text.parse() {
                    Ok(d) => d,
                    Err(e) => {
                        self.not_found_error = false;
                        self.status = Some(ConfigStatus::Error(format!(
                            "cannot parse {}: {e}",
                            path.display()
                        )));
                        return;
                    }
                };
                self.original = text;
                self.items = build_items(&doc);
                self.doc = Some(doc);
                self.not_found_error = false;
            }
            Err(e) => {
                self.not_found_error = false;
                self.status = Some(ConfigStatus::Error(format!(
                    "cannot read {}: {e}",
                    path.display()
                )));
            }
        }
    }

    pub fn current_item(&self) -> Option<&ConfigItem> {
        self.items.get(self.selected)
    }

    pub fn cycle_selected(&mut self, delta: isize) {
        if let Some(item) = self.items.get_mut(self.selected)
            && let ItemKind::Enum(choices, idx) = &mut item.kind
        {
            let n = choices.len() as isize;
            *idx = ((*idx as isize + delta).rem_euclid(n)) as usize;
        }
    }

    pub fn toggle_selected(&mut self) {
        if let Some(item) = self.items.get_mut(self.selected)
            && let ItemKind::Bool(b) = &mut item.kind
        {
            *b = !*b;
        }
    }

    /// Apply all current items to a working document and return the new
    /// content (for the diff preview) without writing to disk.
    pub fn preview(&self) -> String {
        let mut doc = self.doc.clone().unwrap_or_default();
        for item in &self.items {
            apply_item(&mut doc, item);
        }
        doc.to_string()
    }

    /// The diff between the original file and the current edits.
    pub fn pending_diff(&self) -> Vec<DiffLine> {
        diff(&self.original, &self.preview())
    }

    /// Write the current edits back to `book.toml`.
    pub fn write(&mut self) -> std::io::Result<()> {
        let Some(path) = self.book_toml.clone() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "book.toml not found",
            ));
        };
        let content = self.preview();
        crate::tui::book_toml::atomic_write(&path, &content)?;
        self.original = content;
        self.status = Some(ConfigStatus::Written);
        Ok(())
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        if self.not_found_error {
            let msg = format!(
                "No book.toml found in {} or any parent directory.\n\
                 Run this inside a mdbook project directory.",
                std::env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
            );
            widget::confirm_dialog(frame, area, " book.toml not found ", &msg, "Esc to go back");
            return;
        }

        let [header, list, detail, status_line] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Percentage(55),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .areas(area);

        let title = " book.toml editor ";
        let right = self
            .book_toml
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        widget::view_header(frame, header, title, &right);

        // Item list
        let list_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme::border())
            .style(ratatui::style::Style::new().bg(theme::Layer::Raised.bg()));
        let list_inner = list_block.inner(list);
        frame.render_widget(list_block, list);
        let visible = list_inner.height as usize;
        let scroll = self.selected.saturating_sub(visible.saturating_sub(1));
        for (i, item) in self.items.iter().enumerate().skip(scroll).take(visible) {
            let y = list_inner.y + (i - scroll) as u16;
            if y >= list_inner.bottom() {
                break;
            }
            let (key_style, val_style) = if i == self.selected {
                (theme::selected(), theme::selected())
            } else {
                (theme::fg(), theme::dim())
            };
            let path_w = unicode_width::UnicodeWidthStr::width(item.path.as_str());
            let pad = LIST_VALUE_PAD.saturating_sub(path_w);
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(item.path.clone(), key_style),
                    Span::styled(" ".repeat(pad), val_style),
                    Span::styled(item.kind.display_value(), val_style),
                ])),
                Rect {
                    x: list_inner.x,
                    y,
                    width: list_inner.width,
                    height: 1,
                },
            );
        }

        // Detail pane for the selected item
        if let Some(item) = self.current_item() {
            let lines = vec![
                Line::from(Span::styled(item.description.clone(), theme::fg())),
                Line::from(Span::styled("", theme::dim())),
                Line::from(vec![
                    Span::styled("Valid: ", theme::accent_bold()),
                    Span::styled(item.valid.clone(), theme::dim()),
                ]),
                Line::from(vec![
                    Span::styled("Default: ", theme::accent_bold()),
                    Span::styled(item.default.clone(), theme::dim()),
                ]),
                Line::from(vec![
                    Span::styled("Current: ", theme::accent_bold()),
                    Span::styled(item.kind.display_value(), theme::fg()),
                ]),
            ];
            frame.render_widget(
                Paragraph::new(lines)
                    .wrap(Wrap { trim: true })
                    .style(theme::fg()),
                Rect {
                    x: detail.x + 1,
                    y: detail.y,
                    width: detail.width.saturating_sub(2),
                    height: detail.height,
                },
            );
        }

        // Status / input line
        let status = match &self.status {
            Some(ConfigStatus::Written) => {
                Line::from(Span::styled("✓ Written to book.toml.", theme::success()))
            }
            Some(ConfigStatus::NoChanges) => {
                Line::from(Span::styled("No changes to write.", theme::dim()))
            }
            Some(ConfigStatus::Error(msg)) => Line::from(Span::styled(msg.clone(), theme::error())),
            None if self.editing => Line::from(Span::styled(
                "Editing — Enter to confirm, Esc to cancel",
                theme::dim(),
            )),
            None => Line::from(""),
        };
        frame.render_widget(Paragraph::new(status), status_line);
    }

    /// Render the diff preview modal.
    pub fn render_diff_modal(&self, frame: &mut Frame, area: Rect, diff: &[DiffLine]) {
        let popup =
            widget::centered_area(area, DIFF_MODAL_WIDTH_PERCENT, DIFF_MODAL_HEIGHT_PERCENT);
        frame.render_widget(Clear, popup);
        frame.render_widget(
            Block::default()
                .title(" Diff preview ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme::border_active())
                .style(ratatui::style::Style::new().bg(theme::Layer::Floating.bg())),
            popup,
        );
        let inner = popup.inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        let lines: Vec<Line> = diff
            .iter()
            .map(|l| match l {
                DiffLine::Same(t) => Line::from(Span::styled(format!("  {t}"), theme::fg())),
                DiffLine::Added(t) => Line::from(Span::styled(format!("+ {t}"), theme::success())),
                DiffLine::Removed(t) => Line::from(Span::styled(format!("- {t}"), theme::error())),
            })
            .collect();
        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "Write to disk?  y / n",
                theme::accent_bold(),
            ))),
            Rect {
                x: inner.x,
                y: popup.bottom().saturating_sub(2),
                width: inner.width,
                height: 1,
            },
        );
    }
}
