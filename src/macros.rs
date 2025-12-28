/// Used to send critical errors.
/// Will exit directly with exit code `1`.
#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => {{
        let msg = format_args!($($arg)*);
        log::error!("Critical error: {}", msg);
        #[cfg(debug_assertions)]
        {
            log::debug!("Backtrace: {:?}", std::backtrace::Backtrace::capture());
        }
        std::process::exit(1);
    }};
}

/// Used to send critical errors in initiation.
/// Will exit directly with exit code `1`.
///
/// NOTE: This is done for something that was initialized earlier. They may be loaded before the log, so rely on the macro for output.
#[macro_export]
macro_rules! fatal_in_init {
    ($($arg:tt)*) => {{
        let msg = format_args!($($arg)*);
        eprintln!("Critical error in init: {}", msg);
        #[cfg(debug_assertions)]
        {
            eprintln!("Backtrace: {:?}", std::backtrace::Backtrace::capture());
        }
        std::process::exit(1);
    }};
}
