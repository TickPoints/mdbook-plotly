use super::common::parse_color_bar;
use super::until::must_translate_from_context;
use crate::code_handler::parse_context::ParseContext;
use crate::{translate_enum_with_config, translate_with_config};
use anyhow::Result;
use plotly::HeatMap;

pub fn parse_heat_map_data(
    hm_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<HeatMap<f64, f64, Vec<f64>>>> {
    let z: Vec<Vec<f64>> = must_translate_from_context(hm_obj, context, "z")?;
    let heat_map = if hm_obj.get("x").is_some() {
        let x: Vec<f64> = must_translate_from_context(hm_obj, context, "x")?;
        let y: Vec<f64> = must_translate_from_context(hm_obj, context, "y")?;
        HeatMap::new(x, y, z)
    } else {
        HeatMap::new_z(z)
    };

    let heat_map = translate_with_config! {
        heat_map,
        hm_obj,
        context.map(),
        context.map_eval(),
        (name, String),
        (opacity, f64),
        (hover_template, String),
        (hover_template_array, Vec<String>),
        (hover_text, String),
        (hover_text_array, Vec<String>),
        (hover_text_matrix, Vec<Vec<String>>),
        (text, String),
        (text_array, Vec<String>),
        (text_matrix, Vec<Vec<String>>),
        (show_legend, bool),
        (legend_group, String),
        (x_axis, String),
        (y_axis, String),
        (connect_gaps, bool),
        (transpose, bool),
        (auto_color_scale, bool),
        (reverse_scale, bool),
        (show_scale, bool),
        (zauto, bool),
        (zmax, f64),
        (zmin, f64),
        (zmid, f64),
        (x_gap, usize),
        (y_gap, usize),
    }?;

    use plotly::common::HoverInfo;
    let heat_map = translate_enum_with_config! {
        heat_map,
        hm_obj,
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

    let heat_map = if let Some(color_bar_obj) = hm_obj.get_mut("color_bar")
        && color_bar_obj.is_object()
    {
        let color_bar = parse_color_bar(color_bar_obj, context)?;
        heat_map.color_bar(color_bar)
    } else {
        heat_map
    };

    let heat_map = if hm_obj.get("color_scale").is_some() {
        use plotly::common::{ColorScale, ColorScalePalette};
        let color_scale_str: String = must_translate_from_context(hm_obj, context, "color_scale")?;
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
        heat_map.color_scale(ColorScale::Palette(palette))
    } else {
        heat_map
    };

    Ok(heat_map)
}
