use serde::{Deserialize, Serialize};

pub const SUPPORTED_MDBOOK_VERSION: &str = "0.5.2";

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

    /// About the script source control.
    /// If this is false(default), a JS script source from CDN will be injected;
    /// otherwise, an HTML script tag containing an embedded JS source will be added for offline use.
    pub offline_js_sources: bool,
}

/// NOTE: These configurations are printed as kebab-case names. Please pay attention when using.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PlotlyOutputType {
    /// After the code is executed, it is compiled into an `<div>` for display.
    #[default]
    #[cfg(feature = "plotly-html-handler")]
    PlotlyHtml,

    /// After the code is executed, it is compiled into an SVG for display.
    #[cfg(feature = "plotly-svg-handler")]
    PlotlySvg,
}

/// NOTE: These configurations are printed as kebab-case names. Please pay attention when using.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PlotlyInputType {
    /// Translates the Json format into an actual plotly object.
    /// NOTE: In the `PlotlyOutputType = PlotlySvg` state, this method may cause some performance loss due to multiple packaging.
    #[default]
    JSONInput,

    /// NOTE: This entry is deprecated because the use of `rquickjs` was abandoned.
    /// Execute the script locally in a sandbox, either for preprocessing or to complete the compilation directly.
    /// Once processed, follow the target output type.
    SandBoxScript,
}
