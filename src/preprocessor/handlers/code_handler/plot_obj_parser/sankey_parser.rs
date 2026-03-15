use super::until::{Color, Map};
use crate::{translate, translate_enum};
use anyhow::Result;
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
            (color, Color),
            (color_array, Vec<Color>),
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

    use plotly::common::Orientation;
    use plotly::sankey::Arrangement;
    let sankey = translate_enum! {
        sankey,
        sankey_obj,
        map,
        (orientation, {
            "v" => Orientation::Vertical,
            "h" => Orientation::Horizontal,
        }),
        (arrangement, {
            "snap" =>           Arrangement::Snap,
            "perpendicular" =>  Arrangement::Perpendicular,
            "freeform" =>       Arrangement::Freeform,
            "fixed" =>          Arrangement::Fixed,
        }),
    }?;

    Ok(sankey)
}
