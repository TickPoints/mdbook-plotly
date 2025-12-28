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
            println!("{}", supported);
            std::process::exit(0);
        }
        CommandKind::ProcessBook => preprocessor::preprocess_book(),
    }
}

fn set_logger() {
    let mut builder = colog::default_builder();
    if cfg!(debug_assertions) {
        builder.filter_level(log::LevelFilter::Debug);
    }
    builder.init();
}
