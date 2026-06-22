use super::common::parse_marker;
use super::until::must_translate_from_context;
use crate::code_handler::parse_context::ParseContext;
use crate::{translate_enum_with_config, translate_with_config};
use anyhow::{Ok, Result};
use plotly::{ScatterPolar, Trace};

pub fn parse_scatter_polar_data(
    sp_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<ScatterPolar<f64, f64>>> {
    let theta: Vec<f64> = must_translate_from_context(sp_obj, context, "theta")?;
    let r: Vec<f64> = must_translate_from_context(sp_obj, context, "r")?;
    let sp = ScatterPolar::new(theta, r);
    let sp = translate_with_config! {
        sp,
        sp_obj,
        context.map(),
        context.map_eval(),
        (name, String),
        (show_legend, bool),
        (legend_group, String),
        (opacity, f64),
        (text, String),
        (text_array, Vec<String>),
        (hover_text, String),
        (hover_text_array, Vec<String>),
        (hover_template, String),
        (hover_template_array, Vec<String>),
        (subplot, String),
        (connect_gaps, bool),
        (r0, f64),
        (dr, f64),
        (theta0, f64),
        (dtheta, f64),
    }?;

    use plotly::common::Fill;
    use plotly::common::Mode;
    use plotly::common::Visible;
    let sp = translate_enum_with_config! {
        sp,
        sp_obj,
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
        (visible, {
            "true" =>       Visible::True,
            "false" =>      Visible::False,
            "legendonly" => Visible::LegendOnly,
        }),
    }?;

    let sp = if let Some(marker_obj) = sp_obj.get_mut("marker")
        && marker_obj.is_object()
    {
        let marker = parse_marker(marker_obj, context)?;
        sp.marker(marker)
    } else {
        sp
    };

    Ok(sp)
}

pub fn parse_scatter_polar_trace(
    sp_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<dyn Trace>> {
    Ok(parse_scatter_polar_data(sp_obj, context)?)
}
