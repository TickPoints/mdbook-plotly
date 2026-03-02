use super::until::{Map, must_translate};
use crate::translate;
use anyhow::{Result, anyhow};
use plotly::ScatterMapbox;

pub fn parse_scatter_mapbox_data(
    sm_obj: &mut serde_json::Value,
    map: &Map,
) -> Result<Box<ScatterMapbox<f64, f64>>> {
    let lat: Vec<f64> = must_translate(sm_obj, "lat")?;
    let lon: Vec<f64> = must_translate(sm_obj, "lon")?;
    let sm = ScatterMapbox::new(lat, lon);
    let sm = translate! {
        sm,
        sm_obj,
        map,
        (ids, Vec<String>),
        (selected_points, Vec<usize>),
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
        (subplot, String),
        (below, String),
        (meta, String),
    }?;
    let sm = if let Some(mode) = sm_obj.get_mut("mode")
        && mode.is_string()
    {
        use plotly::common::Mode;
        let mode = match mode.as_str().unwrap_or_else(|| unreachable!()) {
            "lines" => Mode::Lines,
            "markers" => Mode::Markers,
            "text" => Mode::Text,
            "linesmarkers" => Mode::LinesMarkers,
            "linestext" => Mode::LinesText,
            "markerstext" => Mode::MarkersText,
            "linemarkerstext" => Mode::LinesMarkersText,
            "none" => Mode::None,
            unexpected => return Err(anyhow!("{unexpected} can't be mode")),
        };
        sm.mode(mode)
    } else {
        sm
    };
    Ok(sm)
}
