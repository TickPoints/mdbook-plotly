use crate::preprocessor::js_tools;
use pulldown_cmark::Event;

pub fn handle(code: String) -> Result<Event<'_>, js_tools::QuickjsError> {
    js_tools::with_js_runtime(|runtime| {
        let context = js_tools::get_sandboxed_context(runtime)?;
        context.with(|local_ctx| {
            let plotly = local_ctx.eval(include_str!("plotly.js"))?;
            eprintln!("code: {code}");
        })?;
        Ok(Event::Text("empty".into()))
    })
}
