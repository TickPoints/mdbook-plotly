use super::common::parse_marker;
use super::until::must_translate_from_context;
use crate::code_handler::parse_context::ParseContext;
use crate::{translate_enum_with_config, translate_with_config};
use anyhow::Result;
use plotly::{Pie, Trace};

pub fn parse_pie_data(
    pie_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<Pie<f64>>> {
    let pie: Vec<f64> = must_translate_from_context(pie_obj, context, "values")?;
    let pie: Box<Pie<f64>> = Pie::new(pie);
    let pie = translate_with_config! {
        pie,
        pie_obj,
        context.map(),
        context.map_eval(),
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
    let pie = translate_enum_with_config! {
        pie,
        pie_obj,
        context.map(),
        context.map_eval(),
        (direction, {
            "clockwise" =>          PieDirection::Clockwise,
            "counterclockwise" =>   PieDirection::CounterClockwise,
        }),
    }?;

    let pie = if let Some(marker_obj) = pie_obj.get_mut("marker")
        && marker_obj.is_object()
    {
        let marker = parse_marker(marker_obj, context)?;
        pie.marker(marker)
    } else {
        pie
    };

    Ok(pie)
}

pub fn parse_pie_trace(
    pie_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<dyn Trace>> {
    Ok(parse_pie_data(pie_obj, context)?)
}
