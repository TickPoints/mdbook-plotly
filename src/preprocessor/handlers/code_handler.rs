mod bar_parser;
mod pie_parser;
mod plot_obj_parser;
mod until;

use crate::preprocessor::config::PlotlyInputType;
use anyhow::Result;
use log::{debug, warn};
use plotly::Plot;
use serde_json::Value;

pub fn handle(raw_code: String, input_type: &PlotlyInputType) -> Result<Plot> {
    let result = match input_type {
        PlotlyInputType::SandBoxScript => {
            warn!("The entry has been discarded. This config shouldn't be used.");
            debug!("This function returns an empty string.");
            // This treatment may not be good, but it is sufficient.
            Plot::new()
        }
        PlotlyInputType::JSONInput => handle_json_input(raw_code)?,
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
pub fn handle_json_input(raw_code: String) -> Result<Plot> {
    // Use Json5 to provide more flexible JSON.
    let mut value: Value = json5::from_str(&raw_code)?;
    plot_obj_parser::parse(&mut value)
}
