use super::until::must_translate_from_context;
use crate::code_handler::parse_context::ParseContext;
use crate::{translate_enum_with_config, translate_with_config};
use anyhow::Result;
use plotly::{Ohlc, Trace};

pub fn parse_ohlc_data(
    ohlc_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<Ohlc<String, f64>>> {
    let x: Vec<String> = must_translate_from_context(ohlc_obj, context, "x")?;
    let open: Vec<f64> = must_translate_from_context(ohlc_obj, context, "open")?;
    let high: Vec<f64> = must_translate_from_context(ohlc_obj, context, "high")?;
    let low: Vec<f64> = must_translate_from_context(ohlc_obj, context, "low")?;
    let close: Vec<f64> = must_translate_from_context(ohlc_obj, context, "close")?;
    let ohlc = Ohlc::new(x, open, high, low, close);
    let ohlc = translate_with_config! {
        *ohlc,
        // UNEXPECTED: The other methods of the `Ohlc` return only `self`, not boxed `self`.
        ohlc_obj,
        context.map(),
        context.map_eval(),
        (name, String),
        (show_legend, bool),
        (legend_group, String),
        (opacity, f64),
        (hover_text, String),
        (hover_text_array, Vec<String>),
        (tick_width, f64),
    }?;

    use plotly::common::Visible;
    let ohlc = translate_enum_with_config! {
        ohlc,
        ohlc_obj,
        context.map(),
        context.map_eval(),
        (visible, {
            "true" =>       Visible::True,
            "false" =>      Visible::False,
            "legendonly" => Visible::LegendOnly,
        }),
    }?;

    // UNEXPECTED: The other methods of the `Ohlc` return only `self`, not boxed `self`.
    Ok(Box::new(ohlc))
}

pub fn parse_ohlc_trace(
    ohlc_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<dyn Trace>> {
    Ok(parse_ohlc_data(ohlc_obj, context)?)
}
