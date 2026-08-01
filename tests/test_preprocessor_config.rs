//! Tests for the preprocessor configuration parsing.

use mdbook_plotly::preprocessor::config::PreprocessorConfig;
use toml::Value;

fn parse(text: &str) -> PreprocessorConfig {
    PreprocessorConfig::from_toml(&toml::from_str::<Value>(text).unwrap())
}

#[test]
fn ignores_unknown_top_level_keys() {
    let config = parse(
        r#"
        output-type = "plotly-html"
        input-type = "toml-input"
        unexpected = 1
        [map-eval]
        enabled = false
        extra = true
    "#,
    );
    assert_eq!(
        config.output_type,
        mdbook_plotly::preprocessor::config::PlotlyOutputType::PlotlyHtml
    );
    assert_eq!(
        config.input_type,
        mdbook_plotly::preprocessor::config::PlotlyInputType::TOMLInput
    );
    assert!(!config.map_eval.enabled);
}

#[test]
fn falls_back_for_only_the_bad_field() {
    let config = parse(
        r#"
        output-type = "plotly-html"
        input-type = 42
        [map-eval]
        enabled = false
        reuse-slab = false
        compile-expressions = false
        namespace-scope = "exports-only"
    "#,
    );
    assert_eq!(
        config.output_type,
        mdbook_plotly::preprocessor::config::PlotlyOutputType::PlotlyHtml
    );
    assert_eq!(
        config.input_type,
        mdbook_plotly::preprocessor::config::PlotlyInputType::JSONInput
    );
    assert_eq!(
        config.map_eval.namespace_scope,
        mdbook_plotly::preprocessor::config::MapNamespaceScope::ExportsOnly
    );
    assert!(!config.map_eval.enabled);
    assert!(!config.map_eval.reuse_slab);
    assert!(!config.map_eval.compile_expressions);
}

#[test]
fn falls_back_for_only_the_nested_bad_field() {
    let config = parse(
        r#"
        output-type = "plotly-html"
        input-type = "toml-input"
        [map-eval]
        enabled = "nope"
        reuse-slab = false
        compile-expressions = false
        namespace-scope = "exports-only"
    "#,
    );
    assert_eq!(
        config.output_type,
        mdbook_plotly::preprocessor::config::PlotlyOutputType::PlotlyHtml
    );
    assert_eq!(
        config.input_type,
        mdbook_plotly::preprocessor::config::PlotlyInputType::TOMLInput
    );
    assert!(config.map_eval.enabled);
    assert!(!config.map_eval.reuse_slab);
    assert!(!config.map_eval.compile_expressions);
    assert_eq!(
        config.map_eval.namespace_scope,
        mdbook_plotly::preprocessor::config::MapNamespaceScope::ExportsOnly
    );
}
