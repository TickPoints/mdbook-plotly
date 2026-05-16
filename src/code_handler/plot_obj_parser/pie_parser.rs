use super::until::{Color, Map, must_translate};
use crate::{translate, translate_enum};
use anyhow::Result;
use plotly::Pie;

pub fn parse_pie_data(pie_obj: &mut serde_json::Value, map: &Map) -> Result<Box<Pie<f64>>> {
    let pie: Vec<f64> = must_translate(pie_obj, map, "values")?;
    let pie: Box<Pie<f64>> = Pie::new(pie);
    let pie = translate! {
        pie,
        pie_obj,
        map,
        (automargin, bool),
        (dlabel, f64),
        (hole, f64),
        (hover_template, String),
        (hover_template_array, Vec<String>),
        (hover_text, String),
        (hover_text_array, Vec<String>),
        (ids, Vec<String>),
        (label0, f64),
        (labels, Vec<String>),
        (legend_group, String),
        (legend_rank, usize),
        (name, String),
        (opacity, f64),
        (meta, String),
        (sort, bool),
        (text_position_src, String),
        (text_position_src_array, Vec<String>),
        (text, String),
        (text_array, Vec<String>),
        (text_info, String),
        (show_legend, bool),
        (rotation, f64),
        (pull, f64),
    }?;

    use plotly::traces::pie::PieDirection;
    let pie = translate_enum! {
        pie,
        pie_obj,
        map,
        (direction, {
            "clockwise" =>          PieDirection::Clockwise,
            "counterclockwise" =>   PieDirection::CounterClockwise,
        }),
    }?;

    let pie = if let Some(marker_obj) = pie_obj.get_mut("marker")
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
        pie.marker(marker)
    } else {
        pie
    };

    Ok(pie)
}
