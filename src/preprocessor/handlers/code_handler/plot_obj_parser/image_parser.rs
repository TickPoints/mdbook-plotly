use super::until::{Map, must_translate};
use crate::translate;
use anyhow::{Result, anyhow};
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
    let image = if let Some(zs) = image_obj.get_mut("z_smooth")
        && zs.is_string()
    {
        use plotly::image::ZSmooth;
        let zs = match zs.as_str().unwrap_or_else(|| unreachable!()) {
            "fast" => ZSmooth::Fast,
            "false" => ZSmooth::False,
            unexpected => return Err(anyhow!("{unexpected} can't be z_smooth")),
        };
        image.z_smooth(zs)
    } else {
        image
    };
    Ok(image)
}
