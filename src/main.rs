mod command;
mod macros;

pub fn main() {
    set_logger();
    start_up();
}

fn start_up() {
    let args = command::ReceivedArgs::receive()
        .unwrap_or_else(|e| fatal!("Parameter error: {:?}", e));

    use command::CommandKind;
    match args.command {
        CommandKind::Supports { renderer } => {
            match renderer.as_str() {
                // These two are built-in backends.
                // Other backends are not currently supported.
                "html" | "markdown" => todo!(),
                _ => todo!(),
            }
        }
    }
}

fn set_logger() {
    let mut builder = colog::default_builder();
    if cfg!(debug_assertions) {
        builder.filter_level(log::LevelFilter::Debug);
    }
    builder.init();
    log::debug!("The logger is configured.");
}
