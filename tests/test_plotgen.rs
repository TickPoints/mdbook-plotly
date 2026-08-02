//! Tests for the plot generator form logic (no terminal needed).

#![cfg(feature = "tui")]

use mdbook_plotly::tui::locale::DocLang;
use mdbook_plotly::tui::plotgen::{GenStatus, OutputFormat, PlotGen};

/// Flat form index of a trace field (`schema.trace_fields` come first).
fn trace_idx(g: &PlotGen, name: &str) -> usize {
    g.trace_fields()
        .iter()
        .position(|f| f.name == name)
        .unwrap_or_else(|| panic!("trace field '{name}' not found"))
}

/// Flat form index of a global field (after the trace fields).
fn global_idx(g: &PlotGen, name: &str) -> usize {
    let n = g.trace_fields().len();
    n + g
        .global_fields
        .iter()
        .position(|f| f.name == name)
        .unwrap_or_else(|| panic!("global field '{name}' not found"))
}

#[test]
fn new_prefills_from_example() {
    let g = PlotGen::new(DocLang::English);
    assert_eq!(g.current_type().id, "line");
    assert_eq!(g.trace_inputs.len(), 1);
    assert!(!g.global_inputs.is_empty());
    assert!(!g.has_errors());
    assert!(g.generated.contains("\"type\": \"scatter\""));
    assert!(g.generated.contains("\"title\": \"Basic Line\""));
}

#[test]
fn chinese_schema_selects_localized_labels() {
    let g = PlotGen::new(DocLang::Chinese);
    assert_eq!(g.current_type().label, "折线图");
    assert!(g.generated.contains("基础折线图"));
}

#[test]
fn set_type_resets_the_form() {
    let mut g = PlotGen::new(DocLang::English);
    g.set_type(2);
    assert_eq!(g.current_type().id, "bar");
    assert!(g.generated.contains("\"type\": \"bar\""));
}

#[test]
fn cycle_output_toggles_format() {
    let mut g = PlotGen::new(DocLang::English);
    assert_eq!(g.output, OutputFormat::Json);
    g.cycle_output();
    assert_eq!(g.output, OutputFormat::Toml);
    assert!(g.generated.contains("[[data]]"), "{}", g.generated);
}

#[test]
fn editing_updates_preview_and_validation() {
    let mut g = PlotGen::new(DocLang::English);
    let xi = trace_idx(&g, "x");
    let yi = trace_idx(&g, "y");
    g.set_text_field(xi, "0,1,2".to_string());
    let value: serde_json::Value = serde_json::from_str(&g.generated).unwrap();
    assert_eq!(value["data"][0]["x"], serde_json::json!([0, 1, 2]));
    g.set_text_field(yi, String::new());
    assert!(g.has_errors());
}

#[test]
fn bool_and_enum_fields_work() {
    let mut g = PlotGen::new(DocLang::English);
    let legend = global_idx(&g, "show_legend");
    assert_eq!(g.field_display(legend), "[x]");
    g.toggle_bool(legend);
    assert_eq!(g.field_display(legend), "[ ]");
    assert!(g.generated.contains("\"showlegend\": false"));

    let color = trace_idx(&g, "color");
    g.cycle_enum(color, 1);
    assert_eq!(g.field_display(color), "Moss green");
    assert!(g.generated.contains("#3D5230"));
}

#[test]
fn add_and_remove_traces() {
    let mut g = PlotGen::new(DocLang::English);
    assert_eq!(g.trace_inputs.len(), 1);
    g.add_trace();
    assert_eq!(g.trace_inputs.len(), 2);
    assert_eq!(g.active_trace, 1);
    g.remove_trace();
    assert_eq!(g.trace_inputs.len(), 1);
    assert_eq!(g.active_trace, 0);
    g.remove_trace();
    assert_eq!(g.trace_inputs.len(), 1, "never drops below one trace");
}

#[test]
fn switch_trace_edits_different_traces() {
    let mut g = PlotGen::new(DocLang::English);
    let xi = trace_idx(&g, "x");
    let yi = trace_idx(&g, "y");
    g.add_trace();
    g.set_text_field(yi, "4,5,6".to_string());
    let value: serde_json::Value = serde_json::from_str(&g.generated).unwrap();
    assert_eq!(value["data"][1]["y"], serde_json::json!([4, 5, 6]));

    g.switch_trace(-1);
    assert_eq!(g.active_trace, 0);
    let value: serde_json::Value = serde_json::from_str(&g.generated).unwrap();
    assert_eq!(value["data"][0]["y"], serde_json::json!([1, 3, 2, 4]));

    g.set_text_field(xi, "0,1".to_string());
    let value: serde_json::Value = serde_json::from_str(&g.generated).unwrap();
    assert_eq!(value["data"][0]["x"], serde_json::json!([0, 1]));
    assert!(
        value["data"][1].get("x").is_none(),
        "new traces start empty, not copied from the example"
    );
}

