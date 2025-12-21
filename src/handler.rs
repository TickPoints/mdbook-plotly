use crate::fatal;
use log::warn;
use mdbook_preprocessor::book::BookItem;

#[derive(Debug)]
pub struct BookData {
    preprocessor_config: PreprocessorConfig,
    mdbook_version: String,
    book_items: Vec<BookItem>,
}

impl BookData {
    fn new(
        preprocessor_config: PreprocessorConfig,
        mdbook_version: String,
        book_items: Vec<BookItem>,
    ) -> Self {
        Self {
            preprocessor_config,
            mdbook_version,
            book_items,
        }
    }

    pub fn check_version(&self) -> bool {
        // HARDCODE: MDBOOK_VERSION: 0.5.2
        self.mdbook_version == "0.5.2"
    }

    pub fn get_version(&self) -> &str {
        &self.mdbook_version
    }

    // Alias: `get_version`
    #[allow(dead_code)]
    pub fn version(&self) -> &str {
        self.get_version()
    }

    pub(crate) fn send_version_note(&self) {
        if !self.check_version() {
            warn!(
                // HARDCODE: MDBOOK_VERSION: 0.5.2
                "This crate was developed using version: `0.5.2`, and {} may not work",
                self.get_version()
            );
        }
    }

    pub fn get_config(&self) -> &PreprocessorConfig {
        &self.preprocessor_config
    }

    // Alias: `get_config`
    #[allow(dead_code)]
    pub fn config(&self) -> &PreprocessorConfig {
        self.get_config()
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PreprocessorConfig {}

impl Default for PreprocessorConfig {
    fn default() -> Self {
        Self {}
    }
}

fn get_bookdata() -> BookData {
    let (ctx, book) = mdbook_preprocessor::parse_input(std::io::stdin())
        .unwrap_or_else(|e| fatal!("Input parsing failed.\nInterError: {:#?}", e));
    let raw_config = ctx
        .config
        // HARDCODE: `preprocessor.mdbook-plotly` is a hard-coded config path.
        // Considering that this version is only a feasibility test, no complex check is made.
        .get::<PreprocessorConfig>("preprocessor.mdbook-plotly")
        .unwrap_or_else(|e| fatal!("Illegal Config.\nInterError: {:#?}", e))
        .unwrap_or_else(|| {
            warn!("Custom Config cannot be used, default has been substituted.");
            PreprocessorConfig::default()
        });
    BookData::new(raw_config, ctx.mdbook_version, book.items)
}

pub(crate) fn handle_book() {
    let bookdata = get_bookdata();
    bookdata.send_version_note();
}
