//! Plot generator view: plot-type tabs, trace tabs, the questionnaire form,
//! a live JSON/TOML preview, and the action bar.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use crate::tui::plotgen::{GenStatus, PlotGen};
use crate::tui::theme;
use crate::tui::widget::view_header;

/// Terminal width at which form + preview render side by side.
pub const SIDE_BY_SIDE_MIN_WIDTH: u16 = 120;
/// Padding used to align the form control after the label column.
const FORM_LABEL_PAD: usize = 20;
/// Height of the selected-field help footer inside the form.
const FORM_HELP_LINES: u16 = 2;
/// The form + preview card occupies this share of the body (1/8 margin on
/// each side leaves the page background visible around it).
const CARD_SHARE: u16 = 8;

#[derive(Debug, Clone)]
pub struct PlotGenView {
    pub state: PlotGen,
}

impl PlotGenView {
    pub fn new(state: PlotGen) -> Self {
        Self { state }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let [header, type_tabs, trace_tabs, body, actions, status] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(area);

        view_header(
            frame,
            header,
            " Plot generator ",
            &format!(
                "v{}  ·  {}",
                env!("CARGO_PKG_VERSION"),
                self.state.lang.label()
            ),
        );

        self.render_tabs(frame, type_tabs);
        self.render_trace_tabs(frame, trace_tabs);

        let card = centered_card(body);
        // Base the split on the frame width (not the inset card), so a
        // wide terminal keeps form + preview side by side inside the card.
        if area.width >= SIDE_BY_SIDE_MIN_WIDTH {
            let [form_col, preview_col] =
                Layout::horizontal([Constraint::Percentage(42), Constraint::Percentage(58)])
                    .areas(card);
            self.render_form(frame, form_col);
            self.render_preview(frame, preview_col);
        } else {
            let [form_col, preview_col] =
                Layout::vertical([Constraint::Percentage(55), Constraint::Percentage(45)])
                    .areas(card);
            self.render_form(frame, form_col);
            self.render_preview(frame, preview_col);
        }

        self.render_actions(frame, actions);
        self.render_status(frame, status);
    }

