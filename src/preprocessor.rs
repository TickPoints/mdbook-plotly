mod bookdata;
mod config;
mod handlers;

use crate::fatal;

#[cfg(feature = "js-tool")]
mod js_tool;

pub fn preprocess_book() {
    let mut book_data = bookdata::get_book_data();

    book_data.emit_compatibility_warning();

    let config = book_data.get_config();
    for chapter in book_data.chapter_iter_mut() {
        handlers::handle(chapter, &config);
    }

    let (ctx, book) = book_data.into_parts();

    if let Err(e) = serde_json::to_writer(std::io::stdout(), &(ctx, book)) {
        fatal!("Write bookdata failed.\nInterError: {:#?}", e);
    }
}
