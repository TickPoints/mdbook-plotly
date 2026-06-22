use super::common::parse_color_bar;
use super::until::{Color, must_translate_from_context};
use crate::code_handler::parse_context::ParseContext;
use crate::{translate_enum_with_config, translate_with_config};
use anyhow::Result;
use plotly::{Mesh3D, Trace};

pub fn parse_mesh3d_data(
    mesh_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<Mesh3D<f64, f64, f64>>> {
    let x: Vec<f64> = must_translate_from_context(mesh_obj, context, "x")?;
    let y: Vec<f64> = must_translate_from_context(mesh_obj, context, "y")?;
    let z: Vec<f64> = must_translate_from_context(mesh_obj, context, "z")?;
    let i: Option<Vec<usize>> = if mesh_obj.get("i").is_some() {
        Some(must_translate_from_context(mesh_obj, context, "i")?)
    } else {
        None
    };
    let j: Option<Vec<usize>> = if mesh_obj.get("j").is_some() {
        Some(must_translate_from_context(mesh_obj, context, "j")?)
    } else {
        None
    };
    let k: Option<Vec<usize>> = if mesh_obj.get("k").is_some() {
        Some(must_translate_from_context(mesh_obj, context, "k")?)
    } else {
        None
    };
    let mesh = Mesh3D::new(x, y, z, i, j, k);

    let mesh = translate_with_config! {
        mesh,
        mesh_obj,
        context.map(),
        context.map_eval(),
        (name, String),
        (opacity, f64),
        (ids, Vec<String>),
        (text, String),
        (text_array, Vec<String>),
        (hover_text, String),
        (hover_text_array, Vec<String>),
        (hover_template, String),
        (hover_template_array, Vec<String>),
        (show_legend, bool),
        (legend_group, String),
        (legend_rank, usize),
        (color, Color),
        (face_color, Vec<Color>),
        (vertex_color, Vec<Color>),
        (intensity, Vec<f64>),
        (scene, String),
        (flat_shading, bool),
        (alpha_hull, f64),
        (meta, String),
        (color_axis, String),
        (auto_color_scale, bool),
        (reverse_scale, bool),
        (show_scale, bool),
        (c_auto, bool),
        (c_max, f64),
        (c_min, f64),
        (c_mid, f64),
    }?;

    use plotly::common::HoverInfo;
    let mesh = translate_enum_with_config! {
        mesh,
        mesh_obj,
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

    use plotly::traces::mesh3d::{DelaunayAxis, IntensityMode};
    let mesh = translate_enum_with_config! {
        mesh,
        mesh_obj,
        context.map(),
        context.map_eval(),
        (intensity_mode, {
            "vertex" => IntensityMode::Vertex,
            "cell" => IntensityMode::Cell,
        }),
        (delaunay_axis, {
            "x" => DelaunayAxis::X,
            "y" => DelaunayAxis::Y,
            "z" => DelaunayAxis::Z,
        }),
    }?;

    let mesh = if let Some(color_bar_obj) = mesh_obj.get_mut("color_bar")
        && color_bar_obj.is_object()
    {
        let color_bar = parse_color_bar(color_bar_obj, context)?;
        mesh.color_bar(color_bar)
    } else {
        mesh
    };

    let mesh = if mesh_obj.get("color_scale").is_some() {
        use plotly::common::{ColorScale, ColorScalePalette};
        let color_scale_str: String =
            must_translate_from_context(mesh_obj, context, "color_scale")?;
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
        mesh.color_scale(ColorScale::Palette(palette))
    } else {
        mesh
    };

    let mesh = if let Some(lighting_obj) = mesh_obj.get_mut("lighting")
        && lighting_obj.is_object()
    {
        use plotly::traces::mesh3d::Lighting;
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
        mesh.lighting(lighting)
    } else {
        mesh
    };

    let mesh = if let Some(light_pos_obj) = mesh_obj.get_mut("light_position")
        && light_pos_obj.is_object()
    {
        use plotly::traces::mesh3d::LightPosition;
        let light_pos = translate_with_config! {
            LightPosition::new(),
            light_pos_obj,
            context.map(),
            context.map_eval(),
            (x, Vec<f64>),
            (y, Vec<f64>),
            (z, Vec<f64>),
        }?;
        mesh.light_position(light_pos)
    } else {
        mesh
    };

    Ok(mesh)
}

pub fn parse_mesh3d_trace(
    mesh_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<dyn Trace>> {
    Ok(parse_mesh3d_data(mesh_obj, context)?)
}
