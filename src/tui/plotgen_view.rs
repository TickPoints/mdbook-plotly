//! Plot generator view: plot-type tabs, the questionnaire form, a live
//! JSON/TOML preview, and the action bar.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};

use crate::tui::plotgen::{GenStatus, PlotGen};
use crate::tui::theme;
use crate::tui::widget::view_header;

/// Terminal width at which form + preview render side by side.
pub const SIDE_BY_SIDE_MIN_WIDTH: u16 = 120;
/// Padding used to align the form control after the label column.
const FORM_LABEL_PAD: usize = 20;
/// Height of the selected-field help footer inside the form.
const FORM_HELP_LINES: u16 = 2;

pub struct PlotGenView {
    pub state: PlotGen,
}

impl PlotGenView {
    pub fn new(state: PlotGen) -> Self {
        Self { state }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let [header, tabs, body, actions, status] = Layout::vertical([
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

        self.render_tabs(frame, tabs);

        if area.width >= SIDE_BY_SIDE_MIN_WIDTH {
            let [form_col, preview_col] =
                Layout::horizontal([Constraint::Percentage(42), Constraint::Percentage(58)])
                    .areas(body);
            self.render_form(frame, form_col);
            self.render_preview(frame, preview_col);
        } else {
            let [form_col, preview_col] =
                Layout::vertical([Constraint::Percentage(55), Constraint::Percentage(45)])
                    .areas(body);
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
                theme::selected()
            } else {
                theme::dim()
            };
            spans.push(Span::styled(
                format!(" {} {} ", i + 1, plot_type.label),
                style,
            ));
        }
        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }

    fn render_form(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" form ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme::border());
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

        let fields = &self.state.current_type().fields;
        let visible = rows_area.height as usize;
        let top = self
            .state
            .selected
            .saturating_sub(visible.saturating_sub(1));
        for (i, field) in fields.iter().enumerate().skip(top).take(visible) {
            let y = rows_area.y + (i - top) as u16;
            if y >= rows_area.bottom() {
                break;
            }
            let (label_style, control_style, cursor) = if i == self.state.selected {
                (theme::accent_bold(), theme::fg(), "›")
            } else {
                (theme::fg(), theme::dim(), " ")
            };
            let required = if field.required { " *" } else { "" };
            let label = format!("{}{required}", field.label);
            let label_w = unicode_width::UnicodeWidthStr::width(label.as_str());
            let pad = FORM_LABEL_PAD.saturating_sub(label_w);
            let control = self.state.field_display(i);
            let mut spans = vec![
                Span::styled(format!("{cursor} {label}"), label_style),
                Span::styled(" ".repeat(pad + 1), control_style),
            ];
            if self.state.errors[i].is_some() {
                spans.push(Span::styled(control, theme::error()));
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
        let block = Block::default()
            .title(format!(" preview ({}) ", self.state.output.label()))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme::border());
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
            Span::styled(
                " p ",
                theme::accent().add_modifier(ratatui::style::Modifier::REVERSED),
            ),
            Span::styled(format!(" {}   ", self.state.output.label()), theme::dim()),
            Span::styled(
                " c ",
                theme::accent().add_modifier(ratatui::style::Modifier::REVERSED),
            ),
            Span::styled(" copy   ", theme::dim()),
            Span::styled(
                " s ",
                theme::accent().add_modifier(ratatui::style::Modifier::REVERSED),
            ),
            Span::styled(" save   ", theme::dim()),
            Span::styled(
                " r ",
                theme::accent().add_modifier(ratatui::style::Modifier::REVERSED),
            ),
            Span::styled(" reset   ", theme::dim()),
            Span::styled(
                " 1-8 ",
                theme::accent().add_modifier(ratatui::style::Modifier::REVERSED),
            ),
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
        let line = match &self.state.status {
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
            Some(GenStatus::Error(msg)) => Line::from(Span::styled(msg.clone(), theme::error())),
            None if self.state.has_errors() => Line::from(Span::styled(
                format!(
                    "Fix the highlighted fields first: {}",
                    self.state
                        .errors
                        .iter()
                        .flatten()
                        .next()
                        .cloned()
                        .unwrap_or_default()
                ),
                theme::error(),
            )),
            None => Line::from(Span::styled(
                "↑↓ move · Enter/Space toggle · ←→ cycle or move cursor · type to edit",
                theme::dim(),
            )),
        };
        frame.render_widget(Paragraph::new(line), area);
    }
}
