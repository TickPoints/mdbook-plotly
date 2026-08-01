//! Welcome view: big title, subtitle with a density accent line, the three
//! tools, and the footer hint.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::Modifier;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use tui_big_text::BigText;

use crate::tui::theme;

pub const MENU_ITEMS: [(&str, &str); 3] = [
    (
        "1  Self-update",
        "Check the latest release, download, verify, and replace this binary",
    ),
    (
        "2  Edit book.toml",
        "Tweak the [preprocessor.plotly] configuration with guided edits",
    ),
    (
        "3  Plot generator",
        "Pick a plot type, fill the questionnaire, and export JSON/TOML",
    ),
];

/// Subtitle shown under the big title.
const SUBTITLE: &str = "FULL EDITION";

#[derive(Debug, Clone, Copy, Default)]
pub struct HomeView {
    pub selected: usize,
}

impl HomeView {
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let [title_area, subtitle_area, info_area, menu_area, footer_area] = Layout::vertical([
            Constraint::Length(8),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .areas(area);

        let big = BigText::builder()
            .lines(vec![Line::from("mdbook-plotly")])
            .style(theme::accent_bold())
            .alignment(Alignment::Center)
            .build();
        frame.render_widget(big, title_area);

        let sub = format!("{}  {}", SUBTITLE, theme::density_line(0.6, 8));
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(sub, theme::accent())))
                .alignment(Alignment::Center),
            subtitle_area,
        );

        // This module only compiles in the full (tui) build.
        let info = format!(
            "v{}  ·  {}",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_REPOSITORY")
        );
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(info, theme::decor())))
                .alignment(Alignment::Center),
            info_area,
        );

        self.render_menu(frame, menu_area);

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("↑↓", theme::on_accent()),
                Span::styled("  select   ", theme::dim()),
                Span::styled("Enter", theme::on_accent()),
                Span::styled("  open", theme::dim()),
            ]))
            .alignment(Alignment::Center),
            footer_area,
        );
    }

    fn render_menu(&mut self, frame: &mut Frame, area: Rect) {
        let item_h = MENU_ITEMS.len() as u16;
        let menu_top = area
            .top()
            .saturating_add(area.height.saturating_sub(item_h) / 2);
        for (i, (title, desc)) in MENU_ITEMS.iter().enumerate() {
            let y = menu_top + i as u16;
            if y >= area.bottom() {
                break;
            }
            let (title_style, desc_style) = if i == self.selected {
                (
                    theme::on_accent().add_modifier(Modifier::BOLD),
                    theme::on_accent(),
                )
            } else {
                (theme::fg(), theme::dim())
            };
            let title_w = unicode_width::UnicodeWidthStr::width(*title);
            let pad = 42usize.saturating_sub(title_w);
            frame.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled(title.to_string(), title_style),
                    Span::styled(" ".repeat(pad), desc_style),
                    Span::styled(desc.to_string(), desc_style),
                ])),
                Rect {
                    x: area.x,
                    y,
                    width: area.width,
                    height: 1,
                },
            );
        }
    }
}
