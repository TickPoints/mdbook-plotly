use super::until::{Map, must_translate};
use crate::{translate, translate_enum};
use anyhow::Result;
use plotly::ScatterPolar;

pub fn parse_scatter_polar_data(
    sp_obj: &mut serde_json::Value,
    map: &Map,
) -> Result<Box<ScatterPolar<f64, f64>>> {
    let theta: Vec<f64> = must_translate(sp_obj, map, "theta")?;
    let r: Vec<f64> = must_translate(sp_obj, map, "r")?;
    let sp = ScatterPolar::new(theta, r);
    let sp = translate! {
        sp,
        sp_obj,
        map,
        (name, String),
        (show_legend, bool),
        (legend_group, String),
        (opacity, f64),
        (text, String),
        (text_array, Vec<String>),
        (hover_text, String),
        (hover_text_array, Vec<String>),
        (hover_template, String),
        (hover_template_array, Vec<String>),
        (subplot, String),
        (connect_gaps, bool),
        (r0, f64),
        (dr, f64),
        (theta0, f64),
        (dtheta, f64),
    }?;

    use plotly::common::Fill;
    use plotly::common::Mode;
    use plotly::common::Visible;
    let sp = translate_enum! {
        sp,
        sp_obj,
        map,
        (fill, {
            "tozeroy" => Fill::ToZeroY,
            "tozerox" => Fill::ToZeroX,
            "tonexty" => Fill::ToNextY,
            "tonextx" => Fill::ToNextX,
            "toself" =>  Fill::ToSelf,
            "tonext" =>  Fill::ToNext,
            "none" =>    Fill::None,
        }),
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
        (visible, {
            "true" =>       Visible::True,
            "false" =>      Visible::False,
            "legendonly" => Visible::LegendOnly,
        }),
    }?;

    Ok(sp)
}
