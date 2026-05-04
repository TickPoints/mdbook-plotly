use super::until::{Map, must_translate};
use crate::{translate, translate_enum};
use anyhow::Result;
use plotly::Candlestick;

pub fn parse_candlestick_data(
    cs_obj: &mut serde_json::Value,
    map: &Map,
) -> Result<Box<Candlestick<String, f64>>> {
    let x: Vec<String> = must_translate(cs_obj, map, "x")?;
    let open: Vec<f64> = must_translate(cs_obj, map, "open")?;
    let high: Vec<f64> = must_translate(cs_obj, map, "high")?;
    let low: Vec<f64> = must_translate(cs_obj, map, "low")?;
    let close: Vec<f64> = must_translate(cs_obj, map, "close")?;
    let cs = Candlestick::new(x, open, high, low, close);
    let cs = translate! {
        // UNEXPECTED: The other methods of the `Ohlc` return only `self`, not boxed `self`.
        *cs,
        cs_obj,
        map,
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
    let cs = translate_enum! {
        cs,
        cs_obj,
        map,
        (visible, {
            "true" =>       Visible::True,
            "false" =>      Visible::False,
            "legendonly" => Visible::LegendOnly,
        }),
    }?;

    // UNEXPECTED: The other methods of the `Ohlc` return only `self`, not boxed `self`.
    Ok(Box::new(cs))
}
