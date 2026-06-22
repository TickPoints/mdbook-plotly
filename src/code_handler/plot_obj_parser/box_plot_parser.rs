use super::common::parse_marker;
use super::until::{Color, must_translate_from_context};
use crate::code_handler::parse_context::ParseContext;
use crate::{translate_enum_with_config, translate_with_config};
use anyhow::Result;
use plotly::BoxPlot;
use plotly::Trace;

pub fn parse_box_plot_data(
    box_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<dyn Trace>> {
    let box_plot = if box_obj.get("y").is_some() {
        let y: Vec<f64> = must_translate_from_context(box_obj, context, "y")?;
        if box_obj.get("x").is_some() {
            let x: Vec<f64> = must_translate_from_context(box_obj, context, "x")?;
            BoxPlot::new_xy(x, y)
        } else {
            BoxPlot::new(y)
        }
    } else {
        let y: Vec<f64> = Vec::new();
        BoxPlot::new(y)
    };

    let box_plot = translate_with_config! {
        box_plot,
        box_obj,
        context.map(),
        context.map_eval(),
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
    let box_plot = translate_enum_with_config! {
        box_plot,
        box_obj,
        context.map(),
        context.map_eval(),
        (orientation, {
            "v" => Orientation::Vertical,
            "h" => Orientation::Horizontal,
        }),
    }?;

    use plotly::traces::box_plot::{BoxMean, BoxPoints, HoverOn, QuartileMethod};
    let box_plot = translate_enum_with_config! {
        box_plot,
        box_obj,
        context.map(),
        context.map_eval(),
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
        let marker = parse_marker(marker_obj, context)?;
        box_plot.marker(marker)
    } else {
        box_plot
    };

    let box_plot = if let Some(line_obj) = box_obj.get_mut("line")
        && line_obj.is_object()
    {
        use plotly::common::Line;
        let line = translate_with_config! {
            Line::new(),
            line_obj,
            context.map(),
            context.map_eval(),
            (color, Color),
            (width, f64),
        }?;
        use plotly::common::DashType;
        let line = translate_enum_with_config! {
            line,
            line_obj,
            context.map(),
            context.map_eval(),
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
