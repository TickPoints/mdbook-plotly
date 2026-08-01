mod code_handler;
mod command;
mod macros;
mod preprocessor;

pub fn main() {
    set_logger();
    let args = command::ReceivedArgs::receive()
        .unwrap_or_else(|e| fatal!("Parameter error.\nInterError: {:#?}", e));

    use command::CommandKind;
    match args.command {
        CommandKind::Supports { renderer } => {
            // HARDCODE: These two are built-in backends.
            // Other backends are not currently supported.
            let supported = matches!(renderer.as_str(), "html" | "markdown");
            if supported {
                std::process::exit(0);
            } else {
                std::process::exit(1);
            }
        }
        CommandKind::ProcessBook => preprocessor::preprocess_book(),
        #[cfg(feature = "tui")]
        CommandKind::Tui {
            dry_run,
            refresh,
            no_effects,
        } => {
            let opts = mdbook_plotly::tui::TuiOptions {
                dry_run,
                refresh,
                no_effects,
            };
            if let Err(e) = mdbook_plotly::tui::run(opts) {
                fatal!("TUI error.\nInterError: {e:#}");
            }
        }
        #[cfg(not(feature = "tui"))]
        CommandKind::TuiNotAvailable => {
            let releases = format!("{}/releases", env!("CARGO_PKG_REPOSITORY"));
            eprintln!(
                "This binary is the slim edition of mdbook-plotly and does not include the `tui` command.\n\
                 To get the full edition with the interactive tools, either:\n\
                 \x20  •  cargo install mdbook-plotly --features tui\n\
                 \x20  •  download the release asset whose name ends with `-tui` from\n\
                 \x20     {releases}"
            );
            std::process::exit(1);
        }
    }
}

fn set_logger() {
    let mut builder = env_logger::builder();
    if cfg!(debug_assertions) {
        builder.filter_level(log::LevelFilter::Debug);
    } else {
        builder.filter_level(log::LevelFilter::Info);
    }
    builder.format(logger_format);
    builder.init();
}

use env_logger::fmt::style::{AnsiColor, Effects, Reset, RgbColor, Style};
use log::{Level, Record};
use std::io::Write;

pub fn logger_format(fmt: &mut env_logger::fmt::Formatter, record: &Record) -> std::io::Result<()> {
    write!(fmt, "[{}] ", fmt.timestamp())?;

    let level_style = match record.level() {
        Level::Error => Style::new()
            .fg_color(Some(AnsiColor::Red.into()))
            .effects(Effects::BOLD),
        Level::Warn => Style::new().fg_color(Some(AnsiColor::Yellow.into())),
        Level::Info => Style::new().fg_color(Some(AnsiColor::Green.into())),
        Level::Debug => Style::new().fg_color(Some(AnsiColor::Blue.into())),
        Level::Trace => Style::new().fg_color(Some(AnsiColor::Magenta.into())),
    };

    write!(
        fmt,
        "{}{:5}{Reset} ",
        level_style.render(),
        record.level().as_str()
    )?;

    if let Some(module_path) = record.module_path() {
        let crate_name = module_path.split("::").next().unwrap_or(module_path);
        write!(fmt, "{crate_name}: ")?;
    }

    write!(fmt, "{}", record.args())?;

    if record.level() >= Level::Debug {
        let gray_style = Style::new().fg_color(Some(RgbColor(128, 128, 128).into()));

        if let Some(file) = record.file() {
            let filename = file.split('/').next_back().unwrap_or(file);
            write!(
                fmt,
                " {}({}:{}){Reset}",
                gray_style.render(),
                filename,
                record.line().unwrap_or(0)
            )?;
        }
    }

    writeln!(fmt)?;
    Ok(())
}
