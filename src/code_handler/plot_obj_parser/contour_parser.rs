use super::common::parse_color_bar;
use super::until::{Color, must_translate_from_context};
use crate::code_handler::parse_context::ParseContext;
use crate::{translate_enum_with_config, translate_with_config};
use anyhow::Result;
use plotly::{Contour, Trace};

pub fn parse_contour_data(
    ct_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<Contour<Vec<f64>, f64, f64>>> {
    let z: Vec<Vec<f64>> = must_translate_from_context(ct_obj, context, "z")?;
    let contour = if ct_obj.get("x").is_some() {
        let x: Vec<f64> = must_translate_from_context(ct_obj, context, "x")?;
        let y: Vec<f64> = must_translate_from_context(ct_obj, context, "y")?;
        Contour::new(x, y, z)
    } else {
        Contour::new_z(z)
    };

    let contour = translate_with_config! {
        contour,
        ct_obj,
        context.map(),
        context.map_eval(),
        (x0, f64),
        (dx, f64),
        (y0, f64),
        (dy, f64),
        (opacity, f64),
        (n_contours, usize),
        (connect_gaps, bool),
        (hover_on_gaps, bool),
        (show_legend, bool),
        (transpose, bool),
        (auto_contour, bool),
        (auto_color_scale, bool),
        (reverse_scale, bool),
        (show_scale, bool),
        (zauto, bool),
        (fill_color, Color),
    }?;

    use plotly::common::HoverInfo;
    let contour = translate_enum_with_config! {
        contour,
        ct_obj,
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

    let contour = if let Some(contours_obj) = ct_obj.get_mut("contours")
        && contours_obj.is_object()
    {
        use plotly::traces::contour::Contours;
        let contours = translate_with_config! {
            Contours::new(),
            contours_obj,
            context.map(),
            context.map_eval(),
            (start, f64),
            (end, f64),
            (size, f64),
        }?;

        use plotly::traces::contour::Coloring;
        let contours = translate_enum_with_config! {
            contours,
            contours_obj,
            context.map(),
            context.map_eval(),
            (coloring, {
                "fill"        => Coloring::Fill,
                "heatmap"     => Coloring::HeatMap,
                "lines"       => Coloring::Lines,
                "none"        => Coloring::None,
            }),
        }?;
        contour.contours(contours)
    } else {
        contour
    };

    let contour = if let Some(line_obj) = ct_obj.get_mut("line")
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
        contour.line(line)
    } else {
        contour
    };

    let contour = if let Some(color_bar_obj) = ct_obj.get_mut("color_bar")
        && color_bar_obj.is_object()
    {
        let color_bar = parse_color_bar(color_bar_obj, context)?;
        contour.color_bar(color_bar)
    } else {
        contour
    };

    let contour = if ct_obj.get("color_scale").is_some() {
        use plotly::common::{ColorScale, ColorScalePalette};
        let color_scale_str: String = must_translate_from_context(ct_obj, context, "color_scale")?;
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
        contour.color_scale(ColorScale::Palette(palette))
    } else {
        contour
    };

    Ok(contour)
}

pub fn parse_contour_trace(
    ct_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<dyn Trace>> {
    Ok(parse_contour_data(ct_obj, context)?)
}
