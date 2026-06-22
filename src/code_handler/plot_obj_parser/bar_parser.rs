use super::common::parse_marker;
use super::until::must_translate_from_context;
use crate::code_handler::parse_context::ParseContext;
use crate::{translate_enum_with_config, translate_with_config};
use anyhow::Result;
use plotly::{Bar, Trace};

pub fn parse_bar_data(
    bar_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<Bar<f64, f64>>> {
    let x: Vec<f64> = must_translate_from_context(bar_obj, context, "x")?;
    let y: Vec<f64> = must_translate_from_context(bar_obj, context, "y")?;
    let bar = Bar::new(x, y);
    let bar = translate_with_config! {
        bar,
        bar_obj,
        context.map(),
        context.map_eval(),
        (ids, Vec<String>),
        (offset, f64),
        (offset_array, Vec<f64>),
        (text, String),
        (text_array, Vec<String>),
        (text_template, String),
        (hover_template, String),
        (hover_template_array, Vec<String>),
        (hover_text, String),
        (hover_text_array, Vec<String>),
        (name, String),
        (opacity, f64),
        (x_axis, String),
        (y_axis, String),
        (alignment_group, String),
        (offset_group, String),
        (clip_on_axis, bool),
        (show_legend, bool),
        (legend_group, String),
        (width, f64),
        (text_angle, f64),
    }?;

    use plotly::common::Orientation;
    let bar = translate_enum_with_config! {
        bar,
        bar_obj,
        context.map(),
        context.map_eval(),
        (orientation, {
            "v" => Orientation::Vertical,
            "h" => Orientation::Horizontal,
        }),
    }?;

    let bar = if let Some(marker_obj) = bar_obj.get_mut("marker")
        && marker_obj.is_object()
    {
        let marker = parse_marker(marker_obj, context)?;
        bar.marker(marker)
    } else {
        bar
    };

    Ok(bar)
}

pub fn parse_bar_trace(
    bar_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<dyn Trace>> {
    Ok(parse_bar_data(bar_obj, context)?)
}
