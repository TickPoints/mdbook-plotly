//! Tests for the plot generator form logic (no terminal needed).

#![cfg(feature = "tui")]

use mdbook_plotly::tui::locale::DocLang;
use mdbook_plotly::tui::plotgen::{GenStatus, OutputFormat, PlotGen};

fn field_idx(g: &PlotGen, name: &str) -> usize {
    g.current_type()
        .fields
        .iter()
        .position(|f| f.name == name)
        .unwrap()
}

#[test]
fn new_prefills_from_example() {
    let g = PlotGen::new(DocLang::English);
    assert_eq!(g.current_type().id, "line");
    assert!(!g.inputs.is_empty());
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
    let xi = field_idx(&g, "x");
    let yi = field_idx(&g, "y");
    g.set_text_field(xi, "0,1,2".to_string());
    let value: serde_json::Value = serde_json::from_str(&g.generated).unwrap();
    assert_eq!(value["data"][0]["x"], serde_json::json!([0, 1, 2]));
    g.set_text_field(yi, String::new());
    assert!(g.has_errors());
}

#[test]
fn bool_and_enum_fields_work() {
    let mut g = PlotGen::new(DocLang::English);
    let legend = field_idx(&g, "show_legend");
    assert_eq!(g.field_display(legend), "[x]");
    g.toggle_bool(legend);
    assert_eq!(g.field_display(legend), "[ ]");
    assert!(g.generated.contains("\"showlegend\": false"));

    let color = field_idx(&g, "color");
    g.cycle_enum(color, 1);
    assert_eq!(g.field_display(color), "Moss green");
    assert!(g.generated.contains("#3D5230"));
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
    let xi = field_idx(&g, "x");
    g.set_text_field(xi, String::new());
    assert!(g.has_errors());
    g.save();
    assert!(matches!(g.status, Some(GenStatus::Error(_))));
}

#[test]
fn reset_restores_the_example() {
    let mut g = PlotGen::new(DocLang::English);
    let xi = field_idx(&g, "x");
    g.set_text_field(xi, "9,9".to_string());
    let value: serde_json::Value = serde_json::from_str(&g.generated).unwrap();
    assert_eq!(value["data"][0]["x"], serde_json::json!([9, 9]));
    g.reset_to_example();
    let value: serde_json::Value = serde_json::from_str(&g.generated).unwrap();
    assert_eq!(value["data"][0]["x"], serde_json::json!([0, 1, 2, 3]));
}
