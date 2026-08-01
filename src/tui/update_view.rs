//! Self-update view: current→target version comparison, release notes,
//! download progress, checksum verification, and the replace confirmation.

use std::path::PathBuf;
use std::time::Duration;

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::tui::theme;
use crate::tui::update::{ReleaseInfo, UpdateMsg};
use crate::tui::widget::{Spinner, view_header};

/// Space reserved for header + version + hint rows inside the update view.
const VERSION_BLOCK_LINES: u16 = 3;
const HINT_LINES: u16 = 1;

pub struct UpdateView {
    pub phase: Phase,
    pub spinner: Spinner,
}

#[derive(Debug)]
pub enum Phase {
    Idle,
    Working,
    Available {
        current: String,
        release: ReleaseInfo,
        dry_run: bool,
    },
    AlreadyLatest {
        current: String,
    },
    Downloading {
        downloaded: u64,
        total: u64,
    },
    Downloaded,
    Verified {
        sha: String,
    },
    ReadyToConfirm {
        archive: PathBuf,
    },
    Replaced,
    Error(String),
}

impl Default for UpdateView {
    fn default() -> Self {
        Self::new()
    }
}

impl UpdateView {
    pub fn new() -> Self {
        Self {
            phase: Phase::Idle,
            spinner: Spinner::new(),
        }
    }

    pub fn tick(&mut self, dt: Duration) {
        self.spinner.tick(dt);
    }

    pub fn on_msg(&mut self, msg: UpdateMsg) {
        match msg {
            UpdateMsg::CheckStarted => self.phase = Phase::Working,
            UpdateMsg::Available {
                release,
                current,
                dry_run,
            } => {
                self.phase = Phase::Available {
                    current,
                    release,
                    dry_run,
                };
            }
            UpdateMsg::AlreadyLatest { current } => {
                self.phase = Phase::AlreadyLatest { current };
            }
            UpdateMsg::DownloadProgress { downloaded, total } => {
                self.phase = Phase::Downloading { downloaded, total };
            }
            UpdateMsg::Downloaded { .. } => self.phase = Phase::Downloaded,
            UpdateMsg::Verified { sha } => self.phase = Phase::Verified { sha },
            UpdateMsg::WaitingConfirm { archive } => {
                self.phase = Phase::ReadyToConfirm { archive };
            }
            UpdateMsg::Replaced => self.phase = Phase::Replaced,
            UpdateMsg::Failed(msg) => self.phase = Phase::Error(msg),
        }
    }

    pub fn is_confirming(&self) -> bool {
        matches!(self.phase, Phase::ReadyToConfirm { .. })
    }

    pub fn is_working(&self) -> bool {
        matches!(
            self.phase,
            Phase::Working | Phase::Downloading { .. } | Phase::Downloaded | Phase::Verified { .. }
        )
    }

