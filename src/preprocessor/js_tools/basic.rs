#![allow(dead_code)]
use super::QuickjsError;
use rquickjs::function::{FromParams, Func, IntoJsFunc};
use rquickjs::{IntoAtom, IntoJs, Object, object::Accessor};

#[derive(Debug)]
pub enum ScriptError {
    ScriptError(String),
    InterError(QuickjsError),
}

use std::error::Error;
impl Error for ScriptError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Self::InterError(inter_error) = self {
            Some(inter_error)
        } else {
            None
        }
    }
}

use std::fmt;
impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ScriptError(e) => write!(f, "Js Tools Error: Script Error: {}", e),
            Self::InterError(e) => write!(f, "Js Tools Error: Inter Error: {:?}", e),
        }
    }
}

impl From<QuickjsError> for ScriptError {
    fn from(v: QuickjsError) -> Self {
        Self::InterError(v)
    }
}

/// Converts an optional string value into its JavaScript string representation.
/// Returns `"undefined"` if the value is `None`, otherwise returns the string value.
#[inline]
pub fn to_js_optional_string<T>(value: T) -> String
where
    T: Into<Option<String>>,
{
    value.into().unwrap_or_else(|| "undefined".to_string())
}

/// Converts any JS value.
#[inline]
pub fn stringify_js_value(value: rquickjs::Value) -> String {
    to_js_optional_string(
        value
            .into_string()
            .map(|s| s.to_string().unwrap_or("undefined".to_string())),
    )
}

/// Used to inject read-only data into an object.
pub fn inject_readonly<'a, 'js, K, V>(
    obj: &'a Object<'js>,
    key: K,
    v: V,
) -> Result<(), QuickjsError>
where
    K: IntoAtom<'js>,
    V: IntoJs<'js> + Clone + 'js,
{
    obj.prop(key, Accessor::new_get(move || v.clone()))
}

/// Used to inject read-only function into an object.
pub fn inject_function<'a, 'js, K, F, A>(
    obj: &'a Object<'js>,
    key: K,
    f: F,
) -> Result<(), QuickjsError>
where
    K: IntoAtom<'js>,
    F: IntoJsFunc<'js, A> + Copy + 'js,
    A: FromParams<'js> + 'js,
{
    obj.prop(key, Accessor::new_get(move || Func::new(f)))
}
