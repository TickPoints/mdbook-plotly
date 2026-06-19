use super::parse_context::ParseContext;
pub use super::until;
use super::until::Map;
use crate::preprocessor::config::MapEvalConfig;
use anyhow::Result;
use plotly::Plot;
use serde_json::Value;

pub mod bar_parser;
pub mod box_plot_parser;
pub mod candlestick_parser;
pub mod common;
pub mod contour_parser;
pub mod density_mapbox_parser;
pub mod heat_map_parser;
pub mod histogram_parser;
pub mod image_parser;
pub mod layout_parser;
pub mod mesh3d_parser;
pub mod ohlc_parser;
pub mod pie_parser;
pub mod sankey_parser;
pub mod scatter3d_parser;
pub mod scatter_geo_parser;
pub mod scatter_mapbox_parser;
pub mod scatter_parser;
pub mod scatter_polar_parser;
pub mod surface_parser;
pub mod table_parser;
pub mod trace_registry;

use layout_parser::{parse_config_obj, parse_layout_obj};
use trace_registry::parse_data_obj;

pub fn parse(plot_obj: &mut Value, map_eval: &MapEvalConfig) -> Result<Plot> {
    let mut plot = Plot::new();

    let map = if let Some(map_obj) = plot_obj.get_mut("map") {
        serde_json::from_value::<Map>(map_obj.take())?
    } else {
        Map::new()
    };

    let context = ParseContext::new(&map, map_eval);

    if let Some(config_obj) = plot_obj.get_mut("config")
        && config_obj.is_object()
    {
        let config = parse_config_obj(config_obj, &context)?;
        plot.set_configuration(config);
    }

    if let Some(layout_obj) = plot_obj.get_mut("layout")
        && layout_obj.is_object()
    {
        let layout = parse_layout_obj(layout_obj, &context)?;
        plot.set_layout(layout);
    }

    if let Some(data_list) = plot_obj.get_mut("data")
        && data_list.is_array()
    {
        for data in data_list.as_array_mut().unwrap_or_else(|| unreachable!()) {
            let trace = parse_data_obj(data, &context)?;
            plot.add_trace(trace);
        }
    }

    Ok(plot)
}