    pub fn render(&self, frame: &mut ratatui::Frame, area: Rect) {
        let [header, version_area, body, hint] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(VERSION_BLOCK_LINES),
            Constraint::Min(3),
            Constraint::Length(HINT_LINES),
        ])
        .areas(area);

        view_header(
            frame,
            header,
            " Self-update ",
            &format!("current v{}", env!("CARGO_PKG_VERSION")),
        );

        self.render_version_block(frame, version_area);
        self.render_body(frame, body);

        let hint_text = match &self.phase {
            Phase::Idle => "c  check for updates     Esc  back",
            Phase::Working => "checking…",
            Phase::Available { dry_run, .. } if *dry_run => {
                "dry-run: nothing was downloaded.    Esc  back"
            }
            Phase::Available { .. } => "downloading…",
            Phase::Downloading { .. } | Phase::Downloaded | Phase::Verified { .. } => {
                "please wait…"
            }
            Phase::ReadyToConfirm { .. } => "y  replace binary     n  cancel",
            Phase::Replaced => "q  quit and restart",
            Phase::AlreadyLatest { .. } => "Esc  back",
            Phase::Error(_) => "c  retry     Esc  back",
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(hint_text, theme::dim()))),
            hint,
        );
    }

    fn info_from_phase(&self) -> Option<(&String, &ReleaseInfo)> {
        match &self.phase {
            Phase::Available {
                current, release, ..
            } => Some((current, release)),
            _ => None,
        }
    }

    fn render_version_block(&self, frame: &mut ratatui::Frame, area: Rect) {
        let Some((current, release)) = self.info_from_phase() else {
            let text = match &self.phase {
                Phase::Idle => "Press c to check for a newer release.",
                Phase::Working => "Checking GitHub for the latest release…",
                Phase::AlreadyLatest { current } => {
                    return frame.render_widget(
                        Paragraph::new(vec![
                            Line::from(Span::styled("You are up to date.", theme::success())),
                            Line::from(Span::styled(
                                format!("v{current} is the latest release."),
                                theme::dim(),
                            )),
                        ]),
                        area,
                    );
                }
                Phase::Error(_) => "",
                _ => "",
            };
            return frame.render_widget(Paragraph::new(text).style(theme::fg()), area);
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(format!("v{current}"), theme::fg()),
                Span::styled("  →  ", theme::accent_bold()),
                Span::styled(format!("v{}", release.version), theme::accent_bold()),
                Span::styled(format!("   (tag {})", release.tag_name), theme::dim()),
            ])),
            area,
        );
    }

    /// A density-filled progress bar: `█` fill, `·` remainder, paper-text on
    /// a darker band.
    fn render_progress(&self, frame: &mut ratatui::Frame, area: Rect, ratio: f64, label: String) {
        let block = ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded)
            .border_style(theme::border())
            .style(ratatui::style::Style::new().bg(theme::Layer::Raised.bg()));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let width = inner.width as usize;
        let filled = ((ratio * width as f64) as usize).min(width);
        let bar = format!("{}{}", "█".repeat(filled), "·".repeat(width - filled));
        let bar_style = ratatui::style::Style::new()
            .fg(theme::active().accent_moss)
            .bg(theme::active().bg_dark);
        frame.render_widget(
            ratatui::widgets::Paragraph::new(ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(bar, bar_style),
                ratatui::text::Span::styled(format!("  {label}"), theme::dim()),
            ])),
            inner,
        );
    }

    fn render_body(&self, frame: &mut ratatui::Frame, area: Rect) {
        let spinner = self.spinner.ch();
        let (lines, style): (Vec<Line>, Style) = match &self.phase {
            Phase::Idle => (vec![Line::from("")], theme::fg()),
            Phase::Working => (
                vec![Line::from(Span::styled(
                    format!("{spinner} Contacting GitHub…"),
                    theme::dim(),
                ))],
                theme::fg(),
            ),
            Phase::Available {
                release, dry_run, ..
            } => {
                let mut lines = vec![];
                if *dry_run {
                    lines.push(Line::from(Span::styled(
                        "DRY-RUN — no download or replacement will happen.",
                        theme::accent_bold(),
                    )));
                    lines.push(Line::from(""));
                }
                lines.push(Line::from(Span::styled(
                    "Release notes:",
                    theme::accent_bold(),
                )));
                let max = area.height.saturating_sub(4) as usize;
                for note in release.body.lines().take(max) {
                    lines.push(Line::from(Span::raw(note.to_string())));
                }
                (lines, theme::dim())
            }
            Phase::AlreadyLatest { .. } => (vec![], theme::dim()),
            Phase::Downloading { downloaded, total } => {
                let ratio = if *total > 0 {
                    *downloaded as f64 / *total as f64
                } else {
                    0.0
                };
                let pct = (ratio * 100.0) as u16;
                self.render_progress(
                    frame,
                    area,
                    ratio,
                    format!("{spinner} downloading {downloaded} / {total} bytes ({pct}%)"),
                );
                return;
            }
            Phase::Downloaded => (
                vec![Line::from(Span::styled(
                    format!("{spinner} verifying SHA-256 checksum…"),
                    theme::dim(),
                ))],
                theme::fg(),
            ),
            Phase::Verified { sha } => (
                vec![
                    Line::from(Span::styled("✓ SHA-256 verified", theme::success())),
                    Line::from(Span::raw(sha.clone())),
                ],
                theme::dim(),
            ),
            Phase::ReadyToConfirm { archive } => (
                vec![
                    Line::from(Span::styled(
                        "The new binary is downloaded and verified.",
                        theme::fg(),
                    )),
                    Line::from(Span::styled(
                        "Replacing the running binary is irreversible.",
                        theme::fg(),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("Archive: {}", archive.display()),
                        theme::dim(),
                    )),
                ],
                theme::fg(),
            ),
            Phase::Replaced => (
                vec![
                    Line::from(Span::styled(
                        "✓ Update complete. The new binary is in place.",
                        theme::success(),
                    )),
                    Line::from(Span::styled(
                        "Quit and restart mdbook-plotly to run the new version.",
                        theme::dim(),
                    )),
                ],
                theme::fg(),
            ),
            Phase::Error(msg) => (
                msg.lines()
                    .map(|l| Line::from(Span::styled(l.to_string(), theme::error())))
                    .collect(),
                theme::fg(),
            ),
        };
        frame.render_widget(Paragraph::new(lines).style(style), area);
    }
}
