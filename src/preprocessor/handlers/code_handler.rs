use crate::preprocessor::config::PlotlyInputType;
use log::{debug, warn};
use plotly::{Layout, Plot, common::color::Rgba, layout::Legend};
use serde_json::Value;
use std::error::Error;

pub fn handle(raw_code: String, input_type: &PlotlyInputType) -> Result<Plot, Box<dyn Error>> {
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

macro_rules! translate {
    ($target:expr, $value:expr, $(($json_key:literal, $method:ident, $ty:ty)),* $(,)?) => {{
        let target = $target;
        $(
            let target = if let Some(v) = $value.get($json_key) {
                let data = serde_json::from_value::<$ty>(v.clone())?;
                target.$method(data)
            } else {
                target
            };
        )*
        Ok::<_, serde_json::Error>(target)
    }};
}

/// `Plot` does not implement `Deserialize`, so this routine is only an
/// unofficial best-effort translation.
///
/// Do not be surprised if the output of `Plot::serialize` cannot be
/// round-tripped through this function.
///
/// In addition, fields that cannot be translated are silently dropped.
pub fn handle_json_input(raw_code: String) -> Result<Plot, Box<dyn Error>> {
    let mut plot = Plot::new();
    let value: Value = serde_json::from_str(&raw_code)?;

    if let Some(layout_obj) = value.get("layout")
        && layout_obj.is_object()
    {
        let layout = translate! {
            Layout::new(),
            layout_obj,
            ("title", title, String),
            ("show_legend", show_legend, bool)
        }?;

        let layout = if let Some(legend_obj) = layout_obj.get("legend")
            && layout_obj.is_object()
        {
            let legend = translate! {
                Legend::new(),
                legend_obj,
                ("background_color", background_color, Rgba),
                ("border_color", border_color, Rgba),
                ("border_width", border_width, usize),
                ("x", x, f64),
                ("y", y, f64),
                ("trace_group_gap", trace_group_gap, usize),
                ("title", title, String),
            }?;
            layout.legend(legend)
        } else {
            layout
        };

        plot.set_layout(layout);
    }

    Ok(plot)
}
