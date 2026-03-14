use super::until::{Map, must_translate};
use crate::{translate, translate_enum};
use anyhow::Result;
use plotly::traces::table::{Cells, Header, Table};

pub fn parse_table_data(
    table_obj: &mut serde_json::Value,
    map: &Map,
) -> Result<Box<Table<Vec<String>, String>>> {
    let header_values: Vec<Vec<String>> = must_translate(table_obj, map, "header_values")?;
    let cells_values: Vec<Vec<String>> = must_translate(table_obj, map, "cells_values")?;
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

    use plotly::common::Visible;
    let table = translate_enum! {
        table,
        table_obj,
        map,
        (visible, {
            "true" =>       Visible::True,
            "false" =>      Visible::False,
            "legendonly" => Visible::LegendOnly,
        }),
    }?;

    Ok(table)
}
