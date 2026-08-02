//! Enforces the plot generator schema (`docs/PLOT-SCHEMA.json` and its
//! `-zh_CN` translation) against the parser: both languages must stay in
//! sync, every example must generate a valid plot configuration, the form
//! validation must catch bad input, and multiple traces plus `config` /
//! `map` must round-trip.

#![cfg(feature = "tui")]

use mdbook_plotly::plot_schema::{
    EMBEDDED_EN, EMBEDDED_ZH_CN, PlotSchema, PlotTypeSchema, build_config, composite_globals,
    config_to_json, config_to_toml, default_input, has_errors, prefill,
};

fn en() -> PlotSchema {
    PlotSchema::parse(EMBEDDED_EN).unwrap()
}

fn zh() -> PlotSchema {
    PlotSchema::parse(EMBEDDED_ZH_CN).unwrap()
}

fn trace_idx(schema: &PlotSchema, name: &str) -> usize {
    schema
        .trace_fields
        .iter()
        .position(|f| f.name == name)
        .unwrap_or_else(|| panic!("trace field '{name}' not found"))
}

fn global_idx(schema: &PlotSchema, plot_type: &PlotTypeSchema, name: &str) -> usize {
    composite_globals(schema, plot_type)
        .iter()
        .position(|f| f.name == name)
        .unwrap_or_else(|| panic!("global field '{name}' not found in '{}'", plot_type.id))
}

fn any_errors(trace_errors: &[Vec<Option<String>>], global_errors: &[Option<String>]) -> bool {
    trace_errors.iter().any(|t| has_errors(t)) || has_errors(global_errors)
}

