use super::until::must_translate_from_context;
use crate::code_handler::parse_context::ParseContext;
use crate::{translate_enum_with_config, translate_with_config};
use anyhow::Result;
use plotly::{Image, Trace, color::Rgba};

pub fn parse_image_data(
    image_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<Image>> {
    let z: Vec<Vec<Rgba>> = must_translate_from_context(image_obj, context, "z")?;
    let image = Image::new(z);
    let image = translate_with_config! {
        image,
        image_obj,
        context.map(),
        context.map_eval(),
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
    let image = translate_enum_with_config! {
        image,
        image_obj,
        context.map(),
        context.map_eval(),
        (z_smooth, {
            "fast" =>   ZSmooth::Fast,
            "false" =>  ZSmooth::False,
        }),
    }?;

    Ok(image)
}

pub fn parse_image_trace(
    image_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<dyn Trace>> {
    Ok(parse_image_data(image_obj, context)?)
}
