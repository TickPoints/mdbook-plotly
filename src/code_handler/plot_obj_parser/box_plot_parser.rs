use super::until::{Color, Map, must_translate};
use crate::{translate, translate_enum};
use anyhow::Result;
use plotly::BoxPlot;

pub fn parse_box_plot_data(
    box_obj: &mut serde_json::Value,
    map: &Map,
) -> Result<Box<BoxPlot<f64, f64>>> {
    let box_plot = if box_obj.get("y").is_some() {
        let y: Vec<f64> = must_translate(box_obj, map, "y")?;
        if box_obj.get("x").is_some() {
            let x: Vec<f64> = must_translate(box_obj, map, "x")?;
            BoxPlot::new_xy(x, y)
        } else {
            BoxPlot::new(y)
        }
    } else {
        let y: Vec<f64> = Vec::new();
        BoxPlot::new(y)
    };

    let box_plot = translate! {
        box_plot,
        box_obj,
        map,
        (name, String),
        (opacity, f64),
        (ids, Vec<String>),
        (text, String),
        (text_array, Vec<String>),
        (hover_text, String),
        (hover_text_array, Vec<String>),
        (hover_template, String),
        (hover_template_array, Vec<String>),
        (x_axis, String),
        (y_axis, String),
        (alignment_group, String),
        (offset_group, String),
        (show_legend, bool),
        (legend_group, String),
        (fill_color, Color),
        (notched, bool),
        (notch_width, f64),
        (whisker_width, f64),
        (q1, Vec<f64>),
        (median, Vec<f64>),
        (q3, Vec<f64>),
        (upper_fence, Vec<f64>),
        (lower_fence, Vec<f64>),
        (notch_span, Vec<f64>),
        (mean, Vec<f64>),
        (standard_deviation, Vec<f64>),
        (point_pos, f64),
        (jitter, f64),
    }?;

    use plotly::common::Orientation;
    let box_plot = translate_enum! {
        box_plot,
        box_obj,
        map,
        (orientation, {
            "v" => Orientation::Vertical,
            "h" => Orientation::Horizontal,
        }),
    }?;

    use plotly::traces::box_plot::{BoxMean, BoxPoints, HoverOn, QuartileMethod};
    let box_plot = translate_enum! {
        box_plot,
        box_obj,
        map,
        (box_mean, {
            "true"    => BoxMean::True,
            "false"   => BoxMean::False,
            "sd"      => BoxMean::StandardDeviation,
        }),
        (box_points, {
            "all"           => BoxPoints::All,
            "outliers"      => BoxPoints::Outliers,
            "suspectedoutliers" => BoxPoints::SuspectedOutliers,
            "false"         => BoxPoints::False,
        }),
        (quartile_method, {
            "linear"        => QuartileMethod::Linear,
            "exclusive"     => QuartileMethod::Exclusive,
            "inclusive"     => QuartileMethod::Inclusive,
        }),
        (hover_on, {
            "points"  => HoverOn::Points,
            "boxes"   => HoverOn::Boxes,
            "boxes+points" => HoverOn::BoxesAndPoints,
        }),
    }?;

    let box_plot = if let Some(marker_obj) = box_obj.get_mut("marker")
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
            (outlier_color, Color),
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
        box_plot.marker(marker)
    } else {
        box_plot
    };

    let box_plot = if let Some(line_obj) = box_obj.get_mut("line")
        && line_obj.is_object()
    {
        use plotly::common::Line;
        let line = translate! {
            Line::new(),
            line_obj,
            map,
            (color, Color),
            (width, f64),
        }?;
        use plotly::common::DashType;
        let line = translate_enum! {
            line,
            line_obj,
            map,
            (dash, {
                "solid" => DashType::Solid,
                "dot" => DashType::Dot,
                "dash" => DashType::Dash,
                "longdash" => DashType::LongDash,
                "dashdot" => DashType::DashDot,
                "longdashdot" => DashType::LongDashDot,
            }),
        }?;
        box_plot.line(line)
    } else {
        box_plot
    };

    Ok(box_plot)
}
