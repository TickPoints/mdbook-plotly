/// Used to send critical errors.
/// Will exit directly with exit code `1`.
///
/// NOTE: This macro is only useful after Rust version `1.79`.
#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => {{
        // This line only compiles smoothly after version `1.79`.
        // Compiling with older versions may result in temporary value errors.
        //
        // If you need some information, here it is:
        // Issue: [#92698](https://github.com/rust-lang/rust/issues/92698)
        // Tracing issues with RFC66: [#15023](https://github.com/rust-lang/rust/issues/15023)
        // The std doc about `Arguments`: [`std::fmt::Arguments`](https://doc.rust-lang.org/stable/std/fmt/struct.Arguments.html)
        let msg = format_args!($($arg)*);

        log::error!("Critical error: {}", msg);
        #[cfg(debug_assertions)]
        {
            log::debug!("Backtrace: {:?}", std::backtrace::Backtrace::capture());
        }
        std::process::exit(1);
    }};
}

/// Used to translate `serde_json::Value` into `DataPack<T>`.
/// This macro avoids writing a lot of duplicate code.
#[macro_export]
macro_rules! translate {
    ($target:expr, $value:expr, $map:expr, $(($method:ident, $ty:ty)),* $(,)?) => {{
        use $crate::preprocessor::handlers::code_handler::until::DataPack;
        let target = $target;
        $(
            let target = if let Some(v) = $value.get_mut(stringify!($method)) {
                let data = serde_json::from_value::<DataPack<$ty>>(v.take())
                    .map_err(|e| ::anyhow::anyhow!("Failed to deserialize field '{}': {}", stringify!($method), e))?;
                target.$method(data.unwrap($map)
                    .map_err(|e| ::anyhow::anyhow!("Failed to unwrap DataPack for field '{}': {}", stringify!($method), e))?)
            } else {
                target
            };
        )*
        Ok::<_, ::anyhow::Error>(target)
    }};
}

/// Used to translate string values in `serde_json::Value` into enum variants
/// via `DataPack<String>`, avoiding a lot of duplicate match-and-build code.
#[macro_export]
macro_rules! translate_enum {
    ($target:expr, $value:expr, $map:expr, $(
        ($method:ident, { $($str_val:literal => $variant:expr),* $(,)? })
    ),* $(,)?) => {{
        use $crate::preprocessor::handlers::code_handler::until::DataPack;
        let target = $target;
        $(
            let target = if let Some(v) = $value.get_mut(stringify!($method)) {
                let data = serde_json::from_value::<DataPack<String>>(v.take())
                    .map_err(|e| ::anyhow::anyhow!("Failed to deserialize field '{}': {}", stringify!($method), e))?;
                let s = data.unwrap($map)
                    .map_err(|e| ::anyhow::anyhow!("Failed to unwrap DataPack for field '{}': {}", stringify!($method), e))?;
                match s.as_str() {
                    $($str_val => target.$method($variant),)*
                    unexpected => {
                        return Err(::anyhow::anyhow!(
                            "\"{}\" is not a valid value for `{}`",
                            unexpected,
                            stringify!($method),
                        ))
                    }
                }
            } else {
                target
            };
        )*
        Ok::<_, ::anyhow::Error>(target)
    }};
}
