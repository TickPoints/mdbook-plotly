use super::until::must_translate_from_context;
use crate::code_handler::parse_context::ParseContext;
use crate::{translate_enum_with_config, translate_with_config};
use anyhow::Result;
use plotly::Trace;
use plotly::traces::table::{Cells, Header, Table};

pub fn parse_table_data(
    table_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<dyn Trace>> {
    let header_values: Vec<Vec<String>> =
        must_translate_from_context(table_obj, context, "header_values")?;
    let cells_values: Vec<Vec<String>> =
        must_translate_from_context(table_obj, context, "cells_values")?;
    let header = Header::new(header_values);
    let cells = Cells::new(cells_values);
    let table = Table::new(header, cells);
    let table = translate_with_config! {
        table,
        table_obj,
        context.map(),
        context.map_eval(),
        (name, String),
        (column_width, f64),
        (column_order, Vec<usize>),
    }?;

    use plotly::common::Visible;
    let table = translate_enum_with_config! {
        table,
        table_obj,
        context.map(),
        context.map_eval(),
        (visible, {
            "true" =>       Visible::True,
            "false" =>      Visible::False,
            "legendonly" => Visible::LegendOnly,
        }),
    }?;

    Ok(table)
}
