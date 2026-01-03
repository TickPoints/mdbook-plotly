use crate::preprocessor::js_tools;
use pulldown_cmark::Event;

#[allow(clippy::let_unit_value)]
pub fn handle(code: String) -> Result<Event<'static>, js_tools::ScriptError> {
    js_tools::with_js_runtime(|runtime| {
        let context = js_tools::get_sandboxed_context(runtime)?;
        context.with(|local_ctx| -> Result<(), js_tools::ScriptError> {
            // NOTE: The seemingly useless binding is a must here.
            // It can avoid being wrongly evaluated as `!`.
            let plotly = js_tools::eval_script(&local_ctx, include_str!("plotly.js"))?;
            log::error!("code: {code}");
            Ok(plotly)
        })?;
        Ok(Event::Text("empty".into()))
    })
}
