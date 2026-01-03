use once_cell::sync::Lazy;
pub use rquickjs::Error as QuickjsError;
use rquickjs::{Context, Ctx, FromJs, IntoAtom, IntoJs, Object, Runtime, object::Accessor};
// basic

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
pub fn to_js_optional_string<T>(value: T) -> String
where
    T: Into<Option<String>>,
{
    value.into().unwrap_or_else(|| "undefined".to_string())
}

pub fn inject_readonly<'a, 'js, K, V>(
    obj: &'a Object<'js>,
    key: K,
    v: V,
) -> Result<(), QuickjsError>
where
    K: IntoAtom<'js>,
    V: IntoJs<'js> + Clone + 'js,
{
    let value = v.clone();
    obj.prop(key, Accessor::new_get(move || value.clone()))
}

// practical

#[cfg(not(feature = "sync"))]
use std::rc::Rc as SupportRc;
#[cfg(feature = "sync")]
use std::sync::Arc as SupportRc;
static GLOBAL_JS_RUNTIME: Lazy<SupportRc<Runtime>> = Lazy::new(|| {
    let rt = Runtime::new().unwrap_or_else(|e| {
        crate::fatal_in_init!(
            "Js Tools Error: Can't create runtime.\nInterError: {:#?}",
            e
        )
    });
    SupportRc::new(rt)
});

/// The interface receives a closure. The closure will be able to take the global runtime and use it.
///
/// NOTE: The obtained Runtime is Rcized, and the form of Rc depends on the `sync` feature.
pub fn with_js_runtime<F, R>(f: F) -> R
where
    F: FnOnce(&Runtime) -> R,
{
    f(GLOBAL_JS_RUNTIME.as_ref())
}

/// The interface allows a Runtime to be used to generate a sandboxed context.
///
/// As other features are refined, the sandbox will also be tighter.
/// Therefore, we do not recommend using JS features and global quantities that are not explicitly supported in other documents.
/// Because they may be silently deleted in the minor version.
pub fn get_sandboxed_context(rt: &Runtime) -> Result<Context, QuickjsError> {
    let context = Context::full(rt)?;
    context.with(|ctx| -> Result<(), QuickjsError> {
        // Inject content
        let globals = ctx.globals();
        use rquickjs::function::Func;
        Ok(())
    })?;
    Ok(context)
}

/// It is a wrapper around the `eval` function of the original `Ctx`.
/// It is convenient to collect JS errors at the same time.
pub fn eval_script<'js, V: FromJs<'js>, S: Into<Vec<u8>>>(
    ctx: &Ctx<'js>,
    source: S,
) -> Result<V, ScriptError> {
    let eval_result = ctx.eval::<V, _>(source);
    match eval_result {
        Err(rquickjs::Error::Exception) => {
            let formatted_exception = ctx
                .catch()
                .as_exception()
                .map(|exception| {
                    format!(
                        "{}\n{}",
                        to_js_optional_string(exception.message()),
                        to_js_optional_string(exception.stack())
                    )
                })
                .unwrap_or_else(|| String::from("Undefined exception."));
            Err(ScriptError::ScriptError(formatted_exception))
        }
        Err(e) => Err(ScriptError::InterError(e)),
        Ok(v) => Ok(v),
    }
}
