use super::until::must_translate_from_context;
use crate::code_handler::parse_context::ParseContext;
use crate::{translate_enum_with_config, translate_with_config};
use anyhow::Result;
use plotly::Candlestick;
use plotly::Trace;

pub fn parse_candlestick_data(
    cs_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<dyn Trace>> {
    let x: Vec<String> = must_translate_from_context(cs_obj, context, "x")?;
    let open: Vec<f64> = must_translate_from_context(cs_obj, context, "open")?;
    let high: Vec<f64> = must_translate_from_context(cs_obj, context, "high")?;
    let low: Vec<f64> = must_translate_from_context(cs_obj, context, "low")?;
    let close: Vec<f64> = must_translate_from_context(cs_obj, context, "close")?;
    let cs = Candlestick::new(x, open, high, low, close);
    let cs = translate_with_config! {
        // UNEXPECTED: The other methods of the `Ohlc` return only `self`, not boxed `self`.
        *cs,
        cs_obj,
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
        (whisker_width, f64),
        (x_axis, String),
        (y_axis, String),
    }?;

    use plotly::common::Visible;
    let cs = translate_enum_with_config! {
        cs,
        cs_obj,
        context.map(),
        context.map_eval(),
        (visible, {
            "true" =>       Visible::True,
            "false" =>      Visible::False,
            "legendonly" => Visible::LegendOnly,
        }),
    }?;

    // UNEXPECTED: The other methods of the `Ohlc` return only `self`, not boxed `self`.
    Ok(Box::new(cs))
}
