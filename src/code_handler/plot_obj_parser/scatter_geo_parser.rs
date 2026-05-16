use super::until::{Color, Map, must_translate};
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

    let sg = if let Some(marker_obj) = sg_obj.get_mut("marker")
        && marker_obj.is_object()
    {
        let marker = plotly::common::Marker::new();
        let marker = translate! {
            marker,
            marker_obj,
            map,
            (color, Color),
            (opacity, f64),
            (size, usize),
            (size_array, Vec<usize>),
            (max_displayed, usize),
            (size_ref, usize),
            (size_min, usize),
            (cauto, bool),
            (cmax, f64),
            (cmin, f64),
            (cmid, f64),
            (auto_color_scale, bool),
            (reverse_scale, bool),
            (show_scale, bool),
            (outlier_color, Color)
        }?;

        use plotly::common::{MarkerSymbol, SizeMode};
        let marker = translate_enum! {
            marker,
            marker_obj,
            map,
            (symbol, {
                "circle" =>         MarkerSymbol::Circle,
                "square" =>         MarkerSymbol::Square,
                "diamond" =>        MarkerSymbol::Diamond,
                "cross" =>          MarkerSymbol::Cross,
                "x" =>              MarkerSymbol::X,
                "triangle-up" =>    MarkerSymbol::TriangleUp,
                "triangle-down" =>  MarkerSymbol::TriangleDown,
                "triangle-left" =>  MarkerSymbol::TriangleLeft,
                "triangle-right" => MarkerSymbol::TriangleRight,
                "pentagon" =>       MarkerSymbol::Pentagon,
                "hexagon" =>        MarkerSymbol::Hexagon,
            }),
            (size_mode, {
                "area" => SizeMode::Area,
                "diameter" => SizeMode::Diameter,
            }),
        }?;
        sg.marker(marker)
    } else {
        sg
    };

    Ok(sg)
}
