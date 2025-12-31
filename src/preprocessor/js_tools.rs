use once_cell::sync::Lazy;
pub use rquickjs::Error as QuickjsError;
use rquickjs::{Context, Runtime};
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

pub fn with_js_runtime<F, R>(f: F) -> R
where
    F: FnOnce(&Runtime) -> R,
{
    f(GLOBAL_JS_RUNTIME.as_ref())
}

pub fn get_sandboxed_context(rt: &Runtime) -> Result<Context, QuickjsError> {
    let context = Context::base(rt)?;
    Ok(context)
}