#[test]
fn embedded_schema_has_all_plot_types() {
    let schema = en();
    assert_eq!(schema.schema, "2.0");
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
fn trace_and_global_fields_are_wellformed() {
    let schema = en();
    assert!(!schema.trace_fields.is_empty(), "no shared trace fields");
    let mut names = std::collections::BTreeSet::new();
    for field in &schema.trace_fields {
        assert!(!field.name.is_empty(), "empty trace field name");
        assert!(!field.label.is_empty(), "'{}' has no label", field.name);
        assert!(
            names.insert(field.name.as_str()),
            "duplicate trace field '{}'",
            field.name
        );
        assert!(
            !field.targets().is_empty(),
            "'{}' has no target",
            field.name
        );
    }
    assert!(
        schema.global_fields.iter().any(|f| f.name == "config"),
        "config global field missing"
    );
    assert!(
        schema.global_fields.iter().any(|f| f.name == "map"),
        "map global field missing"
    );
}

#[test]
fn plot_types_are_wellformed() {
    let schema = en();
    for plot_type in &schema.plot_types {
        assert!(
            !plot_type.label.is_empty(),
            "{}: label missing",
            plot_type.id
        );
        assert!(
            plot_type.trace_defaults.is_some(),
            "{}: trace_defaults missing",
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
        for required in &plot_type.required_data {
            assert!(
                schema.trace_fields.iter().any(|f| &f.name == required),
                "{}: required_data '{required}' is not a trace field",
                plot_type.id
            );
        }
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

    assert_eq!(en.trace_fields.len(), zh.trace_fields.len());
    for (e, z) in en.trace_fields.iter().zip(&zh.trace_fields) {
        assert_eq!(e.name, z.name, "trace field names must stay in sync");
        assert_eq!(e.kind, z.kind, "'{}': kinds", e.name);
        assert_eq!(e.path, z.path, "'{}': paths", e.name);
        assert_eq!(e.item_type, z.item_type, "'{}': item types", e.name);
        assert!(!z.label.is_empty(), "'{}': Chinese label missing", e.name);
    }

    assert_eq!(en.global_fields.len(), zh.global_fields.len());
    for (e, z) in en.global_fields.iter().zip(&zh.global_fields) {
        assert_eq!(e.name, z.name, "global field names must stay in sync");
        assert_eq!(e.kind, z.kind, "'{}': kinds", e.name);
        assert_eq!(e.path, z.path, "'{}': paths", e.name);
    }

    for (e, z) in en.plot_types.iter().zip(&zh.plot_types) {
        assert_eq!(e.id, z.id);
        assert_eq!(
            e.trace_defaults, z.trace_defaults,
            "{}: trace_defaults",
            e.id
        );
        assert_eq!(e.required_data, z.required_data, "{}: required_data", e.id);
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
            let (traces, globals) = prefill(&schema, plot_type);
            let (value, trace_errors, global_errors) =
                build_config(&schema, plot_type, &traces, &globals);
            let all_errors = trace_errors
                .iter()
                .flatten()
                .flatten()
                .chain(global_errors.iter().flatten())
                .collect::<Vec<_>>();
            assert!(
                !any_errors(&trace_errors, &global_errors),
                "{}: {all_errors:?}",
                plot_type.id
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
    let (traces, globals) = prefill(&schema, line);
    let (value, trace_errors, global_errors) = build_config(&schema, line, &traces, &globals);
    assert!(!any_errors(&trace_errors, &global_errors));
    assert_eq!(value["data"].as_array().unwrap().len(), 1);
    assert_eq!(value["data"][0]["type"], "scatter");
    assert_eq!(value["data"][0]["mode"], "lines");
    assert_eq!(value["data"][0]["x"], serde_json::json!([0, 1, 2, 3]));
    assert_eq!(value["layout"]["title"], "Basic Line");
}

#[test]
fn missing_required_field_is_an_error() {
    let schema = en();
    let line = schema.find("line").unwrap();
    let traces = vec![
        schema
            .trace_fields
            .iter()
            .map(default_input)
            .collect::<Vec<_>>(),
    ];
    let globals = line.fields.iter().map(default_input).collect::<Vec<_>>();
    let (_, trace_errors, global_errors) = build_config(&schema, line, &traces, &globals);
    assert!(any_errors(&trace_errors, &global_errors));
    let xi = trace_idx(&schema, "x");
    assert_eq!(trace_errors[0][xi].as_deref(), Some("required"));
    let yi = trace_idx(&schema, "y");
    assert_eq!(trace_errors[0][yi].as_deref(), Some("required"));
}

#[test]
fn number_field_validates_input() {
    let schema = en();
    let scatter = schema.find("scatter").unwrap();
    let si = trace_idx(&schema, "marker_size");
    let (mut traces, globals) = prefill(&schema, scatter);
    traces[0][si].text = "abc".into();
    let (_, trace_errors, _) = build_config(&schema, scatter, &traces, &globals);
    assert!(
        trace_errors[0][si]
            .as_ref()
            .unwrap()
            .contains("not a number")
    );
    traces[0][si].text = "99".into();
    let (_, trace_errors, _) = build_config(&schema, scatter, &traces, &globals);
    assert!(trace_errors[0][si].as_ref().unwrap().contains("at most 50"));
    traces[0][si].text = "12".into();
    let (_, trace_errors, global_errors) = build_config(&schema, scatter, &traces, &globals);
    assert!(trace_errors[0][si].is_none());
    assert!(trace_errors[0].iter().all(Option::is_none));
    assert!(global_errors.iter().all(Option::is_none));
}

#[test]
fn heatmap_array2d_and_toml_serialization() {
    let schema = en();
    let heatmap = schema.find("heatmap").unwrap();
    let zi = trace_idx(&schema, "z");
    let (mut traces, globals) = prefill(&schema, heatmap);
    traces[0][zi].text = "1,20,30;20,1,60;30,60,1".into();
    let (value, trace_errors, global_errors) = build_config(&schema, heatmap, &traces, &globals);
    assert!(!any_errors(&trace_errors, &global_errors));
    assert_eq!(value["data"][0]["z"].as_array().unwrap().len(), 3);
    let toml = config_to_toml(&value).unwrap();
    assert!(
        toml.contains("[[data]]"),
        "expected array-of-tables: {toml}"
    );
}

#[test]
fn multiple_traces_are_emitted() {
    let schema = en();
    let line = schema.find("line").unwrap();
    let xi = trace_idx(&schema, "x");
    let yi = trace_idx(&schema, "y");
    let (mut traces, globals) = prefill(&schema, line);
    traces[0][xi].text = "0,1,2".into();
    traces[0][yi].text = "1,2,3".into();
    traces.push(
        schema
            .trace_fields
            .iter()
            .map(default_input)
            .collect::<Vec<_>>(),
    );
    traces[1][xi].text = "0,1,2".into();
    traces[1][yi].text = "4,5,6".into();
    let (value, trace_errors, global_errors) = build_config(&schema, line, &traces, &globals);
    assert!(!any_errors(&trace_errors, &global_errors));
    assert_eq!(value["data"].as_array().unwrap().len(), 2);
    assert_eq!(value["data"][0]["y"], serde_json::json!([1, 2, 3]));
    assert_eq!(value["data"][1]["y"], serde_json::json!([4, 5, 6]));
}

#[test]
fn config_and_map_are_emitted() {
    let schema = en();
    let line = schema.find("line").unwrap();
    let (traces, mut globals) = prefill(&schema, line);
    let config_idx = global_idx(&schema, line, "config");
    let map_idx = global_idx(&schema, line, "map");
    globals[config_idx].text = "{ responsive: true, scrollZoom: false }".into();
    globals[map_idx].text = "{ n: { type: 'g-choose', options: [1, 2, 3] } }".into();
    let (value, trace_errors, global_errors) = build_config(&schema, line, &traces, &globals);
    assert!(!any_errors(&trace_errors, &global_errors));
    assert_eq!(value["config"]["responsive"], true);
    assert_eq!(value["config"]["scrollZoom"], false);
    assert_eq!(value["map"]["n"]["type"], "g-choose");
    assert_eq!(value["data"].as_array().unwrap().len(), 1);
}

#[test]
fn empty_config_and_map_are_omitted() {
    let schema = en();
    let line = schema.find("line").unwrap();
    let (traces, globals) = prefill(&schema, line);
    let (value, trace_errors, global_errors) = build_config(&schema, line, &traces, &globals);
    assert!(!any_errors(&trace_errors, &global_errors));
    assert!(
        value.get("config").is_none(),
        "empty config must be omitted"
    );
    assert!(value.get("map").is_none(), "empty map must be omitted");
}

#[test]
fn json_field_validates() {
    let schema = en();
    let line = schema.find("line").unwrap();
    let config_idx = global_idx(&schema, line, "config");
    let (traces, mut globals) = prefill(&schema, line);

    globals[config_idx].text = "{ broken".into();
    let (_, _, global_errors) = build_config(&schema, line, &traces, &globals);
    assert!(
        global_errors[config_idx]
            .as_ref()
            .unwrap()
            .contains("invalid JSON")
    );

    globals[config_idx].text = "[1, 2, 3]".into();
    let (_, _, global_errors) = build_config(&schema, line, &traces, &globals);
    assert!(
        global_errors[config_idx]
            .as_ref()
            .unwrap()
            .contains("JSON object")
    );

    globals[config_idx].text = "{ responsive: true }".into();
    let (_, _, global_errors) = build_config(&schema, line, &traces, &globals);
    assert!(global_errors[config_idx].is_none());
}

#[test]
fn dual_axis_example_prefills_two_traces() {
    let schema = en();
    let dual = schema.find("dual_axis").unwrap();
    let (traces, globals) = prefill(&schema, dual);
    assert_eq!(traces.len(), 2, "dual-axis example must prefill two traces");
    let (value, trace_errors, global_errors) = build_config(&schema, dual, &traces, &globals);
    assert!(!any_errors(&trace_errors, &global_errors));
    assert_eq!(value["data"][0]["type"], "bar");
    assert_eq!(value["data"][1]["type"], "scatter");
    assert_eq!(value["data"][1]["yaxis"], "y2");
    assert_eq!(value["layout"]["yaxis2"]["side"], "right");
    assert_eq!(value["data"][0]["x"], value["data"][1]["x"]);
}

#[test]
fn prefill_fills_from_example() {
    let schema = en();
    let line = schema.find("line").unwrap();
    let (traces, globals) = prefill(&schema, line);
    let ti = global_idx(&schema, line, "title");
    let xi = trace_idx(&schema, "x");
    assert_eq!(globals[ti].text, "Basic Line");
    assert_eq!(traces[0][xi].text, "0, 1, 2, 3");
    assert_eq!(traces.len(), 1);
}
