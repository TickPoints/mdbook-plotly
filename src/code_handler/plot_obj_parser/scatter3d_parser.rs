use super::common::parse_marker;
use super::until::{Color, must_translate_from_context};
use crate::code_handler::parse_context::ParseContext;
use crate::{translate_enum_with_config, translate_with_config};
use anyhow::Result;
use plotly::{Scatter3D, Trace};

pub fn parse_scatter3d_data(
    s3_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<Scatter3D<f64, f64, f64>>> {
    let x: Vec<f64> = must_translate_from_context(s3_obj, context, "x")?;
    let y: Vec<f64> = must_translate_from_context(s3_obj, context, "y")?;
    let z: Vec<f64> = must_translate_from_context(s3_obj, context, "z")?;
    let s3 = Scatter3D::new(x, y, z);

    let s3 = translate_with_config! {
        s3,
        s3_obj,
        context.map(),
        context.map_eval(),
        (name, String),
        (opacity, f64),
        (ids, Vec<String>),
        (text, String),
        (text_array, Vec<String>),
        (text_template, String),
        (text_template_array, Vec<String>),
        (hover_text, String),
        (hover_text_array, Vec<String>),
        (hover_template, String),
        (hover_template_array, Vec<String>),
        (show_legend, bool),
        (legend_group, String),
        (legend_rank, usize),
        (surface_color, Color),
        (connect_gaps, bool),
        (scene, String),
        (meta, String),
    }?;

    use plotly::common::{HoverInfo, Mode, Position};
    let s3 = translate_enum_with_config! {
        s3,
        s3_obj,
        context.map(),
        context.map_eval(),
        (mode, {
            "lines" =>          Mode::Lines,
            "markers" =>        Mode::Markers,
            "text" =>           Mode::Text,
            "linesmarkers" =>   Mode::LinesMarkers,
            "linestext" =>      Mode::LinesText,
            "markerstext" =>    Mode::MarkersText,
            "linemarkerstext" =>Mode::LinesMarkersText,
            "none" =>           Mode::None,
        }),
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
        (text_position, {
            "top left" =>       Position::TopLeft,
            "top center" =>     Position::TopCenter,
            "top right" =>      Position::TopRight,
            "middle left" =>    Position::MiddleLeft,
            "middle center" =>  Position::MiddleCenter,
            "middle right" =>   Position::MiddleRight,
            "bottom left" =>    Position::BottomLeft,
            "bottom center" =>  Position::BottomCenter,
            "bottom right" =>   Position::BottomRight,
        }),
    }?;

    use plotly::traces::scatter3d::SurfaceAxis;
    let s3 = translate_enum_with_config! {
        s3,
        s3_obj,
        context.map(),
        context.map_eval(),
        (surface_axis, {
            "-1" => SurfaceAxis::MinusOne,
            "0" =>  SurfaceAxis::Zero,
            "1" =>  SurfaceAxis::One,
            "2" =>  SurfaceAxis::Two,
        }),
    }?;

    let s3 = if let Some(marker_obj) = s3_obj.get_mut("marker")
        && marker_obj.is_object()
    {
        let marker = parse_marker(marker_obj, context)?;
        s3.marker(marker)
    } else {
        s3
    };

    let s3 = if let Some(line_obj) = s3_obj.get_mut("line")
        && line_obj.is_object()
    {
        use plotly::common::Line;
        let line = translate_with_config! {
            Line::new(),
            line_obj,
            context.map(),
            context.map_eval(),
            (color, Color),
            (width, f64),
        }?;
        use plotly::common::DashType;
        let line = translate_enum_with_config! {
            line,
            line_obj,
            context.map(),
            context.map_eval(),
            (dash, {
                "solid" => DashType::Solid,
                "dot" => DashType::Dot,
                "dash" => DashType::Dash,
                "longdash" => DashType::LongDash,
                "dashdot" => DashType::DashDot,
                "longdashdot" => DashType::LongDashDot,
            }),
        }?;
        s3.line(line)
    } else {
        s3
    };

    Ok(s3)
}

pub fn parse_scatter3d_trace(
    s3_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<dyn Trace>> {
    Ok(parse_scatter3d_data(s3_obj, context)?)
}
