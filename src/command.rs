use clap::{Arg, Command, command, error::ErrorKind};

#[derive(Debug)]
pub enum CommandKind {
    Supports { renderer: String },
    ProcessBook,
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
    command!().subcommand(
        Command::new("supports")
            .arg(
                Arg::new("renderer")
                    .required(true)
                    .help("The renderer to check support for"),
            )
            .about("Check whether a renderer is supported by this preprocessor"),
    )
}
