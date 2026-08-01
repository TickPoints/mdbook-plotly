//! Enforces the contract C schema (`docs/PLOT-SCHEMA.json` and its
//! `-zh_CN` translation) against the parser: both languages must stay in
//! sync, every example must generate a valid plot configuration, and the
//! form validation must catch bad input.

use mdbook_plotly::plot_schema::{
    EMBEDDED_EN, EMBEDDED_ZH_CN, PlotSchema, PlotTypeSchema, build_config, config_to_json,
    config_to_toml, default_input, has_errors, prefill_inputs,
};

fn en() -> PlotSchema {
    PlotSchema::parse(EMBEDDED_EN).unwrap()
}

fn zh() -> PlotSchema {
    PlotSchema::parse(EMBEDDED_ZH_CN).unwrap()
}

fn field_idx(plot_type: &PlotTypeSchema, name: &str) -> usize {
    plot_type
        .fields
        .iter()
        .position(|f| f.name == name)
        .unwrap_or_else(|| panic!("field '{name}' not found in '{}'", plot_type.id))
}

#[test]
fn embedded_schema_has_all_plot_types() {
    let schema = en();
    assert_eq!(schema.schema, "1.0");
    let ids: Vec<&str> = schema.plot_types.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "line",
            "scatter",
            "bar",
            "pie",
            "histogram",
            "box",
            "heatmap",
            "dual_axis"
        ]
    );
}

#[test]
fn plot_types_are_wellformed() {
    for plot_type in &en().plot_types {
        assert!(
            !plot_type.label.is_empty(),
            "{}: label missing",
            plot_type.id
        );
        assert!(
            !plot_type.traces.is_empty(),
            "{}: traces missing",
            plot_type.id
        );
        assert!(
            plot_type.example.is_some(),
            "{}: example missing",
            plot_type.id
        );
        assert!(
            !plot_type.fields.is_empty(),
            "{}: fields missing",
            plot_type.id
        );
        let mut names = std::collections::BTreeSet::new();
        for field in &plot_type.fields {
            assert!(!field.name.is_empty(), "{}: empty field name", plot_type.id);
            assert!(
                !field.label.is_empty(),
                "{}: field '{}' has no label",
                plot_type.id,
                field.name
            );
            assert!(
                names.insert(field.name.as_str()),
                "{}: duplicate field '{}'",
                plot_type.id,
                field.name
            );
            assert!(
                !field.targets().is_empty(),
                "{}: field '{}' has no target path",
                plot_type.id,
                field.name
            );
        }
    }
}

#[test]
fn chinese_schema_stays_in_sync_with_english() {
    let en = en();
    let zh = zh();
    assert_eq!(en.schema, zh.schema);
    assert_eq!(en.plot_types.len(), zh.plot_types.len());
    for (e, z) in en.plot_types.iter().zip(&zh.plot_types) {
        assert_eq!(e.id, z.id);
        assert_eq!(e.traces, z.traces, "{}: traces must match", e.id);
        assert_eq!(e.fields.len(), z.fields.len(), "{}: field count", e.id);
        assert!(!z.label.is_empty(), "{}: Chinese label missing", z.id);
        for (ef, zf) in e.fields.iter().zip(&z.fields) {
            assert_eq!(ef.name, zf.name, "{}: field names must stay in sync", e.id);
            assert_eq!(ef.kind, zf.kind, "{}: field kinds", e.id);
            assert_eq!(ef.path, zf.path, "{}: field paths", e.id);
            assert_eq!(ef.required, zf.required, "{}: field required", e.id);
        }
    }
}

#[test]
fn every_embedded_example_generates_valid_plot_input() {
    let config = mdbook_plotly::preprocessor::config::MapEvalConfig::default();
    for schema in [en(), zh()] {
        for plot_type in &schema.plot_types {
            let inputs = prefill_inputs(plot_type);
            let (value, errors) = build_config(plot_type, &inputs);
            assert!(
                !has_errors(&errors),
                "{}: {:?}",
                plot_type.id,
                errors.iter().flatten().collect::<Vec<_>>()
            );
            let json = config_to_json(&value);
            assert!(!json.is_empty(), "{}: empty output", plot_type.id);
            mdbook_plotly::code_handler::handle_json_input(json, &config)
                .unwrap_or_else(|e| panic!("{} generated invalid plot input: {e}", plot_type.id));
        }
    }
}

