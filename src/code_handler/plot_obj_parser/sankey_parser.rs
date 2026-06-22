use super::until::Color;
use crate::code_handler::parse_context::ParseContext;
use crate::{translate_enum_with_config, translate_with_config};
use anyhow::Result;
use plotly::{Trace, sankey::{Node, Sankey}};

pub fn parse_sankey_data(
    sankey_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<Sankey<f64>>> {
    let sankey = Sankey::new();

    let sankey = if let Some(node_obj) = sankey_obj.get_mut("node")
        && node_obj.is_object()
    {
        let node = translate_with_config! {
            Node::new(),
            node_obj,
            context.map(),
            context.map_eval(),
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

    let sankey = translate_with_config! {
        sankey,
        sankey_obj,
        context.map(),
        context.map_eval(),
        (name, String),
        (visible, bool),
        (value_format, String),
        (value_suffix, String),
    }?;

    use plotly::common::Orientation;
    use plotly::sankey::Arrangement;
    let sankey = translate_enum_with_config! {
        sankey,
        sankey_obj,
        context.map(),
        context.map_eval(),
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

pub fn parse_sankey_trace(
    sankey_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<dyn Trace>> {
    Ok(parse_sankey_data(sankey_obj, context)?)
}
