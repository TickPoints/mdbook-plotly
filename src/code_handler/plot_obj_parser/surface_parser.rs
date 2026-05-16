use super::until::{Color, Map, must_translate};
use crate::{translate, translate_enum};
use anyhow::Result;
use plotly::Surface;

pub fn parse_surface_data(
    sf_obj: &mut serde_json::Value,
    map: &Map,
) -> Result<Box<Surface<f64, f64, f64>>> {
    let z: Vec<Vec<f64>> = must_translate(sf_obj, map, "z")?;
    let surface = Surface::new(z);

    let surface = translate! {
        surface,
        sf_obj,
        map,
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
    let surface = translate_enum! {
        surface,
        sf_obj,
        map,
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
        use plotly::common::ColorBar;
        let color_bar = translate! {
            ColorBar::new(),
            color_bar_obj,
            map,
            (thickness, usize),
            (len, usize),
            (x, f64),
            (y, f64),
            (title, String),
        }?;
        surface.color_bar(color_bar)
    } else {
        surface
    };

    let surface = if sf_obj.get("color_scale").is_some() {
        use plotly::common::{ColorScale, ColorScalePalette};
        let color_scale_str: String = must_translate(sf_obj, map, "color_scale")?;
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
        let lighting = translate! {
            Lighting::new(),
            lighting_obj,
            map,
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
        let x: i32 = must_translate(light_pos_obj, map, "x")?;
        let y: i32 = must_translate(light_pos_obj, map, "y")?;
        let z: i32 = must_translate(light_pos_obj, map, "z")?;
        surface.light_position(Position::new(x, y, z))
    } else {
        surface
    };

    Ok(surface)
}
