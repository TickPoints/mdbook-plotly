use super::config::SUPPORTED_MDBOOK_VERSION;
use crate::fatal;
use crate::preprocessor::config::PreprocessorConfig;
use log::{debug, warn};
use mdbook_preprocessor::{
    PreprocessorContext,
    book::{Book, BookItem, Chapter},
};
use rayon::prelude::*;
use std::iter::Iterator;

pub struct BookData {
    ctx: PreprocessorContext,
    book: Book,
    config: PreprocessorConfig,
}

impl BookData {
    fn new(ctx: PreprocessorContext, book: Book, config: PreprocessorConfig) -> Self {
        Self { ctx, book, config }
    }

    pub fn version(&self) -> &str {
        &self.ctx.mdbook_version
    }

    pub fn is_compatible_version(&self) -> bool {
        self.version() == SUPPORTED_MDBOOK_VERSION
    }

    pub fn emit_compatibility_warning(&self) {
        if !self.is_compatible_version() {
            warn!(
                "This preprocessor was developed for mdbook v{}, but found v{}. May not work as expected.",
                SUPPORTED_MDBOOK_VERSION,
                self.version()
            );
        }
    }

    /// NOTE: This interface returns a cloned internal `PreprocessorConfig`.
    pub fn get_config(&self) -> PreprocessorConfig {
        self.config.clone()
    }

    /// NOTE: This interface is actually used in non-sync situations. But it's always there.
    #[allow(dead_code)]
    pub fn chapter_iter_mut(&mut self) -> impl Iterator<Item = &mut Chapter> {
        self.book.items.iter_mut().filter_map(|item| {
            if let BookItem::Chapter(chapter) = item {
                Some(chapter)
            } else {
                None
            }
        })
    }

    #[cfg(feature = "sync")]
    pub fn chapter_par_iter(&mut self) -> impl ParallelIterator<Item = &mut Chapter> {
        self.book.items.par_iter_mut().filter_map(|item| {
            if let BookItem::Chapter(chapter) = item {
                Some(chapter)
            } else {
                None
            }
        })
    }

    /// NOTE: Abandoned.
    #[allow(dead_code)]
    pub fn into_parts(self) -> (PreprocessorContext, Book) {
        (self.ctx, self.book)
    }

    pub fn into_book(self) -> Book {
        self.book
    }
}

pub fn get_book_data() -> BookData {
    let (ctx, book) = match mdbook_preprocessor::parse_input(std::io::stdin()) {
        Ok(parsed) => parsed,
        Err(e) => fatal!("Input parsing failed.\nInterError: {:#?}", e),
    };

    let config = match ctx
        .config
        .get::<PreprocessorConfig>("preprocessor.mdbook-plotly")
    {
        Ok(Some(cfg)) => cfg,
        Ok(None) => {
            debug!("Custom config not found; using default configuration.");
            PreprocessorConfig::default()
        }
        Err(e) => fatal!(
            "Illegal config format for 'preprocessor.mdbook-plotly'.\nInterError: {:#?}",
            e
        ),
    };

    BookData::new(ctx, book, config)
}
