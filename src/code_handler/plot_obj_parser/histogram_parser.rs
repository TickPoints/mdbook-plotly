use super::common::parse_marker;
use super::until::must_translate_from_context;
use crate::code_handler::parse_context::ParseContext;
use crate::{translate_enum_with_config, translate_with_config};
use anyhow::{Result, anyhow};
use plotly::{Histogram, Trace};

pub fn parse_histogram_data(
    hist_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<Histogram<f64>>> {
    let has_x = hist_obj.get("x").is_some();
    let has_y = hist_obj.get("y").is_some();
    let hist = match (has_x, has_y) {
        (true, true) => {
            let x: Vec<f64> = must_translate_from_context(hist_obj, context, "x")?;
            let y: Vec<f64> = must_translate_from_context(hist_obj, context, "y")?;
            Histogram::new_xy(x, y)
        }
        (true, false) => {
            let x: Vec<f64> = must_translate_from_context(hist_obj, context, "x")?;
            Histogram::new(x)
        }
        (false, true) => {
            let y: Vec<f64> = must_translate_from_context(hist_obj, context, "y")?;
            Histogram::new_vertical(y)
        }
        (false, false) => {
            return Err(anyhow!("histogram requires at least 'x' or 'y' data"));
        }
    };
    let hist = translate_with_config! {
        hist,
        hist_obj,
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
        (auto_bin_x, bool),
        (n_bins_x, usize),
        (auto_bin_y, bool),
        (n_bins_y, usize),
        (alignment_group, String),
        (offset_group, String),
        (bin_group, String),
        (x_axis, String),
        (y_axis, String),
    }?;

    use plotly::common::Orientation;
    use plotly::histogram::{HistFunc, HistNorm};

    let hist = translate_enum_with_config! {
        hist,
        hist_obj,
        context.map(),
        context.map_eval(),
        (orientation, {
            "v" => Orientation::Vertical,
            "h" => Orientation::Horizontal,
        }),
        (hist_func, {
            "count" => HistFunc::Count,
            "sum"   => HistFunc::Sum,
            "avg"   => HistFunc::Average,
            "min"   => HistFunc::Minimum,
            "max"   => HistFunc::Maximum,
        }),
        (hist_norm, {
            "percent"             => HistNorm::Percent,
            "probability"         => HistNorm::Probability,
            "density"             => HistNorm::Density,
            "probability density" => HistNorm::ProbabilityDensity,
            ""                    => HistNorm::Default,
        }),
    }?;

    let hist = if let Some(marker_obj) = hist_obj.get_mut("marker")
        && marker_obj.is_object()
    {
        let marker = parse_marker(marker_obj, context)?;
        hist.marker(marker)
    } else {
        hist
    };

    Ok(hist)
}

pub fn parse_histogram_trace(
    hist_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<dyn Trace>> {
    Ok(parse_histogram_data(hist_obj, context)?)
}
