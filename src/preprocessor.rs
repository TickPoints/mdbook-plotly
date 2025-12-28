mod bookdata;
mod config;
mod handlers;

use crate::fatal;

#[cfg(feature = "js-tools")]
mod js_tools;

pub fn preprocess_book() {
    let mut book_data = bookdata::get_book_data();

    book_data.emit_compatibility_warning();

    let config = book_data.get_config();
    #[cfg(feature = "js-tools")]
    {
        use rayon::prelude::*;

        book_data
            .chapter_iter_mut()
            .par_iter()
            .for_each(|chapter| handlers::handle(chapter, &config));
    }
    #[cfg(not(feature = "js-tools"))]
    {
        book_data
            .chapter_iter_mut()
            .for_each(|chapter| handlers::handle(chapter, &config));
    }

    let (ctx, book) = book_data.into_parts();

    if let Err(e) = serde_json::to_writer(std::io::stdout(), &(ctx, book)) {
        fatal!("Write bookdata failed.\nInterError: {:#?}", e);
    }
}
