use super::until::Color;
use crate::code_handler::parse_context::ParseContext;
use crate::{translate_enum_with_config, translate_with_config};
use anyhow::Result;
use plotly::common::{ColorBar, Marker};

pub fn parse_marker(
    marker_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Marker> {
    let marker = translate_with_config! {
        Marker::new(),
        marker_obj,
        context.map(),
        context.map_eval(),
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
    translate_enum_with_config! {
        marker,
        marker_obj,
        context.map(),
        context.map_eval(),
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
    }
}

pub fn parse_color_bar(
    color_bar_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<ColorBar> {
    translate_with_config! {
        ColorBar::new(),
        color_bar_obj,
        context.map(),
        context.map_eval(),
        (thickness, usize),
        (len, usize),
        (x, f64),
        (y, f64),
        (title, String),
    }
}
