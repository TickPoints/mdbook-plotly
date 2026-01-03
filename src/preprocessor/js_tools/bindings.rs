//! These are some JS functions implemented in rust.
//! The principle is to implement some external functions through Rust and bind them to Javascript.
//!
//! ### Purpose
//! - Functions that implement other Js standards not supported by QuickJs.
use rquickjs::{Value, Function, function as into_js_function};
use super::QuickjsError;
use std::thread::sleep;
use std::time::Duration;

#[into_js_function]
pub fn set_timeout<'js>(callback: Function<'js>, delay: u64) -> Result<Value<'js>, QuickjsError> {
    sleep(Duration::from_millis(delay));

    let result = callback.call(())?;

    Ok(result)
}
