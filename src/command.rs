use clap::{Arg, Command, command, error::ErrorKind};

#[derive(Debug)]
pub enum CommandKind {
    Supports {
        renderer: String,
    },
    ProcessBook,
    #[cfg(feature = "tui")]
    Tui {
        dry_run: bool,
        refresh: bool,
        no_effects: bool,
    },
    /// The binary was built without the `tui` feature; the subcommand is
    /// recognized but not implemented.
    #[cfg(not(feature = "tui"))]
    TuiNotAvailable,
}

#[derive(Debug)]
pub(crate) struct ReceivedArgs {
    pub command: CommandKind,
}

impl ReceivedArgs {
    pub(crate) fn receive() -> Result<Self, clap::Error> {
        let matches = make_app().get_matches();

        let command = match matches.subcommand() {
            Some(("supports", sub_m)) => {
                let renderer = sub_m
                    .get_one::<String>("renderer")
                    .unwrap_or_else(|| {
                        // SAFETY: `renderer` is required and thus always present
                        unreachable!()
                    })
                    .to_string();
                CommandKind::Supports { renderer }
            }
            #[cfg(feature = "tui")]
            Some(("tui", sub_m)) => CommandKind::Tui {
                dry_run: sub_m.get_flag("dry-run"),
                refresh: sub_m.get_flag("refresh"),
                no_effects: sub_m.get_flag("no-effects"),
            },
            #[cfg(not(feature = "tui"))]
            Some(("tui", _)) => CommandKind::TuiNotAvailable,
            None => CommandKind::ProcessBook,
            _ => {
                return Err(clap::Error::raw(
                    ErrorKind::InvalidSubcommand,
                    "Unknown subcommand",
                ));
            }
        };

        Ok(ReceivedArgs { command })
    }
}

fn make_app() -> Command {
    let tui = Command::new("tui")
        .about("Interactive tools: self-update, book.toml editor, plot cheat-sheet");
    #[cfg(feature = "tui")]
    let tui = tui
        .arg(
            Arg::new("dry-run")
                .long("dry-run")
                .action(clap::ArgAction::SetTrue)
                .help("Check for updates without downloading or replacing anything"),
        )
        .arg(
            Arg::new("refresh")
                .long("refresh")
                .action(clap::ArgAction::SetTrue)
                .help("Force a refresh of the cached cheat-sheet document"),
        )
        .arg(
            Arg::new("no-effects")
                .long("no-effects")
                .action(clap::ArgAction::SetTrue)
                .help("Disable transition animations"),
        );

    command!()
        .subcommand(
            Command::new("supports")
                .arg(
                    Arg::new("renderer")
                        .required(true)
                        .help("The renderer to check support for"),
                )
                .about("Check whether a renderer is supported by this preprocessor"),
        )
        .subcommand(tui)
}
