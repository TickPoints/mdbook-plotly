use once_cell::sync::Lazy;
use rquickjs::Runtime;
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
