use log::warn;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use toml::{Value, value::Table};

pub const SUPPORTED_MDBOOK_VERSION: &str = "0.5.2";
pub const PREPROCESSOR_CONFIG_KEY: &str = "preprocessor.plotly";

/// NOTE: These configurations are printed as kebab-case names. Please pay attention when using.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "kebab-case")]
pub struct PreprocessorConfig {
    /// About the output form of the chart.
    /// This output format may affect the presentation of the chart.
    ///
    /// In addition, in most cases, the different output forms can significantly affect the time at which the book is compiled.
    ///
    /// Other: The inner is an enumeration.
    pub output_type: PlotlyOutputType,

    /// About the input form of the chart.
    ///
    /// Charts are usually in the form of code in a markdown document. At the time of input, we allow the code to be presented in different forms.
    ///
    /// The two forms we consider for adoption are: a general script and a configuration file organized in a specific form. In theory, you can read and operate files directly from the current path by turning on some of the functions that come with MDBook.
    pub input_type: PlotlyInputType,

    /// Controls map expression evaluation behavior such as namespace visibility
    /// and whether fasteval optimizations should be enabled.
    pub map_eval: MapEvalConfig,
}

impl PreprocessorConfig {
    pub fn from_toml(value: &Value) -> Self {
        let Some(table) = value.as_table() else {
            warn!(
                "Illegal config format for '{}': expected a table; using default configuration.",
                PREPROCESSOR_CONFIG_KEY
            );
            return Self::default();
        };

        warn_unknown_keys(
            PREPROCESSOR_CONFIG_KEY,
            table,
            &["output-type", "input-type", "map-eval"],
        );

        Self {
            output_type: parse_field(
                table,
                "output-type",
                &format!("{}.output-type", PREPROCESSOR_CONFIG_KEY),
                Self::default().output_type,
            ),
            input_type: parse_field(
                table,
                "input-type",
                &format!("{}.input-type", PREPROCESSOR_CONFIG_KEY),
                Self::default().input_type,
            ),
            map_eval: match table.get("map-eval") {
                Some(map_eval) => MapEvalConfig::from_toml(map_eval),
                None => MapEvalConfig::default(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, rename_all = "kebab-case")]
pub struct MapEvalConfig {
    pub enabled: bool,
    pub reuse_slab: bool,
    pub compile_expressions: bool,
    pub namespace_scope: MapNamespaceScope,
}

impl MapEvalConfig {
    pub fn from_toml(value: &Value) -> Self {
        let Some(table) = value.as_table() else {
            warn!(
                "Illegal config format for '{}.map-eval': expected a table; using default configuration.",
                PREPROCESSOR_CONFIG_KEY
            );
            return Self::default();
        };

        warn_unknown_keys(
            &format!("{}.map-eval", PREPROCESSOR_CONFIG_KEY),
            table,
            &[
                "enabled",
                "reuse-slab",
                "compile-expressions",
                "namespace-scope",
            ],
        );

        Self {
            enabled: parse_field(
                table,
                "enabled",
                &format!("{}.map-eval.enabled", PREPROCESSOR_CONFIG_KEY),
                Self::default().enabled,
            ),
            reuse_slab: parse_field(
                table,
                "reuse-slab",
                &format!("{}.map-eval.reuse-slab", PREPROCESSOR_CONFIG_KEY),
                Self::default().reuse_slab,
            ),
            compile_expressions: parse_field(
                table,
                "compile-expressions",
                &format!("{}.map-eval.compile-expressions", PREPROCESSOR_CONFIG_KEY),
                Self::default().compile_expressions,
            ),
            namespace_scope: parse_field(
                table,
                "namespace-scope",
                &format!("{}.map-eval.namespace-scope", PREPROCESSOR_CONFIG_KEY),
                Self::default().namespace_scope,
            ),
        }
    }
}

fn parse_field<T>(table: &Table, key: &str, path: &str, default: T) -> T
where
    T: DeserializeOwned,
{
    match table.get(key) {
        Some(value) => deserialize_value(value).unwrap_or_else(|e| {
            warn!(
                "Failed to parse config field '{}': {}; using default value.",
                path, e
            );
            default
        }),
        None => default,
    }
}

fn deserialize_value<T>(value: &Value) -> Result<T, toml::de::Error>
where
    T: DeserializeOwned,
{
    #[derive(Deserialize)]
    struct Wrapper<T> {
        value: T,
    }

    toml::from_str::<Wrapper<T>>(&format!("value = {}", value)).map(|wrapper| wrapper.value)
}

fn warn_unknown_keys(path: &str, table: &Table, known_keys: &[&str]) {
    for key in table.keys() {
        if !known_keys.contains(&key.as_str()) {
            warn!("Unknown config key '{}.{}' will be ignored.", path, key);
        }
    }
}

impl Default for MapEvalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            reuse_slab: true,
            compile_expressions: true,
            namespace_scope: MapNamespaceScope::FullMap,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MapNamespaceScope {
    #[default]
    #[serde(rename = "full-map")]
    FullMap,
    #[serde(rename = "exports-only")]
    ExportsOnly,
}

/// NOTE: These configurations are printed as kebab-case names. Please pay attention when using.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlotlyOutputType {
    /// After the code is executed, it is compiled into an `<div>` for display.
    #[default]
    #[cfg(feature = "plotly-html-handler")]
    #[serde(rename = "plotly-html")]
    PlotlyHtml,

    /// After the code is executed, it is compiled into an SVG for display.
    #[cfg(feature = "plotly-svg-handler")]
    #[serde(rename = "plotly-svg")]
    PlotlySvg,
}

/// NOTE: These configurations are printed as kebab-case names. Please pay attention when using.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlotlyInputType {
    /// Translates the Json format into an actual plotly object.
    /// NOTE: In the `PlotlyOutputType = PlotlySvg` state, this method may cause some performance loss due to multiple packaging.
    #[default]
    #[serde(rename = "json-input")]
    JSONInput,

    /// Translates the TOML format into JSON value first, then reuses the existing plot parser.
    #[serde(rename = "toml-input")]
    TOMLInput,
}
