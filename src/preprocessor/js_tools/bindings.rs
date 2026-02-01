//! These are some JS functions implemented in rust.
//! The principle is to implement some external functions through Rust and bind them to Javascript.
//!
//! ### Purpose
//! - Functions that implement other Js standards not supported by QuickJs.
use super::{QuickjsError, basic::*};
use rquickjs::{Function, Value, function as into_js_function};
use std::thread::sleep;
use std::time::Duration;

#[into_js_function]
pub fn set_timeout<'js>(callback: Function<'js>, delay: u64) -> Result<Value<'js>, QuickjsError> {
    sleep(Duration::from_millis(delay));

    let result = callback.call(())?;

    Ok(result)
}

#[into_js_function]
pub fn log<'js>(_message: Value<'js>) {}

#[into_js_function]
pub fn debug<'js>(message: Value<'js>) {
    log::warn!(
        "Js Tools Error: Script Debug: {}",
        stringify_js_value(message)
    );
}

#[into_js_function]
pub fn warn<'js>(message: Value<'js>) {
    log::warn!(
        "Js Tools Error: Script Warn: {}",
        stringify_js_value(message)
    );
}

#[into_js_function]
pub fn error<'js>(message: Value<'js>) {
    log::warn!(
        "Js Tools Error: Script Error: {}",
        stringify_js_value(message)
    );
}
