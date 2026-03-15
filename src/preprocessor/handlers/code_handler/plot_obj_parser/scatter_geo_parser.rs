use super::until::{Map, must_translate};
use crate::{translate, translate_enum};
use anyhow::Result;
use plotly::ScatterGeo;

pub fn parse_scatter_geo_data(
    sg_obj: &mut serde_json::Value,
    map: &Map,
) -> Result<Box<ScatterGeo<f64, f64>>> {
    let lat: Vec<f64> = must_translate(sg_obj, map, "lat")?;
    let lon: Vec<f64> = must_translate(sg_obj, map, "lon")?;
    let sg = ScatterGeo::new(lat, lon);
    let sg = translate! {
        sg,
        sg_obj,
        map,
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
    let sg = translate_enum! {
        sg,
        sg_obj,
        map,
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

    Ok(sg)
}
