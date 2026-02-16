use std::path::Path;
use plotly::{ImageFormat, Plot};
use pulldown_cmark::{CowStr, Event};
use rand::Rng;

fn rand_name(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

pub fn handle(code: Plot, book_path: &Path) -> Event<'static> {
    let name = rand_name(10);
    let path = book_path.join(format!("{}/{}.svg", "book", name));
    code.write_image(
        path,
        ImageFormat::SVG,
        800,
        600,
        1.0,
    )?;
    Event::Html(CowStr::from(format!(
        "<img src=\"/{}.svg\" alt=\"{}\"/>",
        name, name
    )))
}
