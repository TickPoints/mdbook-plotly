use super::until::{Map, must_translate};
use crate::{translate, translate_enum};
use anyhow::Result;
use plotly::Ohlc;

pub fn parse_ohlc_data(
    ohlc_obj: &mut serde_json::Value,
    map: &Map,
) -> Result<Box<Ohlc<String, f64>>> {
    let x: Vec<String> = must_translate(ohlc_obj, map, "x")?;
    let open: Vec<f64> = must_translate(ohlc_obj, map, "open")?;
    let high: Vec<f64> = must_translate(ohlc_obj, map, "high")?;
    let low: Vec<f64> = must_translate(ohlc_obj, map, "low")?;
    let close: Vec<f64> = must_translate(ohlc_obj, map, "close")?;
    let ohlc = Ohlc::new(x, open, high, low, close);
    let ohlc = translate! {
        *ohlc,
        // UNEXPECTED: The other methods of the `Ohlc` return only `self`, not boxed `self`.
        ohlc_obj,
        map,
        (name, String),
        (show_legend, bool),
        (legend_group, String),
        (opacity, f64),
        (hover_text, String),
        (hover_text_array, Vec<String>),
        (tick_width, f64),
    }?;

    use plotly::common::Visible;
    let ohlc = translate_enum! {
        ohlc,
        ohlc_obj,
        map,
        (visible, {
            "true" =>       Visible::True,
            "false" =>      Visible::False,
            "legendonly" => Visible::LegendOnly,
        }),
    }?;

    // UNEXPECTED: The other methods of the `Ohlc` return only `self`, not boxed `self`.
    Ok(Box::new(ohlc))
}
