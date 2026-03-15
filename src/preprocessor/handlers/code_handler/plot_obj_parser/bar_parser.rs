use super::until::{Map, must_translate};
use crate::{translate, translate_enum};
use anyhow::Result;
use plotly::Bar;

pub fn parse_bar_data(bar_obj: &mut serde_json::Value, map: &Map) -> Result<Box<Bar<f64, f64>>> {
    let x: Vec<f64> = must_translate(bar_obj, map, "x")?;
    let y: Vec<f64> = must_translate(bar_obj, map, "y")?;
    let bar = Bar::new(x, y);
    let bar = translate! {
        bar,
        bar_obj,
        map,
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
        (name, String),
        (opacity, f64),
        (x_axis, String),
        (y_axis, String),
        (alignment_group, String),
        (offset_group, String),
        (clip_on_axis, bool),
        (show_legend, bool),
        (legend_group, String),
        (width, f64),
        (text_angle, f64),
    }?;

    use plotly::common::Orientation;
    let bar = translate_enum! {
        bar,
        bar_obj,
        map,
        (orientation, {
            "v" => Orientation::Vertical,
            "h" => Orientation::Horizontal,
        }),
    }?;
    Ok(bar)
}
