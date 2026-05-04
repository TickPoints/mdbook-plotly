use plotly::{ImageFormat, Plot};
use pulldown_cmark::{CowStr, Event};
use rand::distr::{Alphanumeric, SampleString};
use std::path::Path;

pub fn rand_name(length: usize) -> String {
    let mut rng = rand::rng();
    Alphanumeric.sample_string(&mut rng, length)
}

pub fn handle(code: Plot, book_path: &Path) -> Event<'static> {
    let name = rand_name(10);
    let path = book_path.join(format!("{}/{}.svg", "book", name));
    code.write_image(path, ImageFormat::SVG, 800, 600, 1.0)
        .expect("Failed to write image"); // tmp
    Event::Html(CowStr::from(format!(
        "<img src=\"/{}.svg\" alt=\"{}\"/>",
        name, name
    )))
}
