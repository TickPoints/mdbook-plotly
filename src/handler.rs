mod chapter_handler;

use crate::fatal;
use log::warn;
use mdbook_preprocessor::{
    PreprocessorContext,
    book::{Book, BookItem, Chapter},
};
use serde::{Deserialize, Serialize};
use std::iter::Iterator;

const SUPPORTED_MDBOOK_VERSION: &str = "0.5.2";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct PreprocessorConfig {}
// Todo: The above content is to prepare for the subsequent development.

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

    /// Note: This interface returns a cloned internal `PreprocessorConfig`.
    pub fn get_config(&self) -> PreprocessorConfig {
        self.config.clone()
    }

    pub fn chapter_iter_mut(&mut self) -> impl Iterator<Item = &mut Chapter> {
        self.book.items.iter_mut().filter_map(|item| {
            if let BookItem::Chapter(chapter) = item {
                Some(chapter)
            } else {
                None
            }
        })
    }

    pub fn into_parts(self) -> (PreprocessorContext, Book) {
        (self.ctx, self.book)
    }
}

fn get_book_data() -> BookData {
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
            warn!("Custom config not found; using default configuration.");
            PreprocessorConfig::default()
        }
        Err(e) => fatal!(
            "Illegal config format for 'preprocessor.mdbook-plotly'.\nInterError: {:#?}",
            e
        ),
    };

    BookData::new(ctx, book, config)
}

pub(crate) fn handle_book() {
    let mut book_data = get_book_data();

    book_data.emit_compatibility_warning();

    let config = book_data.get_config();
    for chapter in book_data.chapter_iter_mut() {
        if let Err(e) = chapter_handler::handle(chapter, &config) {
            warn!("Error processing chapter '{}': {:?}", chapter.name, e);
        }
    }

    let (ctx, book) = book_data.into_parts();

    if let Err(e) = serde_json::to_writer(std::io::stdout(), &(ctx, book)) {
        fatal!("Write bookdata failed.\nInterError: {:#?}", e);
    }
}