#[test]
fn builds_a_line_config() {
    let schema = en();
    let line = schema.find("line").unwrap();
    let mut inputs = prefill_inputs(line);
    let xi = field_idx(line, "x");
    let yi = field_idx(line, "y");
    inputs[xi].text = "0,1,2,3".into();
    inputs[yi].text = "1,3,2,4".into();
    let (value, errors) = build_config(line, &inputs);
    assert!(!has_errors(&errors), "{errors:?}");
    assert_eq!(value["data"][0]["type"], "scatter");
    assert_eq!(value["data"][0]["mode"], "lines");
    assert_eq!(value["data"][0]["x"], serde_json::json!([0, 1, 2, 3]));
    assert_eq!(value["layout"]["title"], "Basic Line");
}

#[test]
fn missing_required_field_is_an_error() {
    let schema = en();
    let line = schema.find("line").unwrap();
    let inputs = line.fields.iter().map(default_input).collect::<Vec<_>>();
    let (_, errors) = build_config(line, &inputs);
    assert!(has_errors(&errors));
    let xi = field_idx(line, "x");
    assert_eq!(errors[xi].as_deref(), Some("required"));
}

#[test]
fn number_field_validates_input() {
    let schema = en();
    let scatter = schema.find("scatter").unwrap();
    let si = field_idx(scatter, "marker_size");
    let mut inputs = prefill_inputs(scatter);
    inputs[si].text = "abc".into();
    let (_, errors) = build_config(scatter, &inputs);
    assert!(errors[si].as_ref().unwrap().contains("not a number"));
    inputs[si].text = "99".into();
    let (_, errors) = build_config(scatter, &inputs);
    assert!(errors[si].as_ref().unwrap().contains("at most 50"));
    inputs[si].text = "12".into();
    let (_, errors) = build_config(scatter, &inputs);
    assert!(errors[si].is_none());
}

#[test]
fn heatmap_array2d_and_toml_serialization() {
    let schema = en();
    let heatmap = schema.find("heatmap").unwrap();
    let zi = field_idx(heatmap, "z");
    let mut inputs = prefill_inputs(heatmap);
    inputs[zi].text = "1,20,30;20,1,60;30,60,1".into();
    let (value, errors) = build_config(heatmap, &inputs);
    assert!(!has_errors(&errors), "{errors:?}");
    assert_eq!(value["data"][0]["z"].as_array().unwrap().len(), 3);
    let toml = config_to_toml(&value).unwrap();
    assert!(
        toml.contains("[[data]]"),
        "expected array-of-tables: {toml}"
    );
}

#[test]
fn dual_axis_shares_x_between_traces() {
    let schema = en();
    let dual = schema.find("dual_axis").unwrap();
    let mut inputs = prefill_inputs(dual);
    inputs[field_idx(dual, "x")].text = "1,2,3,4".into();
    inputs[field_idx(dual, "y_bar")].text = "10,15,13,17".into();
    inputs[field_idx(dual, "y_line")].text = "0.4,0.6,0.55,0.8".into();
    let (value, errors) = build_config(dual, &inputs);
    assert!(!has_errors(&errors), "{errors:?}");
    assert_eq!(value["data"][0]["x"], value["data"][1]["x"]);
    assert_eq!(value["data"][1]["type"], "scatter");
    assert_eq!(value["data"][1]["yaxis"], "y2");
    assert_eq!(value["layout"]["yaxis2"]["side"], "right");
}

#[test]
fn prefill_fills_from_example() {
    let schema = en();
    let line = schema.find("line").unwrap();
    let inputs = prefill_inputs(line);
    let ti = field_idx(line, "title");
    let xi = field_idx(line, "x");
    assert_eq!(inputs[ti].text, "Basic Line");
    assert_eq!(inputs[xi].text, "0, 1, 2, 3");
}
