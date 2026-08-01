//! Cheat-sheet view: search box, example list, code preview.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

use crate::docs_parser::{PlotEntry, UsageDoc};
use crate::tui::cheatsheet::DocSource;
use crate::tui::theme;
use crate::tui::widget::{self, Input};

/// Search box height.
pub const SEARCH_BOX_LINES: u16 = 3;
/// Terminal width at which list + preview render side by side.
pub const SIDE_BY_SIDE_MIN_WIDTH: u16 = 120;
/// Space reserved for the title column inside the example list.
pub const LIST_TITLE_PAD: usize = 24;

pub struct CheatSheetView {
    pub search: Input,
    pub doc: Option<UsageDoc>,
    pub source: Option<DocSource>,
    pub selected: usize,
    pub scroll: usize,
    pub loading: bool,
    pub copy_status: Option<CopyStatus>,
}

pub enum CopyStatus {
    Copied,
    ClipboardUnavailable,
    NoSelection,
}

impl Default for CheatSheetView {
    fn default() -> Self {
        Self::new()
    }
}

impl CheatSheetView {
    pub fn new() -> Self {
        Self {
            search: Input::new(),
            doc: None,
            source: None,
            selected: 0,
            scroll: 0,
            loading: true,
            copy_status: None,
        }
    }

    pub fn matches(&self, entry: &PlotEntry) -> bool {
        let q = self.search.text.trim();
        q.is_empty() || entry.matches(q)
    }

    pub fn filtered(&self) -> Vec<&PlotEntry> {
        self.doc
            .iter()
            .flat_map(|d| d.plots.iter())
            .filter(|p| self.matches(p))
            .collect()
    }

    pub fn clamp_selection(&mut self, len: usize) {
        if len == 0 {
            self.selected = 0;
        } else {
            self.selected = self.selected.min(len - 1);
        }
    }

    pub fn selected_entry(&self) -> Option<PlotEntry> {
        let filtered = self.filtered();
        filtered.get(self.selected).cloned().cloned()
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let [header, search_area, body, status] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(SEARCH_BOX_LINES),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .areas(area);

        widget::view_header(
            frame,
            header,
            " Plot cheat-sheet ",
            &format!("v{}", env!("CARGO_PKG_VERSION")),
        );

        widget::text_input(
            frame,
            search_area,
            " search (type to filter) ",
            &self.search.text,
            self.search.cursor,
        );

        let source_label = self
            .source
            .map(|s| format!(" · source: {}", s.label()))
            .unwrap_or_default();

        if self.loading {
            let msg = format!("Loading examples{source_label}…");
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(msg, theme::dim())))
                    .alignment(Alignment::Center),
                body,
            );
            return;
        }

        let entries = self.filtered();
        let wide = area.width >= SIDE_BY_SIDE_MIN_WIDTH;
        if wide {
            let [list_col, preview_col] =
                Layout::horizontal([Constraint::Percentage(38), Constraint::Percentage(62)])
                    .areas(body);
            self.render_list(frame, list_col, &entries);
            self.render_preview(frame, preview_col);
        } else {
            let [list_col, preview_col] =
                Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(body);
            self.render_list(frame, list_col, &entries);
            self.render_preview(frame, preview_col);
        }

        let status_line = match &self.copy_status {
            Some(CopyStatus::Copied) => {
                Line::from(Span::styled("✓ Copied to clipboard.", theme::success()))
            }
            Some(CopyStatus::ClipboardUnavailable) => Line::from(Span::styled(
                "Clipboard unavailable — the example will be printed to stdout instead.",
                theme::error(),
            )),
            Some(CopyStatus::NoSelection) => {
                Line::from(Span::styled("No example selected.", theme::dim()))
            }
            None => {
                let count_msg = if entries.is_empty() {
                    "No matches.".to_string()
                } else {
                    format!("{} examples", entries.len())
                };
                Line::from(Span::styled(
                    format!("{count_msg}{source_label}"),
                    theme::dim(),
                ))
            }
        };
        frame.render_widget(Paragraph::new(status_line), status);
    }

    fn render_list(&self, frame: &mut Frame, area: Rect, entries: &[&PlotEntry]) {
        let block = Block::default()
            .title(" examples ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme::border());
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let visible = inner.height as usize;
        let top = self.selected.saturating_sub(visible.saturating_sub(1));
        for (i, entry) in entries.iter().enumerate().skip(top).take(visible) {
            let y = inner.y + (i - top) as u16;
            if y >= inner.bottom() {
                break;
            }
            let (style, tags_style) = if i == self.selected {
                (theme::selected(), theme::selected())
            } else {
                (theme::fg(), theme::dim())
            };
            let title = if entry.title.is_empty() {
                &entry.id
            } else {
                &entry.title
            };
            let title_w = unicode_width::UnicodeWidthStr::width(title.as_str());
            let pad = LIST_TITLE_PAD.saturating_sub(title_w);
            let tags = entry.tags.join(",");
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(title.clone(), style),
                    Span::styled(" ".repeat(pad), tags_style),
                    Span::styled(tags, tags_style),
                ])),
                Rect {
                    x: inner.x + 1,
                    y,
                    width: inner.width.saturating_sub(2),
                    height: 1,
                },
            );
        }
    }

    fn render_preview(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" preview ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme::border());
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let Some(entry) = self.selected_entry() else {
            frame.render_widget(
                Paragraph::new("No example selected.").style(theme::dim()),
                inner,
            );
            return;
        };
        let lines: Vec<Line> = entry
            .code
            .lines()
            .map(|l| Line::from(Span::raw(l.to_string())))
            .collect();
        let paragraph = Paragraph::new(lines)
            .scroll((self.scroll as u16, 0))
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph.style(theme::fg()), inner);
    }
}
