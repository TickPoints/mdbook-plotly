use super::until::{Color, Map, must_translate};
use crate::{translate, translate_enum};
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

    use plotly::common::Orientation;
    use plotly::histogram::{HistFunc, HistNorm};

    let hist = translate_enum! {
        hist,
        hist_obj,
        map,
        (orientation, {
            "v" => Orientation::Vertical,
            "h" => Orientation::Horizontal,
        }),
        (hist_func, {
            "count" => HistFunc::Count,
            "sum"   => HistFunc::Sum,
            "avg"   => HistFunc::Average,
            "min"   => HistFunc::Minimum,
            "max"   => HistFunc::Maximum,
        }),
        (hist_norm, {
            "percent"             => HistNorm::Percent,
            "probability"         => HistNorm::Probability,
            "density"             => HistNorm::Density,
            "probability density" => HistNorm::ProbabilityDensity,
            ""                    => HistNorm::Default,
        }),
    }?;

    let hist = if let Some(marker_obj) = hist_obj.get_mut("marker")
        && marker_obj.is_object()
    {
        let marker = plotly::common::Marker::new();
        let marker = translate! {
            marker,
            marker_obj,
            map,
            (color, Color),
            (opacity, f64),
            (size, usize),
            (size_array, Vec<usize>),
            (max_displayed, usize),
            (size_ref, usize),
            (size_min, usize),
            (cauto, bool),
            (cmax, f64),
            (cmin, f64),
            (cmid, f64),
            (auto_color_scale, bool),
            (reverse_scale, bool),
            (show_scale, bool),
            (outlier_color, Color)
        }?;

        use plotly::common::{MarkerSymbol, SizeMode};
        let marker = translate_enum! {
            marker,
            marker_obj,
            map,
            (symbol, {
                "circle" =>         MarkerSymbol::Circle,
                "square" =>         MarkerSymbol::Square,
                "diamond" =>        MarkerSymbol::Diamond,
                "cross" =>          MarkerSymbol::Cross,
                "x" =>              MarkerSymbol::X,
                "triangle-up" =>    MarkerSymbol::TriangleUp,
                "triangle-down" =>  MarkerSymbol::TriangleDown,
                "triangle-left" =>  MarkerSymbol::TriangleLeft,
                "triangle-right" => MarkerSymbol::TriangleRight,
                "pentagon" =>       MarkerSymbol::Pentagon,
                "hexagon" =>        MarkerSymbol::Hexagon,
            }),
            (size_mode, {
                "area" => SizeMode::Area,
                "diameter" => SizeMode::Diameter,
            }),
        }?;
        hist.marker(marker)
    } else {
        hist
    };

    Ok(hist)
}
