use super::until::Map;
use crate::translate;
use anyhow::{Result, anyhow};
use plotly::common::color::Rgb;
use plotly::sankey::{Node, Sankey};

pub fn parse_sankey_data(
    sankey_obj: &mut serde_json::Value,
    map: &Map,
) -> Result<Box<Sankey<f64>>> {
    let sankey = Sankey::new();

    let sankey = if let Some(node_obj) = sankey_obj.get_mut("node")
        && node_obj.is_object()
    {
        let node = translate! {
            Node::new(),
            node_obj,
            map,
            (color, Rgb),
            (color_array, Vec<Rgb>),
            (hover_template, String),
            (pad, usize),
            (thickness, usize),
            (x, Vec<f64>),
            (y, Vec<f64>),
        }?;
        sankey.node(node)
    } else {
        sankey
    };

    let sankey = translate! {
        sankey,
        sankey_obj,
        map,
        (name, String),
        (visible, bool),
        (value_format, String),
        (value_suffix, String),
    }?;

    let sankey = if let Some(orientation) = sankey_obj.get_mut("orientation")
        && orientation.is_string()
    {
        use plotly::common::Orientation;
        let orientation = match orientation.as_str().unwrap_or_else(|| unreachable!()) {
            "v" => Orientation::Vertical,
            "h" => Orientation::Horizontal,
            unexpected => return Err(anyhow!("{unexpected} can't be orientation")),
        };
        sankey.orientation(orientation)
    } else {
        sankey
    };

    let sankey = if let Some(arrangement) = sankey_obj.get_mut("arrangement")
        && arrangement.is_string()
    {
        use plotly::sankey::Arrangement;
        let arrangement = match arrangement.as_str().unwrap_or_else(|| unreachable!()) {
            "snap" => Arrangement::Snap,
            "perpendicular" => Arrangement::Perpendicular,
            "freeform" => Arrangement::Freeform,
            "fixed" => Arrangement::Fixed,
            unexpected => return Err(anyhow!("{unexpected} can't be arrangement")),
        };
        sankey.arrangement(arrangement)
    } else {
        sankey
    };

    Ok(sankey)
}