    fn render_tabs(&mut self, frame: &mut Frame, area: Rect) {
        let mut spans = Vec::new();
        for (i, plot_type) in self.state.schema.plot_types.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw("  "));
            }
            let style = if i == self.state.type_index {
                theme::on_accent().add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                theme::outlined()
            };
            spans.push(Span::styled(
                format!(" {} {} ", i + 1, plot_type.label),
                style,
            ));
        }
        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }

    fn render_trace_tabs(&mut self, frame: &mut Frame, area: Rect) {
        let mut spans = vec![Span::styled(" trace ", theme::decor())];
        for i in 0..self.state.trace_inputs.len() {
            let style = if i == self.state.active_trace {
                theme::on_accent().add_modifier(ratatui::style::Modifier::BOLD)
            } else {
                theme::outlined()
            };
            spans.push(Span::styled(format!(" {} ", i + 1), style));
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(" + ", theme::on_accent()));
        spans.push(Span::styled(" add   ", theme::dim()));
        spans.push(Span::styled(" - ", theme::on_accent()));
        spans.push(Span::styled(" remove", theme::dim()));
        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }

    fn render_form(&mut self, frame: &mut Frame, area: Rect) {
        let block = theme::block(" form ", theme::Layer::Raised, true);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let rows_area = Rect {
            x: inner.x,
            y: inner.y,
            width: inner.width,
            height: inner.height.saturating_sub(FORM_HELP_LINES),
        };
        let help_area = Rect {
            x: inner.x,
            y: rows_area.bottom(),
            width: inner.width,
            height: inner.height.saturating_sub(rows_area.height),
        };

        let total = self.state.display_len();
        let visible = rows_area.height as usize;
        let top = self
            .state
            .selected
            .saturating_sub(visible.saturating_sub(1));
        for row in top..(top + visible).min(total) {
            let y = rows_area.y + (row - top) as u16;
            if y >= rows_area.bottom() {
                break;
            }
            let (label_style, control_style, cursor) = if row == self.state.selected {
                (theme::on_accent(), theme::on_accent(), "▸")
            } else {
                (theme::fg(), theme::dim(), " ")
            };
            let Some(field) = self.state.field_at(row) else {
                continue;
            };
            let required = if self.state.is_trace_row(row) {
                self.state
                    .current_type()
                    .required_data
                    .iter()
                    .any(|r| r == &field.name)
            } else {
                field.required
            };
            let required = if required { " *" } else { "" };
            let label = format!("{}{required}", field.label);
            let label_w = unicode_width::UnicodeWidthStr::width(label.as_str());
            let pad = FORM_LABEL_PAD.saturating_sub(label_w);
            let control = if self.state.is_editing() && self.state.editing == Some(row) {
                text_with_cursor(&self.state.edit_text, self.state.edit_cursor)
            } else {
                self.state.field_display(row)
            };
            let mut spans = vec![
                Span::styled(format!("{cursor} {label}"), label_style),
                Span::styled(" ".repeat(pad + 1), control_style),
            ];
            if let Some(error) = self.state.error_at(row) {
                spans.push(Span::styled(
                    format!("{control}  ⚠ {error}"),
                    theme::error(),
                ));
            } else {
                spans.push(Span::styled(control, control_style));
            }
            frame.render_widget(
                Paragraph::new(Line::from(spans)),
                Rect {
                    x: rows_area.x + 1,
                    y,
                    width: rows_area.width.saturating_sub(2),
                    height: 1,
                },
            );
        }

        let help = self
            .state
            .current_field()
            .map(|f| f.help.as_str())
            .unwrap_or_default();
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("  ", theme::dim()),
                Span::styled(help, theme::dim()),
            ]))
            .wrap(Wrap { trim: true }),
            help_area,
        );
    }

    fn render_preview(&mut self, frame: &mut Frame, area: Rect) {
        let block = theme::block(
            &format!(" preview ({}) ", self.state.output.label()),
            theme::Layer::Floating,
            true,
        );
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let lines: Vec<Line> = self
            .state
            .generated
            .lines()
            .map(|l| Line::from(Span::raw(l.to_string())))
            .collect();
        frame.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .style(theme::fg()),
            inner,
        );
    }

    fn render_actions(&mut self, frame: &mut Frame, area: Rect) {
        let mut spans = vec![
            Span::styled(" p ", theme::on_accent()),
            Span::styled(format!(" {}   ", self.state.output.label()), theme::dim()),
            Span::styled(" c ", theme::on_accent()),
            Span::styled(" copy   ", theme::dim()),
            Span::styled(" s ", theme::on_accent()),
            Span::styled(" save   ", theme::dim()),
            Span::styled(" r ", theme::on_accent()),
            Span::styled(" reset   ", theme::dim()),
            Span::styled(" [ ] ", theme::on_accent()),
            Span::styled(" trace   ", theme::dim()),
            Span::styled(" a/d ", theme::on_accent()),
            Span::styled(" add/remove   ", theme::dim()),
            Span::styled(" 1-8 ", theme::on_accent()),
            Span::styled(" plot type", theme::dim()),
        ];
        if self.state.has_errors() {
            spans.push(Span::styled("   ⚠ invalid", theme::error()));
        }
        frame.render_widget(
            Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
            area,
        );
    }

    fn render_status(&mut self, frame: &mut Frame, area: Rect) {
        let line = if self.state.is_editing() {
            Line::from(Span::styled(
                "Esc cancel · Enter commit · type to edit · ←→ move cursor",
                theme::dim(),
            ))
        } else {
            match &self.state.status {
                Some(GenStatus::Copied) => {
                    Line::from(Span::styled("✓ Copied to clipboard.", theme::success()))
                }
                Some(GenStatus::ClipboardUnavailable) => Line::from(Span::styled(
                    "Clipboard unavailable — the generated text will be printed to stdout instead.",
                    theme::error(),
                )),
                Some(GenStatus::Saved(path)) => Line::from(Span::styled(
                    format!("✓ Saved to {path}."),
                    theme::success(),
                )),
                Some(GenStatus::Error(msg)) => {
                    Line::from(Span::styled(msg.clone(), theme::error()))
                }
                None if self.state.has_errors() => Line::from(Span::styled(
                    format!(
                        "Fix the highlighted fields first: {}",
                        self.state
                            .trace_errors
                            .iter()
                            .flatten()
                            .flatten()
                            .chain(self.state.global_errors.iter().flatten())
                            .next()
                            .cloned()
                            .unwrap_or_default()
                    ),
                    theme::error(),
                )),
                None => Line::from(Span::styled(
                    "↑↓ move · Enter edit or toggle · ←→ cycle · [ ] trace · a add · d delete",
                    theme::dim(),
                )),
            }
        };
        frame.render_widget(Paragraph::new(line), area);
    }
}

/// Shrink `area` to a centered card occupying about 3/4 of both dimensions,
/// leaving the page background visible around it.
fn centered_card(area: Rect) -> Rect {
    let margin_w = area.width / CARD_SHARE;
    let margin_h = area.height / CARD_SHARE;
    let margin_w = margin_w.min(area.width.saturating_sub(20) / 2);
    let margin_h = margin_h.min(area.height.saturating_sub(3) / 2);
    Rect {
        x: area.x + margin_w,
        y: area.y + margin_h,
        width: area.width - 2 * margin_w,
        height: area.height - 2 * margin_h,
    }
}

/// Insert a cursor marker into text at a character index.
fn text_with_cursor(text: &str, cursor: usize) -> String {
    let mut chars: Vec<char> = text.chars().collect();
    let pos = cursor.min(chars.len());
    chars.insert(pos, '▏');
    chars.into_iter().collect()
}
