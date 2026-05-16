use super::until::{Color, Map, must_translate};
use crate::{translate, translate_enum};
use anyhow::Result;
use plotly::Contour;

pub fn parse_contour_data(
    ct_obj: &mut serde_json::Value,
    map: &Map,
) -> Result<Box<Contour<Vec<f64>, f64, f64>>> {
    let z: Vec<Vec<f64>> = must_translate(ct_obj, map, "z")?;
    let contour = if ct_obj.get("x").is_some() {
        let x: Vec<f64> = must_translate(ct_obj, map, "x")?;
        let y: Vec<f64> = must_translate(ct_obj, map, "y")?;
        Contour::new(x, y, z)
    } else {
        Contour::new_z(z)
    };

    let contour = translate! {
        contour,
        ct_obj,
        map,
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
    let contour = translate_enum! {
        contour,
        ct_obj,
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

    let contour = if let Some(contours_obj) = ct_obj.get_mut("contours")
        && contours_obj.is_object()
    {
        use plotly::traces::contour::Contours;
        let contours = translate! {
            Contours::new(),
            contours_obj,
            map,
            (start, f64),
            (end, f64),
            (size, f64),
        }?;

        use plotly::traces::contour::Coloring;
        let contours = translate_enum! {
            contours,
            contours_obj,
            map,
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
        let line = translate! {
            Line::new(),
            line_obj,
            map,
            (color, Color),
            (width, f64),
        }?;
        use plotly::common::DashType;
        let line = translate_enum! {
            line,
            line_obj,
            map,
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
        contour.color_bar(color_bar)
    } else {
        contour
    };

    let contour = if ct_obj.get("color_scale").is_some() {
        use plotly::common::{ColorScale, ColorScalePalette};
        let color_scale_str: String = must_translate(ct_obj, map, "color_scale")?;
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
