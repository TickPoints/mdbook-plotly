use super::common::parse_color_bar;
use super::until::{Color, must_translate_from_context};
use crate::code_handler::parse_context::ParseContext;
use crate::{translate_enum_with_config, translate_with_config};
use anyhow::Result;
use plotly::Surface;

pub fn parse_surface_data(
    sf_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<Surface<f64, f64, f64>>> {
    let z: Vec<Vec<f64>> = must_translate_from_context(sf_obj, context, "z")?;
    let surface = Surface::new(z);

    let surface = translate_with_config! {
        surface,
        sf_obj,
        context.map(),
        context.map_eval(),
        (x, Vec<f64>),
        (y, Vec<f64>),
        (name, String),
        (opacity, f64),
        (text, String),
        (text_array, Vec<String>),
        (hover_text, String),
        (hover_text_array, Vec<String>),
        (hover_template, String),
        (hover_template_array, Vec<String>),
        (show_legend, bool),
        (legend_group, String),
        (connect_gaps, bool),
        (hide_surface, bool),
        (surface_color, Vec<Color>),
        (auto_color_scale, bool),
        (reverse_scale, bool),
        (show_scale, bool),
        (cauto, bool),
        (cmax, f64),
        (cmin, f64),
        (cmid, f64),
    }?;

    use plotly::common::HoverInfo;
    let surface = translate_enum_with_config! {
        surface,
        sf_obj,
        context.map(),
        context.map_eval(),
        (hover_info, {
            "all" => HoverInfo::All,
            "x" => HoverInfo::X,
            "y" => HoverInfo::Y,
            "z" => HoverInfo::Z,
            "x+y" => HoverInfo::XAndY,
            "x+z" => HoverInfo::XAndZ,
            "y+z" => HoverInfo::YAndZ,
            "x+y+z" => HoverInfo::XAndYAndZ,
            "text" => HoverInfo::Text,
            "name" => HoverInfo::Name,
            "none" => HoverInfo::None,
            "skip" => HoverInfo::Skip,
        }),
    }?;

    let surface = if let Some(color_bar_obj) = sf_obj.get_mut("color_bar")
        && color_bar_obj.is_object()
    {
        let color_bar = parse_color_bar(color_bar_obj, context)?;
        surface.color_bar(color_bar)
    } else {
        surface
    };

    let surface = if sf_obj.get("color_scale").is_some() {
        use plotly::common::{ColorScale, ColorScalePalette};
        let color_scale_str: String = must_translate_from_context(sf_obj, context, "color_scale")?;
        let palette = match color_scale_str.to_lowercase().as_str() {
            "greys" => ColorScalePalette::Greys,
            "ylgnbu" => ColorScalePalette::YlGnBu,
            "greens" => ColorScalePalette::Greens,
            "ylorrd" => ColorScalePalette::YlOrRd,
            "bluered" => ColorScalePalette::Bluered,
            "rdbu" => ColorScalePalette::RdBu,
            "reds" => ColorScalePalette::Reds,
            "blues" => ColorScalePalette::Blues,
            "picnic" => ColorScalePalette::Picnic,
            "rainbow" => ColorScalePalette::Rainbow,
            "portland" => ColorScalePalette::Portland,
            "jet" => ColorScalePalette::Jet,
            "hot" => ColorScalePalette::Hot,
            "blackbody" => ColorScalePalette::Blackbody,
            "earth" => ColorScalePalette::Earth,
            "electric" => ColorScalePalette::Electric,
            "viridis" => ColorScalePalette::Viridis,
            "cividis" => ColorScalePalette::Cividis,
            _ => return Err(anyhow::anyhow!("unknown color_scale: {}", color_scale_str)),
        };
        surface.color_scale(ColorScale::Palette(palette))
    } else {
        surface
    };

    let surface = if let Some(lighting_obj) = sf_obj.get_mut("lighting")
        && lighting_obj.is_object()
    {
        use plotly::traces::surface::Lighting;
        let lighting = translate_with_config! {
            Lighting::new(),
            lighting_obj,
            context.map(),
            context.map_eval(),
            (ambient, f64),
            (diffuse, f64),
            (specular, f64),
            (roughness, f64),
            (fresnel, f64),
        }?;
        surface.lighting(lighting)
    } else {
        surface
    };

    let surface = if let Some(light_pos_obj) = sf_obj.get_mut("light_position")
        && light_pos_obj.is_object()
    {
        use plotly::traces::surface::Position;
        let x: i32 = must_translate_from_context(light_pos_obj, context, "x")?;
        let y: i32 = must_translate_from_context(light_pos_obj, context, "y")?;
        let z: i32 = must_translate_from_context(light_pos_obj, context, "z")?;
        surface.light_position(Position::new(x, y, z))
    } else {
        surface
    };

    Ok(surface)
}
