pub use super::until;
use super::until::{Color, Map};
use crate::translate;
use anyhow::{Result, anyhow};
use plotly::{
    Configuration, Layout, Plot, Trace,
    layout::{Legend, Margin},
};
use serde_json::Value;

pub mod bar_parser;
pub mod candlestick_parser;
pub mod density_mapbox_parser;
pub mod histogram_parser;
pub mod image_parser;
pub mod ohlc_parser;
pub mod pie_parser;
pub mod sankey_parser;
pub mod scatter_geo_parser;
pub mod scatter_mapbox_parser;
pub mod scatter_parser;
pub mod scatter_polar_parser;
pub mod table_parser;

pub fn parse(plot_obj: &mut Value) -> Result<Plot> {
    let mut plot = Plot::new();

    let map = if let Some(map_obj) = plot_obj.get_mut("map") {
        serde_json::from_value::<Map>(map_obj.take())?
    } else {
        Map::new()
    };

    if let Some(config_obj) = plot_obj.get_mut("config")
        && config_obj.is_object()
    {
        let config = parse_config_obj(config_obj, &map)?;
        plot.set_configuration(config);
    }

    if let Some(layout_obj) = plot_obj.get_mut("layout")
        && layout_obj.is_object()
    {
        let layout = parse_layout_obj(layout_obj, &map)?;
        plot.set_layout(layout);
    }

    if let Some(data_list) = plot_obj.get_mut("data")
        && data_list.is_array()
    {
        for data in data_list.as_array_mut().unwrap_or_else(|| unreachable!()) {
            let trace = parse_data_obj(data, &map)?;
            plot.add_trace(trace);
        }
    }

    Ok(plot)
}

fn parse_config_obj(config_obj: &mut Value, map: &Map) -> Result<Configuration> {
    let config = translate! {
        Configuration::new(),
        config_obj,
        map,
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
        (double_click_delay, usize),
        (queue_length, usize),
        (display_logo, bool),
        (watermark, bool),
    }?;

    Ok(config)
}

fn parse_layout_obj(layout_obj: &mut Value, map: &Map) -> Result<Layout> {
    let layout = translate! {
        Layout::new(),
        layout_obj,
        map,
        (title, String),
        (show_legend, bool),
        (height, usize),
        (width, usize),
        (colorway, Vec<Color>),
        (plot_background_color, Color),
        (separators, String),
    }?;

    let layout = if let Some(legend_obj) = layout_obj.get_mut("legend")
        && legend_obj.is_object()
    {
        let legend = translate! {
            Legend::new(),
            legend_obj,
            map,
            (background_color, Color),
            (border_color, Color),
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
            map,
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

pub fn parse_data_obj(data_obj: &mut Value, map: &Map) -> Result<Box<dyn Trace>> {
    let data_type = data_obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("`type` must be a string"))?;
    match data_type {
        "bar" => bar_parser::parse_bar_data(data_obj, map).map(|v| v as Box<dyn Trace>),
        "candlestick" => {
            candlestick_parser::parse_candlestick_data(data_obj, map).map(|v| v as Box<dyn Trace>)
        }
        "density_mapbox" => density_mapbox_parser::parse_density_mapbox_data(data_obj, map)
            .map(|v| v as Box<dyn Trace>),
        "histogram" => {
            histogram_parser::parse_histogram_data(data_obj, map).map(|v| v as Box<dyn Trace>)
        }
        "ohlc" => ohlc_parser::parse_ohlc_data(data_obj, map).map(|v| v as Box<dyn Trace>),
        "image" => image_parser::parse_image_data(data_obj, map).map(|v| v as Box<dyn Trace>),
        "pie" => pie_parser::parse_pie_data(data_obj, map).map(|v| v as Box<dyn Trace>),
        "sankey" => sankey_parser::parse_sankey_data(data_obj, map).map(|v| v as Box<dyn Trace>),
        "scatter" => scatter_parser::parse_scatter_data(data_obj, map).map(|v| v as Box<dyn Trace>),
        "scatter_geo" => {
            scatter_geo_parser::parse_scatter_geo_data(data_obj, map).map(|v| v as Box<dyn Trace>)
        }
        "scatter_mapbox" => scatter_mapbox_parser::parse_scatter_mapbox_data(data_obj, map)
            .map(|v| v as Box<dyn Trace>),
        "scatter_polar" => scatter_polar_parser::parse_scatter_polar_data(data_obj, map)
            .map(|v| v as Box<dyn Trace>),
        "table" => table_parser::parse_table_data(data_obj, map).map(|v| v as Box<dyn Trace>),
        unexpected => Err(anyhow!("{} isn't a type", unexpected)),
    }
}
