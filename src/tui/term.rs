//! Terminal lifecycle: raw mode + alternate screen setup/teardown and the
//! mandatory panic hook that restores the terminal if anything goes wrong.

use crossterm::cursor;
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;

pub fn setup() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    crate::tui::theme::init_palette(crate::tui::theme::detect_color_level());
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

pub fn teardown(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let _ = terminal.show_cursor();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    disable_raw_mode()
}

/// Best-effort restoration callable from a panic hook.
pub fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    let _ = execute!(io::stdout(), cursor::Show);
}

/// Install a panic hook that leaves raw mode / the alternate screen before
/// the default handler prints the panic. Without this, a panic leaves the
/// user's terminal unusable.
pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        previous(info);
    }));
}
