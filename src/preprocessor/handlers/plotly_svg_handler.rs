use crate::preprocessor::js_tools;
use pulldown_cmark::Event;

pub fn handle(code: String) -> Result<Event<'static>, js_tools::ScriptError> {
    js_tools::with_js_runtime(|runtime| {
        let context = js_tools::get_sandboxed_context(runtime)?;
        context.with(|local_ctx| -> Result<(), js_tools::ScriptError> {
            js_tools::eval_script::<(), _>(&local_ctx, include_str!("plotly.js"))?;
            Ok(())
        })?;
        Ok(Event::Text("empty".into()))
    })
}
