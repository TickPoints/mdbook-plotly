pub mod parse_context;
pub mod plot_obj_parser;
pub mod until;

use crate::preprocessor::config::{MapEvalConfig, PlotlyInputType};
use anyhow::Result;
use plotly::Plot;
use serde_json::Value;

pub fn handle(
    raw_code: String,
    input_type: &PlotlyInputType,
    map_eval: &MapEvalConfig,
) -> Result<Plot> {
    let result = match input_type {
        PlotlyInputType::JSONInput => handle_json_input(raw_code, map_eval)?,
    };
    Ok(result)
}

/// `Plot` does not implement `Deserialize`, so this routine is only an
/// unofficial best-effort translation.
///
/// Do not be surprised if the output of `Plot::serialize` cannot be
/// round-tripped through this function.
///
/// In addition, fields that cannot be translated are silently dropped.
pub fn handle_json_input(raw_code: String, map_eval: &MapEvalConfig) -> Result<Plot> {
    // Use Json5 to provide more flexible JSON.
    let mut value: Value = json5::from_str(&raw_code)?;
    plot_obj_parser::parse(&mut value, map_eval)
}
