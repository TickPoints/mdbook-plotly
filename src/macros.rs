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

/// Used to translate `serde_json::Value` into T.
#[macro_export]
macro_rules! translate {
    ($target:expr, $value:expr, $map:expr, $(($method:ident, $ty:ty)),* $(,)?) => {{
        use $crate::preprocessor::handlers::code_handler::until::DataPack;
        let target = $target;
        $(
            let target = if let Some(v) = $value.get_mut(stringify!($method)) {
                let data = serde_json::from_value::<DataPack<$ty>>(v.take())?;
                target.$method(data.unwrap($map)?)
            } else {
                target
            };
        )*
        Ok::<_, serde_json::Error>(target)
    }};
}
