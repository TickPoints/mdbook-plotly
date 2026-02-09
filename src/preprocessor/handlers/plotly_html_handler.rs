use plotly::Plot;
use pulldown_cmark::Event;

pub fn handle(code: Plot) -> Event<'static> {
    Event::Html(code.to_inline_html(None).into())
}

fn trim_leading_spaces(text: &mut String) {
    let mut out = String::with_capacity(text.len());
    for (idx, line) in text.lines().enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        out.push_str(line.trim_start());
    }
    *text = out;
}

pub fn inject_header(offline_js_sources: bool) -> Event<'static> {
    let mut html: String = if offline_js_sources {
        Plot::offline_js_sources()
    } else {
        Plot::online_cdn_js()
    };
    trim_leading_spaces(&mut html);
    Event::Html(html.into())
}
