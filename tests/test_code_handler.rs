use mdbook_plotly::preprocessor::handlers::code_handler;
use plotly::{Configuration, Layout, Plot};

#[test]
fn test_json5() {
    let raw_code = r#"
    {
        layout: {
            title: "Test",
        },
        config: {
            static_plot: true,
        }
    }
    "#;
    let generated_plot = code_handler::handle_json_input(raw_code.to_string()).unwrap();

    let mut reasonable_plot = Plot::new();
    let config = Configuration::new().static_plot(true);
    reasonable_plot.set_configuration(config);
    let layout = Layout::new().title("Test");
    reasonable_plot.set_layout(layout);

    assert!(reasonable_plot == generated_plot);
}
