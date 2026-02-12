use crate::preprocessor::config::PlotlyInputType;
use log::{debug, warn};
use plotly::{
    Layout, Plot, Trace,
    common::color::Rgba,
    layout::{Legend, Margin},
};
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
    ($target:expr, $value:expr, $(($method:ident, $ty:ty)),* $(,)?) => {{
        let target = $target;
        $(
            let target = if let Some(v) = $value.get(stringify!($method)) {
                let data = serde_json::from_value::<$ty>(v.clone())?;
                target.$method(data)
            } else {
                target
            };
        )*
        Ok::<_, serde_json::Error>(target)
    }};


    ($target:expr, $value:expr, $(($method:ident, $function:expr)),* $(,)?) => {{
        let target = $target;
        $(
            let target = if let Some(v) = $value.get(stringify!($method)) {
                let data = $function()?;
                target.$method(data)
            } else {
                target
            };
        )*
        Ok(target)
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
        let layout = handle_layout_obj(layout_obj)?;
        plot.set_layout(layout);
    }

    if let Some(data_list) = value.get("data")
        && data_list.is_array()
    {
        // Safety: This `unwrap` will never be reached.
        for data in data_list.as_array().unwrap() {
            let trace = handle_data_obj(data)?;
            plot.add_trace(trace);
        }
    }

    Ok(plot)
}

fn handle_layout_obj(layout_obj: &Value) -> Result<Layout, Box<dyn Error>> {
    let layout = translate! {
        Layout::new(),
        layout_obj,
        (title, String),
        (show_legend, bool),
        (height, usize),
        (width, usize),
    }?;

    let layout = if let Some(legend_obj) = layout_obj.get("legend")
        && layout_obj.is_object()
    {
        let legend = translate! {
            Legend::new(),
            legend_obj,
            (background_color, Rgba),
            (border_color, Rgba),
            (border_width, usize),
            (x, f64),
            (y, f64),
            (trace_group_gap, usize),
            (title, String),
        }?;
        layout.legend(legend)
    } else {
        layout
    };

    let layout = if let Some(margin_obj) = layout_obj.get("margin")
        && layout_obj.is_object()
    {
        let margin = translate! {
            Margin::new(),
            margin_obj,
            (left, usize),
            (right, usize),
            (top, usize),
            (bottom, usize),
            (pad, usize),
            (auto_expand, bool)
        }?;
        layout.margin(margin)
    } else {
        layout
    };

    Ok(layout)
}

fn handle_data_obj(data_obj: &Value) -> Result<Box<dyn Trace>, Box<dyn Error>> {
    let data_type = data_obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else::<String, _>(|| "`type` must be a string".into())?;
    match data_type {
        "pie" => handle_pie_data(data_obj).map(|v| v as Box<dyn Trace>),
        unexpected => Err(format!("{unexpected} isn't a type").into()),
    }
}

use plotly::Pie;

fn handle_pie_data(pie_obj: &Value) -> Result<Box<Pie<u64>>, Box<dyn Error>> {
    let pie = pie_obj
        .get("values")
        .ok_or(String::from("missing `values` field"))?;
    let pie = Pie::new(serde_json::from_value::<Vec<u64>>(pie.clone())?);
    Ok(pie)
}
