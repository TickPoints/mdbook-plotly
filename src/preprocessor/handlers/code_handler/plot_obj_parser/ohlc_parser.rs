use super::until::{Map, must_translate};
use crate::translate;
use anyhow::{Result, anyhow};
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
    let ohlc = if let Some(visible) = ohlc_obj.get_mut("visible")
        && visible.is_string()
    {
        use plotly::common::Visible;
        let visible = match visible.as_str().unwrap_or_else(|| unreachable!()) {
            "true" => Visible::True,
            "false" => Visible::False,
            "legendonly" => Visible::LegendOnly,
            unexpected => return Err(anyhow!("{unexpected} can't be visible")),
        };
        ohlc.visible(visible)
    } else {
        ohlc
    };

    // UNEXPECTED: The other methods of the `Ohlc` return only `self`, not boxed `self`.
    Ok(Box::new(ohlc))
}
