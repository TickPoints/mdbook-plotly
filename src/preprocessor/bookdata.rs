use super::config::SUPPORTED_MDBOOK_VERSION;
use crate::fatal;
use crate::preprocessor::config::PreprocessorConfig;
use log::{error, warn};
use mdbook_preprocessor::{
    PreprocessorContext,
    book::{Book, BookItem, Chapter},
};
#[cfg(feature = "sync")]
use rayon::prelude::*;

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

    /// NOTE: This interface returns a cloned internal `PathBuf`.
    pub fn get_book_path(&self) -> std::path::PathBuf {
        self.ctx.root.clone()
    }

    #[allow(unused)]
    pub fn for_each_chapter_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Chapter),
    {
        fn walk<F: FnMut(&mut Chapter)>(items: &mut [BookItem], f: &mut F) {
            for item in items.iter_mut() {
                if let BookItem::Chapter(ch) = item {
                    f(ch);
                    walk(&mut ch.sub_items, f);
                }
            }
        }
        walk(&mut self.book.items, &mut f);
    }

    #[cfg(feature = "sync")]
    pub fn for_each_chapter_par<F>(&mut self, f: F)
    where
        F: Fn(&mut Chapter) + Sync + Send,
    {
        fn walk_par<F: Fn(&mut Chapter) + Sync + Send>(items: &mut [BookItem], f: &F) {
            items.par_iter_mut().for_each(|item| {
                if let BookItem::Chapter(ch) = item {
                    f(ch);
                    walk_par(&mut ch.sub_items, f);
                }
            });
        }
        walk_par(&mut self.book.items, &f);
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

    let config = match ctx.config.get::<PreprocessorConfig>("preprocessor.plotly") {
        Ok(Some(cfg)) => cfg,
        Ok(None) => {
            warn!("Custom config not found; using default configuration.");
            PreprocessorConfig::default()
        }
        Err(e) => {
            error!(
                "Illegal config format for 'preprocessor.mdbook-plotly': {}",
                e.root_cause()
            );
            PreprocessorConfig::default()
        }
    };

    BookData::new(ctx, book, config)
}
