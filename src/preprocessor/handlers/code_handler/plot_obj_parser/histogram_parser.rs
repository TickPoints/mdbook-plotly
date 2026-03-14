use super::until::{Map, must_translate};
use crate::translate;
use anyhow::{Result, anyhow};
use plotly::Histogram;

pub fn parse_histogram_data(
    hist_obj: &mut serde_json::Value,
    map: &Map,
) -> Result<Box<Histogram<f64>>> {
    let has_x = hist_obj.get("x").is_some();
    let has_y = hist_obj.get("y").is_some();
    let hist = match (has_x, has_y) {
        (true, true) => {
            let x: Vec<f64> = must_translate(hist_obj, map, "x")?;
            let y: Vec<f64> = must_translate(hist_obj, map, "y")?;
            Histogram::new_xy(x, y)
        }
        (true, false) => {
            let x: Vec<f64> = must_translate(hist_obj, map, "x")?;
            Histogram::new(x)
        }
        (false, true) => {
            let y: Vec<f64> = must_translate(hist_obj, map, "y")?;
            Histogram::new_vertical(y)
        }
        (false, false) => {
            return Err(anyhow!("histogram requires at least 'x' or 'y' data"));
        }
    };
    let hist = translate! {
        hist,
        hist_obj,
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
        (auto_bin_x, bool),
        (n_bins_x, usize),
        (auto_bin_y, bool),
        (n_bins_y, usize),
        (alignment_group, String),
        (offset_group, String),
        (bin_group, String),
        (x_axis, String),
        (y_axis, String),
    }?;
    let hist = if let Some(orientation) = hist_obj.get_mut("orientation")
        && orientation.is_string()
    {
        use plotly::common::Orientation;
        let orientation = match orientation.as_str().unwrap_or_else(|| unreachable!()) {
            "v" => Orientation::Vertical,
            "h" => Orientation::Horizontal,
            unexpected => return Err(anyhow!("{unexpected} can't be orientation")),
        };
        hist.orientation(orientation)
    } else {
        hist
    };
    let hist = if let Some(hf) = hist_obj.get_mut("hist_func")
        && hf.is_string()
    {
        use plotly::histogram::HistFunc;
        let hf = match hf.as_str().unwrap_or_else(|| unreachable!()) {
            "count" => HistFunc::Count,
            "sum" => HistFunc::Sum,
            "avg" => HistFunc::Average,
            "min" => HistFunc::Minimum,
            "max" => HistFunc::Maximum,
            unexpected => return Err(anyhow!("{unexpected} can't be hist_func")),
        };
        hist.hist_func(hf)
    } else {
        hist
    };
    let hist = if let Some(hn) = hist_obj.get_mut("hist_norm")
        && hn.is_string()
    {
        use plotly::histogram::HistNorm;
        let hn = match hn.as_str().unwrap_or_else(|| unreachable!()) {
            "percent" => HistNorm::Percent,
            "probability" => HistNorm::Probability,
            "density" => HistNorm::Density,
            "probability density" => HistNorm::ProbabilityDensity,
            "" => HistNorm::Default,
            unexpected => return Err(anyhow!("{unexpected} can't be hist_norm")),
        };
        hist.hist_norm(hn)
    } else {
        hist
    };
    Ok(hist)
}
