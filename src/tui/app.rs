//! The `App` shell: state, event handling, worker plumbing, and rendering.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use tachyonfx::{Effect, EffectTimer, Interpolation, Motion, fx};

use crate::tui::book_toml;
use crate::tui::book_toml_view::{self, ConfigView};
use crate::tui::github::{GithubHosts, RepoSpec};
use crate::tui::home::{HomeView, MENU_ITEMS};
use crate::tui::locale::resolve_language;
use crate::tui::plotgen::PlotGen;
use crate::tui::plotgen_view::PlotGenView;
use crate::tui::settings;
use crate::tui::update;
use crate::tui::update_view::{self, UpdateView};
use crate::tui::{
    EVENT_POLL_MS, TerminalType, UPDATE_SWEEP_MS, VIEW_FADE_MS, View, help, theme, widget,
};

/// Sweep gradient length / randomness for the update-complete flash.
const SWEEP_GRADIENT: u16 = 3;
const SWEEP_RANDOMNESS: u16 = 10;

pub struct App {
    opts: crate::tui::TuiOptions,
    view: View,
    help: bool,
    quit: bool,
    home: HomeView,
    update: UpdateView,
    config: ConfigView,
    gen_view: PlotGenView,
    transition: Option<Effect>,
    flash: Option<Effect>,
    hosts: GithubHosts,
    repo: RepoSpec,
    update_rx: Option<mpsc::Receiver<update::UpdateMsg>>,
    update_confirm_tx: Option<mpsc::Sender<bool>>,
    clipboard: Option<arboard::Clipboard>,
}

impl App {
    pub fn new(opts: crate::tui::TuiOptions) -> Self {
        let clipboard = arboard::Clipboard::new().ok();
        let hosts = GithubHosts::resolve(&settings::github_overrides());
        let repo = RepoSpec::from_pkg_repository();
        let lang = resolve_language();
        Self {
            opts,
            view: View::Home,
            help: false,
            quit: false,
            home: HomeView::default(),
            update: UpdateView::new(),
            config: ConfigView::new(),
            gen_view: PlotGenView::new(PlotGen::new(lang)),
            transition: None,
            flash: None,
            hosts,
            repo,
            update_rx: None,
            update_confirm_tx: None,
            clipboard,
        }
    }

    // -- view switching -----------------------------------------------------

    fn change_view(&mut self, view: View) {
        self.view = view;
        if !self.opts.no_effects {
            self.transition = Some(fx::fade_from(
                Color::Black,
                Color::Black,
                EffectTimer::from_ms(VIEW_FADE_MS, Interpolation::QuadOut),
            ));
        }
    }

    // -- worker plumbing ----------------------------------------------------

    fn start_update_worker(&mut self) {
        let (tx, rx) = mpsc::channel();
        let (confirm_tx, confirm_rx) = mpsc::channel();
        let cache_dir = directories::ProjectDirs::from("", "", "mdbook-plotly")
            .map(|d| d.cache_dir().to_path_buf())
            .unwrap_or_else(std::env::temp_dir);
        update::spawn_update_worker(
            self.opts.dry_run,
            &cache_dir,
            self.hosts.clone(),
            self.repo.clone(),
            tx,
            confirm_rx,
        );
        self.update_rx = Some(rx);
        self.update_confirm_tx = Some(confirm_tx);
    }

    fn drain_channels(&mut self) {
        if let Some(rx) = &self.update_rx {
            while let Ok(msg) = rx.try_recv() {
                self.update.on_msg(msg);
                if matches!(self.update.phase, update_view::Phase::Replaced)
                    && !self.opts.no_effects
                {
                    self.flash = Some(fx::sweep_in(
                        Motion::LeftToRight,
                        SWEEP_GRADIENT,
                        SWEEP_RANDOMNESS,
                        Color::Black,
                        EffectTimer::from_ms(UPDATE_SWEEP_MS, Interpolation::QuadOut),
                    ));
                }
            }
        }
    }

    // -- event handling -----------------------------------------------------

    fn on_key(&mut self, key: KeyEvent) {
        let code = key.code;

        // Global keys.
        if code == KeyCode::Char('?') {
            self.help = !self.help;
            return;
        }
        if self.help {
            if matches!(code, KeyCode::Esc | KeyCode::Char('q' | 'Q')) {
                self.help = false;
            }
            return;
        }
        if key.modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }
        if code == KeyCode::Char('q') {
            self.quit = true;
            return;
        }

