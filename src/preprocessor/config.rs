use serde::{Deserialize, Serialize};

pub const SUPPORTED_MDBOOK_VERSION: &str = "0.5.2";

/// NOTE: These configurations are printed as kebab-case names. Please pay attention when using.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
}

/// NOTE: These configurations are printed as kebab-case names. Please pay attention when using.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlotlyOutputType {
    /// After the code is executed, it is compiled into an SVG for display.
    #[default]
    PlotlySvg,
}

/// NOTE: These configurations are printed as kebab-case names. Please pay attention when using.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlotlyInputType {
    /// Execute the script locally in a sandbox, either for preprocessing or to complete the compilation directly.
    /// Once processed, follow the target output type.
    #[default]
    SandBoxScript,
}
