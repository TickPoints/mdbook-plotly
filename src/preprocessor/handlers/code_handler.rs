use crate::preprocessor::config::PlotlyInputType;
use log::{debug, warn};
use plotly::{
    Configuration, Layout, Plot, Trace,
    common::color::Rgb,
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
    // Use Json5 to provide more flexible JSON.
    let value: Value = json5::from_str(&raw_code)?;

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

    if let Some(config_obj) = value.get("config")
        && config_obj.is_object()
    {
        let config = handle_config_obj(config_obj)?;
        plot.set_configuration(config);
    }

    Ok(plot)
}

fn handle_config_obj(config_obj: &Value) -> Result<Configuration, Box<dyn Error>> {
    let config = translate! {
        Configuration::new(),
        config_obj,
        (static_plot, bool),
        (typeset_math, bool),
    }?;

    Ok(config)
}

fn handle_layout_obj(layout_obj: &Value) -> Result<Layout, Box<dyn Error>> {
    let layout = translate! {
        Layout::new(),
        layout_obj,
        (title, String),
        (show_legend, bool),
        (height, usize),
        (width, usize),
        (colorway, Vec<Rgb>),
        (plot_background_color, Rgb),
        (separators, String),
    }?;

    let layout = if let Some(legend_obj) = layout_obj.get("legend")
        && layout_obj.is_object()
    {
        let legend = translate! {
            Legend::new(),
            legend_obj,
            (background_color, Rgb),
            (border_color, Rgb),
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
    let pie = translate! {
        pie,
        pie_obj,
        (automargin, bool),
        (dlabel, f64),
        (hole, f64),
        (hover_template, String),
        (hover_template_array, Vec<String>),
        (hover_text, String),
        (hover_text_array, Vec<String>),
        (ids, Vec<String>),
        (label0, f64),
        (labels, Vec<String>),
        (legend_group, String),
        (legend_rank, usize),
        (name, String),
        (opacity, f64),
        (meta, String),
        (sort, bool),
        (text_position_src, String),
        (text_position_src_array, Vec<String>),
        (text, String),
        (text_array, Vec<String>),
        (text_info, String),
    }?;
    Ok(pie)
}
