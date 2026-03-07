use super::until::{Map, must_translate};
use crate::translate;
use anyhow::{Result, anyhow};
use plotly::traces::table::{Cells, Header, Table};

pub fn parse_table_data(
    table_obj: &mut serde_json::Value,
    map: &Map,
) -> Result<Box<Table<Vec<String>, String>>> {
    let header_values: Vec<Vec<String>> = must_translate(table_obj, "header_values")?;
    let cells_values: Vec<Vec<String>> = must_translate(table_obj, "cells_values")?;
    let header = Header::new(header_values);
    let cells = Cells::new(cells_values);
    let table = Table::new(header, cells);
    let table = translate! {
        table,
        table_obj,
        map,
        (name, String),
        (column_width, f64),
        (column_order, Vec<usize>),
    }?;
    let table = if let Some(visible) = table_obj.get_mut("visible")
        && visible.is_string()
    {
        use plotly::common::Visible;
        let visible = match visible.as_str().unwrap_or_else(|| unreachable!()) {
            "true" => Visible::True,
            "false" => Visible::False,
            "legendonly" => Visible::LegendOnly,
            unexpected => return Err(anyhow!("{unexpected} can't be visible")),
        };
        table.visible(visible)
    } else {
        table
    };
    Ok(table)
}
