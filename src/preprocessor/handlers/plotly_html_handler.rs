use plotly::Plot;
use pulldown_cmark::Event;

pub fn handle(code: Plot) -> Event<'static> {
    Event::Html(code.to_html().into())
}
