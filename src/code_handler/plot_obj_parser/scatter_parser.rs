use super::until::{Color, Map, must_translate};
use crate::{translate, translate_enum};
use anyhow::Result;
use plotly::Scatter;

pub fn parse_scatter_data(
    sc_obj: &mut serde_json::Value,
    map: &Map,
) -> Result<Box<Scatter<f64, f64>>> {
    let x: Vec<f64> = must_translate(sc_obj, map, "x")?;
    let y: Vec<f64> = must_translate(sc_obj, map, "y")?;
    let sc = Scatter::new(x, y);
    let sc = translate! {
        sc,
        sc_obj,
        map,
        (web_gl_mode, bool),
        (x0, f64),
        (dx, f64),
        (y0, f64),
        (dy, f64),
        (ids, Vec<String>),
        (text, String),
        (text_array, Vec<String>),
        (text_template, String),
        (hover_template, String),
        (hover_template_array, Vec<String>),
        (hover_text, String),
        (hover_text_array, Vec<String>),
        (name, String),
        (opacity, f64),
        (meta, String),
        (x_axis, String),
        (y_axis, String),
        (stack_group, String),
        (clip_on_axis, bool),
        (connect_gaps, bool),
        (fill_color, Color),
        (show_legend, bool),
        (legend_group, String),
    }?;

    use plotly::common::Fill;
    use plotly::common::Mode;
    let sc = translate_enum! {
        sc,
        sc_obj,
        map,
        (fill, {
            "tozeroy" => Fill::ToZeroY,
            "tozerox" => Fill::ToZeroX,
            "tonexty" => Fill::ToNextY,
            "tonextx" => Fill::ToNextX,
            "toself" =>  Fill::ToSelf,
            "tonext" =>  Fill::ToNext,
            "none" =>    Fill::None,
        }),
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

    let sc = if let Some(marker_obj) = sc_obj.get_mut("marker")
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
        sc.marker(marker)
    } else {
        sc
    };

    Ok(sc)
}
