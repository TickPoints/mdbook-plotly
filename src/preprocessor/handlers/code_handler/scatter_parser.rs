use super::until::must_translate;
use crate::translate;
use anyhow::{Result, anyhow};
use plotly::{Scatter, common::color::Rgba};

pub fn parse_scatter_data(scatter_obj: &mut serde_json::Value) -> Result<Box<Scatter<f64, f64>>> {
    let x: Vec<f64> = must_translate(scatter_obj, "x")?;
    let y: Vec<f64> = must_translate(scatter_obj, "y")?;
    let scatter = Scatter::new(x, y);
    let scatter = translate! {
        scatter,
        scatter_obj,
        (web_gl_mode, bool),
        (x0, f64),
        (dx, f64),
        (y0, f64),
        (dy, f64),
        (ids, Vec<String>),
        (text, String),
        (text_array, Vec<String>),
        (text_template, String),
        (hover_template, String),
        (hover_template_array, Vec<String>),
        (hover_text, String),
        (hover_text_array, Vec<String>),
        (name, String),
        (opacity, f64),
        (meta, String),
        (x_axis, String),
        (y_axis, String),
        (stack_group, String),
        (clip_on_axis, bool),
        (connect_gaps, bool),
        (fill_color, Rgba),
    }?;
    let scatter = if let Some(fill) = scatter_obj.get_mut("fill")
        && fill.is_string() {
        // Safety: This `unwrap` will never be reached.
        use plotly::common::Fill;
        let fill = match fill.as_str().unwrap() {
            "tozeroy" => Fill::ToZeroY,
            "tozerox" => Fill::ToZeroX,
            "tonexty" => Fill::ToNextY,
            "tonextx" => Fill::ToNextX,
            "toself" => Fill::ToSelf,
            "tonext" => Fill::ToNext,
            "none" => Fill::None,
            unexpected => return Err(anyhow!("{unexpected} can't be fill")),
        };
        scatter.fill(fill)
    } else {
        scatter
    };
    let scatter = if let Some(mode) = scatter_obj.get_mut("mode")
        && mode.is_string() {
        // Safety: This `unwrap` will never be reached.
        use plotly::common::Mode;
        let mode = match mode.as_str().unwrap() {
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
        scatter.mode(mode)
    } else {
        scatter
    };
    Ok(scatter)
}
