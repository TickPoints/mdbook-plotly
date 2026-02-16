use super::until::must_translate;
use crate::translate;
use anyhow::Result;
use plotly::Pie;

pub fn parse_pie_data(pie_obj: &mut serde_json::Value) -> Result<Box<Pie<usize>>> {
    let pie: Vec<usize> = must_translate(pie_obj, "values")?;
    let pie: Box<Pie<usize>> = Pie::new(pie);
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
