use crate::translate;
use anyhow::{Result, anyhow};
use plotly::{
    Configuration, Layout, Plot, Trace,
    common::color::Rgb,
    layout::{Legend, Margin},
};
use serde_json::Value;

pub fn parse(plot_obj: &mut Value) -> Result<Plot> {
    let mut plot = Plot::new();

    if let Some(layout_obj) = plot_obj.get_mut("layout")
        && layout_obj.is_object()
    {
        let layout = parse_layout_obj(layout_obj)?;
        plot.set_layout(layout);
    }

    if let Some(data_list) = plot_obj.get_mut("data")
        && data_list.is_array()
    {
        // Safety: This `unwrap` will never be reached.
        for data in data_list.as_array_mut().unwrap() {
            let trace = parse_data_obj(data)?;
            plot.add_trace(trace);
        }
    }

    if let Some(config_obj) = plot_obj.get_mut("config")
        && config_obj.is_object()
    {
        let config = parse_config_obj(config_obj)?;
        plot.set_configuration(config);
    }

    Ok(plot)
}

fn parse_config_obj(config_obj: &mut Value) -> Result<Configuration> {
    let config = translate! {
        Configuration::new(),
        config_obj,
        (static_plot, bool),
        (typeset_math, bool),
        (editable, bool),
        (autosizable, bool),
        // NOTE:
        // Although this method is still in place, it is no longer valid as far as the documentation is concerned.
        // Subsequent versions may remove this method without warning.
        (responsive, bool),
        (fill_frame, bool),
        (frame_margins, f64),
        (scroll_zoom, bool),
        (show_axis_drag_handles, bool),
        (show_axis_range_entry_boxes, bool),
        (show_tips, bool),
        (show_link, bool),
        (send_data, bool),
    }?;

    Ok(config)
}

fn parse_layout_obj(layout_obj: &mut Value) -> Result<Layout> {
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

    let layout = if let Some(legend_obj) = layout_obj.get_mut("legend")
        && legend_obj.is_object()
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

    let layout = if let Some(margin_obj) = layout_obj.get_mut("margin")
        && margin_obj.is_object()
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

pub fn parse_data_obj(data_obj: &mut Value) -> Result<Box<dyn Trace>> {
    let data_type = data_obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("`type` must be a string"))?;
    match data_type {
        "bar" => super::bar_parser::parse_bar_data(data_obj).map(|v| v as Box<dyn Trace>),
        "pie" => super::pie_parser::parse_pie_data(data_obj).map(|v| v as Box<dyn Trace>),
        unexpected => Err(anyhow!("{} isn't a type", unexpected)),
    }
}
