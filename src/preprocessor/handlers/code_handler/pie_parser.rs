use crate::translate;
use plotly::Pie;
use serde_json::Value;
use std::error::Error;

pub fn parse_pie_data(pie_obj: &mut Value) -> Result<Box<Pie<u64>>, Box<dyn Error>> {
    let pie = pie_obj
        .get_mut("values")
        .ok_or(String::from("missing `values` field"))?;
    let pie = Pie::new(serde_json::from_value::<Vec<u64>>(pie.take())?);
    let pie = translate! {
        pie,
        pie_obj,
        (automargin, bool),
        (dlabel, f64),
        (hole, f64),
        (hover_template, String),
        (hover_template_array, Vec<String>),
        (hover_text, String),
        (hover_text_array, Vec<String>),
        (ids, Vec<String>),
        (label0, f64),
        (labels, Vec<String>),
        (legend_group, String),
        (legend_rank, usize),
        (name, String),
        (opacity, f64),
        (meta, String),
        (sort, bool),
        (text_position_src, String),
        (text_position_src_array, Vec<String>),
        (text, String),
        (text_array, Vec<String>),
        (text_info, String),
    }?;
    Ok(pie)
}
