use super::until::must_translate;
use crate::translate;
use anyhow::Result;
use plotly::Bar;

pub fn parse_bar_data(bar_obj: &mut serde_json::Value) -> Result<Box<Bar<u64, u64>>> {
    let x: Vec<u64> = must_translate(bar_obj, "x")?;
    let y: Vec<u64> = must_translate(bar_obj, "y")?;
    let bar = Bar::new(x, y);
    let bar = translate! {
        bar,
        bar_obj,
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
        (x_axis, String),
        (y_axis, String),
        (alignment_group, String),
        (offset_group, String),
        (clip_on_axis, bool),
    }?;
    Ok(bar)
}