#[test]
fn edit_mode_commits_and_cancels() {
    let mut g = PlotGen::new(DocLang::English);
    let ni = trace_idx(&g, "name");
    assert_eq!(g.field_display(ni), "");
    assert!(!g.is_editing());
    g.start_edit(ni);
    assert!(g.is_editing());
    for c in "Series A".chars() {
        g.insert_char(c);
    }
    g.commit_edit();
    assert!(!g.is_editing());
    let value: serde_json::Value = serde_json::from_str(&g.generated).unwrap();
    assert_eq!(value["data"][0]["name"], "Series A");

    let zi = trace_idx(&g, "z");
    g.start_edit(zi);
    g.insert_char('1');
    g.cancel_edit();
    assert!(!g.is_editing());
    let value: serde_json::Value = serde_json::from_str(&g.generated).unwrap();
    assert!(
        value["data"][0].get("z").is_none(),
        "cancelled edits must not reach the output"
    );
}

#[test]
fn edit_mode_handles_cursor_moves() {
    let mut g = PlotGen::new(DocLang::English);
    let ni = trace_idx(&g, "name");
    g.start_edit(ni);
    for c in "123".chars() {
        g.insert_char(c);
    }
    assert_eq!(g.edit_text, "123");
    assert_eq!(g.edit_cursor, 3);
    g.cursor_left();
    g.cursor_left();
    g.insert_char('9');
    assert_eq!(g.edit_text, "1923");
    g.cursor_home();
    g.backspace();
    assert_eq!(g.edit_text, "1923");
    g.cursor_right();
    g.delete();
    assert_eq!(g.edit_text, "123");
    g.cancel_edit();
}

#[test]
fn edit_mode_ignores_bool_and_enum_fields() {
    let mut g = PlotGen::new(DocLang::English);
    let legend = global_idx(&g, "show_legend");
    let color = trace_idx(&g, "color");
    g.start_edit(legend);
    assert!(!g.is_editing(), "bool fields are toggled, not edited");
    g.start_edit(color);
    assert!(!g.is_editing(), "enum fields are cycled, not edited");
}

#[test]
fn config_and_map_are_global_json_fields() {
    let mut g = PlotGen::new(DocLang::English);
    let config = global_idx(&g, "config");
    let map = global_idx(&g, "map");
    assert_eq!(g.field_display(config), "");
    g.set_text_field(config, "{ responsive: true }".to_string());
    assert_eq!(g.field_display(config), "{ … }");
    let value: serde_json::Value = serde_json::from_str(&g.generated).unwrap();
    assert_eq!(value["config"]["responsive"], true);
    assert!(value.get("map").is_none(), "empty map must be omitted");
    g.set_text_field(map, "{ n: 3 }".to_string());
    let value: serde_json::Value = serde_json::from_str(&g.generated).unwrap();
    assert_eq!(value["map"]["n"], 3);
}

#[test]
fn save_file_name_reflects_type_and_format() {
    let mut g = PlotGen::new(DocLang::English);
    assert_eq!(g.save_file_name(), "plot-line.json");
    g.set_type(4);
    assert_eq!(g.save_file_name(), "plot-histogram.json");
    g.cycle_output();
    assert_eq!(g.save_file_name(), "plot-histogram.toml");
}

#[test]
fn save_with_errors_is_refused() {
    let mut g = PlotGen::new(DocLang::English);
    let xi = trace_idx(&g, "x");
    g.set_text_field(xi, String::new());
    assert!(g.has_errors());
    g.save();
    assert!(matches!(g.status, Some(GenStatus::Error(_))));
}

#[test]
fn reset_restores_the_example() {
    let mut g = PlotGen::new(DocLang::English);
    let xi = trace_idx(&g, "x");
    g.set_text_field(xi, "9,9".to_string());
    let value: serde_json::Value = serde_json::from_str(&g.generated).unwrap();
    assert_eq!(value["data"][0]["x"], serde_json::json!([9, 9]));
    g.reset_to_example();
    let value: serde_json::Value = serde_json::from_str(&g.generated).unwrap();
    assert_eq!(value["data"][0]["x"], serde_json::json!([0, 1, 2, 3]));
}

#[test]
fn no_preview_defers_generation_until_regen() {
    let mut g = PlotGen::new(DocLang::English);
    g.set_no_preview(true);
    let xi = trace_idx(&g, "x");
    let before = g.generated.clone();
    g.set_text_field(xi, "9,9".to_string());
    assert_eq!(
        g.generated, before,
        "with --no-preview, editing must not regenerate"
    );
    g.regen();
    let value: serde_json::Value = serde_json::from_str(&g.generated).unwrap();
    assert_eq!(value["data"][0]["x"], serde_json::json!([9, 9]));
}
