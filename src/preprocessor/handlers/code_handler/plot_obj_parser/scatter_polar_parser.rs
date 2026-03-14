use super::until::{Map, must_translate};
use crate::translate;
use anyhow::{Result, anyhow};
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
    let sp = if let Some(visible) = sp_obj.get_mut("visible")
        && visible.is_string()
    {
        use plotly::common::Visible;
        let visible = match visible.as_str().unwrap_or_else(|| unreachable!()) {
            "true" => Visible::True,
            "false" => Visible::False,
            "legendonly" => Visible::LegendOnly,
            unexpected => return Err(anyhow!("{unexpected} can't be visible")),
        };
        sp.visible(visible)
    } else {
        sp
    };
    let sp = if let Some(mode) = sp_obj.get_mut("mode")
        && mode.is_string()
    {
        use plotly::common::Mode;
        let mode = match mode.as_str().unwrap_or_else(|| unreachable!()) {
            "lines" => Mode::Lines,
            "markers" => Mode::Markers,
            "text" => Mode::Text,
            "linesmarkers" => Mode::LinesMarkers,
            "linestext" => Mode::LinesText,
            "markerstext" => Mode::MarkersText,
            "linemarkerstext" => Mode::LinesMarkersText,
            "none" => Mode::None,
            unexpected => return Err(anyhow!("{unexpected} can't be mode")),
        };
        sp.mode(mode)
    } else {
        sp
    };
    let sp = if let Some(fill) = sp_obj.get_mut("fill")
        && fill.is_string()
    {
        use plotly::common::Fill;
        let fill = match fill.as_str().unwrap_or_else(|| unreachable!()) {
            "tozeroy" => Fill::ToZeroY,
            "tozerox" => Fill::ToZeroX,
            "tonexty" => Fill::ToNextY,
            "tonextx" => Fill::ToNextX,
            "toself" => Fill::ToSelf,
            "tonext" => Fill::ToNext,
            "none" => Fill::None,
            unexpected => return Err(anyhow!("{unexpected} can't be fill")),
        };
        sp.fill(fill)
    } else {
        sp
    };
    Ok(sp)
}
