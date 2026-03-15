use super::until::{Map, must_translate};
use crate::{translate, translate_enum};
use anyhow::Result;
use plotly::{Image, color::Rgba};

pub fn parse_image_data(image_obj: &mut serde_json::Value, map: &Map) -> Result<Box<Image>> {
    let z: Vec<Vec<Rgba>> = must_translate(image_obj, map, "z")?;
    let image = Image::new(z);
    let image = translate! {
        image,
        image_obj,
        map,
        (opacity, f64),
        (name, String),
        (legend_rank, usize),
        (text, String),
        (text_array, Vec<String>),
        (hover_text, String),
        (hover_text_array, Vec<String>),
        (hover_template, String),
        (hover_template_array, Vec<String>),
        (source, String),
        (x0, f64),
        (dx, f64),
        (y0, f64),
        (dy, f64),
        (x_axis, String),
        (y_axis, String),
        (ids, Vec<String>),
        (meta, String),
    }?;

    use plotly::image::ZSmooth;
    let image = translate_enum! {
        image,
        image_obj,
        map,
        (z_smooth, {
            "fast" =>   ZSmooth::Fast,
            "false" =>  ZSmooth::False,
        }),
    }?;

    Ok(image)
}
