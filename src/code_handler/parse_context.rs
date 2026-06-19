use super::until::Map;
use crate::preprocessor::config::MapEvalConfig;

pub struct ParseContext<'a> {
    map: &'a Map,
    map_eval: &'a MapEvalConfig,
}

impl<'a> ParseContext<'a> {
    pub fn new(map: &'a Map, map_eval: &'a MapEvalConfig) -> Self {
        Self { map, map_eval }
    }

    pub fn map(&self) -> &'a Map {
        self.map
    }

    pub fn map_eval(&self) -> &'a MapEvalConfig {
        self.map_eval
    }
}
