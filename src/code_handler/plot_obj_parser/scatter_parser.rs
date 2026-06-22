use super::common::parse_marker;
use super::until::{Color, must_translate_from_context};
use crate::code_handler::parse_context::ParseContext;
use crate::{translate_enum_with_config, translate_with_config};
use anyhow::Result;
use plotly::{Scatter, Trace};

pub fn parse_scatter_data(
    sc_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<Scatter<f64, f64>>> {
    let x: Vec<f64> = must_translate_from_context(sc_obj, context, "x")?;
    let y: Vec<f64> = must_translate_from_context(sc_obj, context, "y")?;
    let sc = Scatter::new(x, y);
    let sc = translate_with_config! {
        sc,
        sc_obj,
        context.map(),
        context.map_eval(),
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
    let sc = translate_enum_with_config! {
        sc,
        sc_obj,
        context.map(),
        context.map_eval(),
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
        let marker = parse_marker(marker_obj, context)?;
        sc.marker(marker)
    } else {
        sc
    };

    Ok(sc)
}

pub fn parse_scatter_trace(
    sc_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<dyn Trace>> {
    Ok(parse_scatter_data(sc_obj, context)?)
}
