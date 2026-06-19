use super::{
    bar_parser, box_plot_parser, candlestick_parser, contour_parser, density_mapbox_parser,
    heat_map_parser, histogram_parser, image_parser, mesh3d_parser, ohlc_parser, pie_parser,
    sankey_parser, scatter_geo_parser, scatter_mapbox_parser, scatter_parser, scatter_polar_parser,
    scatter3d_parser, surface_parser, table_parser,
};
use crate::code_handler::parse_context::ParseContext;
use anyhow::{Result, anyhow};
use plotly::Trace;
use serde_json::Value;

pub fn parse_data_obj(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    let data_type = data_obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("`type` must be a string"))?;

    match data_type {
        "bar" => bar_parser::parse_bar_data(data_obj, context).map(|v| v as Box<dyn Trace>),
        "box" => box_plot_parser::parse_box_plot_data(data_obj, context.map())
            .map(|v| v as Box<dyn Trace>),
        "candlestick" => candlestick_parser::parse_candlestick_data(data_obj, context.map())
            .map(|v| v as Box<dyn Trace>),
        "contour" => {
            contour_parser::parse_contour_data(data_obj, context.map()).map(|v| v as Box<dyn Trace>)
        }
        "density_mapbox" => {
            density_mapbox_parser::parse_density_mapbox_data(data_obj, context.map())
                .map(|v| v as Box<dyn Trace>)
        }
        "heatmap" => heat_map_parser::parse_heat_map_data(data_obj, context.map())
            .map(|v| v as Box<dyn Trace>),
        "histogram" => histogram_parser::parse_histogram_data(data_obj, context.map())
            .map(|v| v as Box<dyn Trace>),
        "ohlc" => {
            ohlc_parser::parse_ohlc_data(data_obj, context.map()).map(|v| v as Box<dyn Trace>)
        }
        "image" => {
            image_parser::parse_image_data(data_obj, context.map()).map(|v| v as Box<dyn Trace>)
        }
        "mesh3d" => {
            mesh3d_parser::parse_mesh3d_data(data_obj, context.map()).map(|v| v as Box<dyn Trace>)
        }
        "pie" => pie_parser::parse_pie_data(data_obj, context.map()).map(|v| v as Box<dyn Trace>),
        "sankey" => {
            sankey_parser::parse_sankey_data(data_obj, context.map()).map(|v| v as Box<dyn Trace>)
        }
        "scatter" => {
            scatter_parser::parse_scatter_data(data_obj, context).map(|v| v as Box<dyn Trace>)
        }
        "scatter3d" => scatter3d_parser::parse_scatter3d_data(data_obj, context.map())
            .map(|v| v as Box<dyn Trace>),
        "scatter_geo" => scatter_geo_parser::parse_scatter_geo_data(data_obj, context.map())
            .map(|v| v as Box<dyn Trace>),
        "scatter_mapbox" => {
            scatter_mapbox_parser::parse_scatter_mapbox_data(data_obj, context.map())
                .map(|v| v as Box<dyn Trace>)
        }
        "scatter_polar" => scatter_polar_parser::parse_scatter_polar_data(data_obj, context.map())
            .map(|v| v as Box<dyn Trace>),
        "surface" => {
            surface_parser::parse_surface_data(data_obj, context.map()).map(|v| v as Box<dyn Trace>)
        }
        "table" => {
            table_parser::parse_table_data(data_obj, context.map()).map(|v| v as Box<dyn Trace>)
        }
        unexpected => Err(anyhow!("{} isn't a type in data", unexpected)),
    }
}
