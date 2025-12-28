//! Used to send critical errors.
//! Will exit directly with exit code `1`.
#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => {{
        let msg = format_args!($($arg)*);
        log::error!("Critical error: {}", msg);
        #[cfg(debug_assertions)]
        {
            eprintln!("Backtrace: {:?}", std::backtrace::Backtrace::capture());
        }
        std::process::exit(1);
    }};
}
