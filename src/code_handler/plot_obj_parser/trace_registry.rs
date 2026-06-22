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

type TraceParser = fn(&mut Value, &ParseContext<'_>) -> Result<Box<dyn Trace>>;

fn parse_box(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    box_plot_parser::parse_box_plot_data(data_obj, context)
}

fn parse_candlestick(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    candlestick_parser::parse_candlestick_data(data_obj, context)
}

fn parse_density_mapbox(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    density_mapbox_parser::parse_density_mapbox_trace(data_obj, context)
}

fn parse_histogram(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    histogram_parser::parse_histogram_trace(data_obj, context)
}

fn parse_ohlc(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    ohlc_parser::parse_ohlc_trace(data_obj, context)
}

fn parse_image(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    image_parser::parse_image_trace(data_obj, context)
}

fn parse_mesh3d(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    mesh3d_parser::parse_mesh3d_trace(data_obj, context)
}

fn parse_pie(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    pie_parser::parse_pie_trace(data_obj, context)
}

fn parse_sankey(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    sankey_parser::parse_sankey_trace(data_obj, context)
}

fn parse_scatter3d(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    scatter3d_parser::parse_scatter3d_trace(data_obj, context)
}

fn parse_scatter_geo(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    scatter_geo_parser::parse_scatter_geo_trace(data_obj, context)
}

fn parse_scatter_mapbox(
    data_obj: &mut Value,
    context: &ParseContext<'_>,
) -> Result<Box<dyn Trace>> {
    scatter_mapbox_parser::parse_scatter_mapbox_trace(data_obj, context)
}

fn parse_scatter_polar(
    data_obj: &mut Value,
    context: &ParseContext<'_>,
) -> Result<Box<dyn Trace>> {
    scatter_polar_parser::parse_scatter_polar_trace(data_obj, context)
}

fn parse_table(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    table_parser::parse_table_data(data_obj, context)
}

fn parse_bar(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    bar_parser::parse_bar_trace(data_obj, context)
}

fn parse_contour(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    contour_parser::parse_contour_trace(data_obj, context)
}

fn parse_heatmap(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    heat_map_parser::parse_heat_map_trace(data_obj, context)
}

fn parse_scatter(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    scatter_parser::parse_scatter_trace(data_obj, context)
}

fn parse_surface(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    surface_parser::parse_surface_trace(data_obj, context)
}

const TRACE_PARSERS: &[(&str, TraceParser)] = &[
    ("bar", parse_bar),
    ("box", parse_box),
    ("candlestick", parse_candlestick),
    ("contour", parse_contour),
    ("density_mapbox", parse_density_mapbox),
    ("heatmap", parse_heatmap),
    ("histogram", parse_histogram),
    ("ohlc", parse_ohlc),
    ("image", parse_image),
    ("mesh3d", parse_mesh3d),
    ("pie", parse_pie),
    ("sankey", parse_sankey),
    ("scatter", parse_scatter),
    ("scatter3d", parse_scatter3d),
    ("scatter_geo", parse_scatter_geo),
    ("scatter_mapbox", parse_scatter_mapbox),
    ("scatter_polar", parse_scatter_polar),
    ("surface", parse_surface),
    ("table", parse_table),
];

pub fn parse_data_obj(data_obj: &mut Value, context: &ParseContext<'_>) -> Result<Box<dyn Trace>> {
    let data_type = data_obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("`type` must be a string"))?
        .to_owned();

    let Some((_, parser)) = TRACE_PARSERS
        .iter()
        .find(|(trace_type, _)| *trace_type == data_type.as_str())
    else {
        return Err(anyhow!("{} isn't a type in data", data_type));
    };

    parser(data_obj, context)
}
