mod bookdata;
mod config;
mod handlers;

use crate::fatal;

pub fn preprocess_book() {
    let mut book_data = bookdata::get_book_data();

    book_data.emit_compatibility_warning();

    let config = book_data.get_config();
    #[cfg(feature = "sync")]
    {
        use rayon::prelude::*;
        book_data
            .chapter_par_iter()
            .for_each(|chapter| handlers::handle(chapter, &config));
    }
    #[cfg(not(feature = "sync"))]
    {
        book_data
            .chapter_iter_mut()
            .for_each(|chapter| handlers::handle(chapter, &config));
    }

    let preprocessed_book = book_data.into_book();

    if let Err(e) = serde_json::to_writer(std::io::stdout(), &preprocessed_book) {
        fatal!("Write bookdata failed.\nInterError: {:#?}", e);
    }
}
