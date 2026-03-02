use mdbook_plotly::preprocessor::handlers::code_handler;
use plotly::{Layout, Plot};

#[test]
fn test_json5() {
    let raw_code = r#"
    {
        map: {
            title: "Test"
        },
        layout: {
            title: "map.title"
        }
    }
    "#;
    let generated_plot = code_handler::handle_json_input(raw_code.to_string()).unwrap();

    let mut reasonable_plot = Plot::new();
    let layout = Layout::new().title("Test");
    reasonable_plot.set_layout(layout);

    assert!(reasonable_plot == generated_plot);
}