        if self.view == View::Update && self.update.is_confirming() {
            match code {
                KeyCode::Char('y' | 'Y') => {
                    if let Some(tx) = self.update_confirm_tx.take() {
                        let _ = tx.send(true);
                    }
                }
                KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                    if let Some(tx) = self.update_confirm_tx.take() {
                        let _ = tx.send(false);
                    }
                    self.update.phase = update_view::Phase::Idle;
                }
                _ => {}
            }
            return;
        }

        match self.view {
            View::Home => match code {
                KeyCode::Esc | KeyCode::Char('q') => self.quit = true,
                KeyCode::Up => {
                    self.home.selected = self.home.selected.saturating_sub(1);
                }
                KeyCode::Down => {
                    self.home.selected = (self.home.selected + 1).min(MENU_ITEMS.len() - 1);
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    if let Some(idx) = c.to_digit(10)
                        && (1..=MENU_ITEMS.len() as u32).contains(&idx)
                    {
                        self.open_tool(idx as usize - 1);
                    }
                }
                KeyCode::Enter => self.open_tool(self.home.selected),
                _ => {}
            },
            View::Update => match code {
                KeyCode::Esc => self.change_view(View::Home),
                KeyCode::Char('c' | 'C') if !self.update.is_working() => {
                    self.update.phase = update_view::Phase::Idle;
                    self.start_update_worker();
                }
                KeyCode::Char('r' | 'R')
                    if matches!(self.update.phase, update_view::Phase::Error(_)) =>
                {
                    self.update.phase = update_view::Phase::Idle;
                    self.start_update_worker();
                }
                _ => {}
            },
            View::Config => self.on_config_key(code),
            View::PlotGen => self.on_plotgen_key(code),
        }
    }

    fn open_tool(&mut self, idx: usize) {
        let view = match idx {
            0 => View::Update,
            1 => View::Config,
            2 => View::PlotGen,
            _ => return,
        };
        self.change_view(view);
    }

    fn on_config_key(&mut self, code: KeyCode) {
        if self.config.confirm_apply {
            match code {
                KeyCode::Char('y' | 'Y') => match self.config.write() {
                    Ok(()) => {
                        self.config.confirm_apply = false;
                        self.config.editing = false;
                    }
                    Err(e) => {
                        self.config.confirm_apply = false;
                        self.config.status =
                            Some(book_toml_view::ConfigStatus::Error(e.to_string()));
                    }
                },
                KeyCode::Char('n' | 'N') | KeyCode::Esc => {
                    self.config.confirm_apply = false;
                }
                _ => {}
            }
            return;
        }

        if self.config.editing {
            match code {
                KeyCode::Esc => {
                    self.config.editing = false;
                }
                KeyCode::Enter => {
                    if let Some(item) = self.config.items.get_mut(self.config.selected) {
                        item.kind.parse_text(&self.config.input.text);
                    }
                    self.config.editing = false;
                }
                KeyCode::Char(c) => self.config.input.insert(c),
                KeyCode::Backspace => self.config.input.backspace(),
                KeyCode::Delete => self.config.input.delete(),
                KeyCode::Left => self.config.input.left(),
                KeyCode::Right => self.config.input.right(),
                KeyCode::Home => self.config.input.cursor = 0,
                KeyCode::End => self.config.input.cursor = self.config.input.text.len(),
                _ => {}
            }
            return;
        }

        match code {
            KeyCode::Esc => self.change_view(View::Home),
            KeyCode::Up => {
                self.config.selected = self.config.selected.saturating_sub(1);
            }
            KeyCode::Down => {
                self.config.selected =
                    (self.config.selected + 1).min(self.config.items.len().saturating_sub(1));
            }
            KeyCode::Char('a' | 'A') => {
                let diff = self.config.pending_diff();
                if diff.is_empty() {
                    self.config.status = Some(book_toml_view::ConfigStatus::NoChanges);
                } else {
                    self.config.status = None;
                    self.config.confirm_apply = true;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                enum Action {
                    Toggle,
                    Cycle,
                    Edit(String),
                }
                let action = self.config.current_item().map(|item| match &item.kind {
                    book_toml::ItemKind::Bool(_) => Action::Toggle,
                    book_toml::ItemKind::Enum(_, _) => Action::Cycle,
                    book_toml::ItemKind::Text(_) => Action::Edit(item.kind.display_value()),
                    book_toml::ItemKind::StringList(list) => Action::Edit(list.join(", ")),
                });
                match action {
                    Some(Action::Toggle) => self.config.toggle_selected(),
                    Some(Action::Cycle) => self.config.cycle_selected(1),
                    Some(Action::Edit(text)) => {
                        self.config.editing = true;
                        self.config.input.text = text;
                        self.config.input.cursor = self.config.input.text.len();
                    }
                    None => {}
                }
            }
            KeyCode::Left => self.config.cycle_selected(-1),
            KeyCode::Right => self.config.cycle_selected(1),
            _ => {}
        }
    }

    fn on_plotgen_key(&mut self, code: KeyCode) {
        if code == KeyCode::Esc {
            self.change_view(View::Home);
            return;
        }
        let form = &mut self.gen_view.state;
        match code {
            KeyCode::Up => {
                if form.selected > 0 {
                    form.selected -= 1;
                }
            }
            KeyCode::Down => {
                let len = form.current_type().fields.len();
                if len > 0 {
                    form.selected = (form.selected + 1).min(len - 1);
                }
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if let Some(idx) = c.to_digit(10) {
                    form.set_type(idx as usize - 1);
                }
            }
            KeyCode::Char('p' | 'P') => form.cycle_output(),
            KeyCode::Char('c' | 'C') => form.copy(&mut self.clipboard),
            KeyCode::Char('s' | 'S') => form.save(),
            KeyCode::Char('r' | 'R') => form.reset_to_example(),
            KeyCode::Char('g' | 'G') if self.opts.no_preview => form.regen(),
            KeyCode::Enter | KeyCode::Char(' ') => {
                let i = form.selected;
                if form
                    .current_type()
                    .fields
                    .get(i)
                    .is_some_and(|f| f.is_bool())
                {
                    form.toggle_bool(i);
                }
            }
            KeyCode::Left => {
                let i = form.selected;
                if form
                    .current_type()
                    .fields
                    .get(i)
                    .is_some_and(|f| f.is_enum())
                {
                    form.cycle_enum(i, -1);
                }
            }
            KeyCode::Right => {
                let i = form.selected;
                if form
                    .current_type()
                    .fields
                    .get(i)
                    .is_some_and(|f| f.is_enum())
                {
                    form.cycle_enum(i, 1);
                }
            }
            KeyCode::Char(c) => {
                let i = form.selected;
                if form
                    .current_type()
                    .fields
                    .get(i)
                    .is_some_and(|f| !f.is_bool() && !f.is_enum())
                {
                    let mut text = form.inputs[i].text.clone();
                    text.push(c);
                    form.set_text_field(i, text);
                }
            }
            KeyCode::Backspace => {
                let i = form.selected;
                if form
                    .current_type()
                    .fields
                    .get(i)
                    .is_some_and(|f| !f.is_bool() && !f.is_enum())
                {
                    let mut text = form.inputs[i].text.clone();
                    text.pop();
                    form.set_text_field(i, text);
                }
            }
            _ => {}
        }
    }

    // -- main loop ----------------------------------------------------------

    /// Consume the generator's stdout fallback (clipboard unavailable),
    /// printed by the parent module after the alternate screen is left.
    pub(super) fn take_gen_output(&mut self) -> Option<String> {
        self.gen_view.state.print_on_exit.take()
    }

    pub fn run(&mut self, terminal: &mut TerminalType) -> std::io::Result<()> {
        let mut last_frame = Instant::now();
        while !self.quit {
            let dt = last_frame.elapsed();
            last_frame = Instant::now();
            self.update.tick(dt);
            self.drain_channels();
            terminal.draw(|frame| self.render(frame, dt))?;
            self.poll_events()?;
        }
        Ok(())
    }

    fn poll_events(&mut self) -> std::io::Result<()> {
        while event::poll(Duration::from_millis(EVENT_POLL_MS))? {
            match event::read()? {
                Event::Key(key) => self.on_key(key),
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    // -- rendering ----------------------------------------------------------

    fn render(&mut self, frame: &mut Frame, dt: Duration) {
        let area = frame.area();
        frame.render_widget(
            ratatui::widgets::Paragraph::new("").style(Style::new().bg(theme::base_bg())),
            area,
        );

        match self.view {
            View::Home => self.home.render(frame, area),
            View::Update => self.update.render(frame, area),
            View::Config => self.config.render(frame, area),
            View::PlotGen => self.gen_view.render(frame, area),
        }

        // Config diff modal (over the config view).
        if self.view == View::Config && self.config.confirm_apply {
            let diff = self.config.pending_diff();
            self.config.render_diff_modal(frame, area, &diff);
        }

        // Bottom key hints.
        let hints = self.key_hints();
        widget::key_bar(
            frame,
            Rect {
                x: area.x,
                y: area.bottom().saturating_sub(1),
                width: area.width,
                height: 1,
            },
            &hints,
        );

        if self.help {
            help::render(frame, area);
        }

        // Effects.
        if let Some(effect) = self.transition.as_mut()
            && effect
                .process(dt.into(), frame.buffer_mut(), area)
                .is_some()
        {
            self.transition = None;
        }
        if let Some(effect) = self.flash.as_mut()
            && effect
                .process(dt.into(), frame.buffer_mut(), area)
                .is_some()
        {
            self.flash = None;
        }
    }

    fn key_hints(&self) -> Vec<(&'static str, &'static str)> {
        let base = [("Esc", "home"), ("?", "help"), ("q", "quit")];
        let mut hints: Vec<(&'static str, &'static str)> = base.to_vec();
        match self.view {
            View::Home => {
                hints.insert(0, ("↑↓", "select"));
                hints.insert(1, ("Enter", "open"));
            }
            View::Update => {
                hints.insert(0, ("c", "check"));
                if self.update.is_confirming() {
                    hints.insert(0, ("y/n", "confirm"));
                }
            }
            View::Config => {
                hints.insert(0, ("↑↓", "select"));
                hints.insert(1, ("Enter", "edit"));
                hints.insert(2, ("a", "apply"));
            }
            View::PlotGen => {
                hints.insert(0, ("↑↓", "move"));
                hints.insert(1, ("1-8", "type"));
                hints.insert(2, ("p", "fmt"));
                hints.insert(3, ("c", "copy"));
                hints.insert(4, ("s", "save"));
            }
        }
        hints
    }
}
