use super::common::parse_marker;
use super::until::must_translate_from_context;
use crate::code_handler::parse_context::ParseContext;
use crate::{translate_enum_with_config, translate_with_config};
use anyhow::Result;
use plotly::{ScatterGeo, Trace};

pub fn parse_scatter_geo_data(
    sg_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<ScatterGeo<f64, f64>>> {
    let lat: Vec<f64> = must_translate_from_context(sg_obj, context, "lat")?;
    let lon: Vec<f64> = must_translate_from_context(sg_obj, context, "lon")?;
    let sg = ScatterGeo::new(lat, lon);
    let sg = translate_with_config! {
        sg,
        sg_obj,
        context.map(),
        context.map_eval(),
        (ids, Vec<String>),
        (show_legend, bool),
        (name, String),
        (legend_group, String),
        (legend_rank, usize),
        (opacity, f64),
        (text, String),
        (text_array, Vec<String>),
        (text_template, String),
        (text_template_array, Vec<String>),
        (hover_text, String),
        (hover_text_array, Vec<String>),
        (hover_template, String),
        (hover_template_array, Vec<String>),
        (connect_gaps, bool),
        (subplot, String),
        (below, String),
    }?;

    use plotly::common::Mode;
    let sg = translate_enum_with_config! {
        sg,
        sg_obj,
        context.map(),
        context.map_eval(),
        (mode, {
            "lines" =>          Mode::Lines,
            "markers" =>        Mode::Markers,
            "text" =>           Mode::Text,
            "linesmarkers" =>   Mode::LinesMarkers,
            "linestext" =>      Mode::LinesText,
            "markerstext" =>    Mode::MarkersText,
            "linemarkerstext" =>Mode::LinesMarkersText,
            "none" =>           Mode::None,
        }),
    }?;

    let sg = if let Some(marker_obj) = sg_obj.get_mut("marker")
        && marker_obj.is_object()
    {
        let marker = parse_marker(marker_obj, context)?;
        sg.marker(marker)
    } else {
        sg
    };

    Ok(sg)
}

pub fn parse_scatter_geo_trace(
    sg_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<dyn Trace>> {
    Ok(parse_scatter_geo_data(sg_obj, context)?)
}
