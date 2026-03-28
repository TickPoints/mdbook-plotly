use mdbook_plotly::preprocessor::handlers::code_handler;
use plotly::{Layout, Plot, Scatter};

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

#[test]
fn test_g_linear() {
    let raw_code = r#"
    {
        map: {
            myrange: {
                type: "g-linear",
                begin: 0.0,
                end: 1.0,
                count: 5
            }
        },
        data: [{
            type: "scatter",
            x: [0, 1, 2, 3, 4],
            y: "map.myrange"
        }]
    }
    "#;
    let generated_plot = code_handler::handle_json_input(raw_code.to_string()).unwrap();
    // Debug: print JSON representation
    println!("Generated: {}", serde_json::to_string(&generated_plot).unwrap());
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0, 2.0, 3.0, 4.0], vec![0.0, 0.25, 0.5, 0.75, 1.0]);
    reasonable_plot.add_trace(trace);
    println!("Reasonable: {}", serde_json::to_string(&reasonable_plot).unwrap());
    assert!(reasonable_plot == generated_plot);
}

#[test]
fn test_g_range() {
    let raw_code = r#"
    {
        map: {
            myrange: {
                type: "g-range",
                begin: 0.0,
                end: 5.0,
                step: 1.0
            }
        },
        data: [{
            type: "scatter",
            x: [0, 1, 2, 3, 4],
            y: "map.myrange"
        }]
    }
    "#;
    let generated_plot = code_handler::handle_json_input(raw_code.to_string()).unwrap();
    println!("Generated: {}", serde_json::to_string(&generated_plot).unwrap());
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0, 2.0, 3.0, 4.0], vec![0.0, 1.0, 2.0, 3.0, 4.0]);
    reasonable_plot.add_trace(trace);
    println!("Reasonable: {}", serde_json::to_string(&reasonable_plot).unwrap());
    assert!(reasonable_plot == generated_plot);
}

#[test]
fn test_g_repeat() {
    let raw_code = r#"
    {
        map: {
            myrange: {
                type: "g-repeat",
                value: 42.0,
                count: 3
            }
        },
        data: [{
            type: "scatter",
            x: [0, 1, 2],
            y: "map.myrange"
        }]
    }
    "#;
    let generated_plot = code_handler::handle_json_input(raw_code.to_string()).unwrap();
    println!("Generated: {}", serde_json::to_string(&generated_plot).unwrap());
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0, 2.0], vec![42.0, 42.0, 42.0]);
    reasonable_plot.add_trace(trace);
    println!("Reasonable: {}", serde_json::to_string(&reasonable_plot).unwrap());
    assert!(reasonable_plot == generated_plot);
}

#[test]
fn test_g_number() {
    let raw_code = r#"
    {
        map: {
            mynum: {
                type: "g-number",
                expr: "2 + 3"
            }
        },
        data: [{
            type: "scatter",
            x: [0, 1, 2],
            y: [0, 0, 0],
            opacity: "map.mynum"
        }]
    }
    "#;
    let generated_plot = code_handler::handle_json_input(raw_code.to_string()).unwrap();
    println!("Generated: {}", serde_json::to_string(&generated_plot).unwrap());
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0, 2.0], vec![0.0, 0.0, 0.0]).opacity(5.0);
    reasonable_plot.add_trace(trace);
    println!("Reasonable: {}", serde_json::to_string(&reasonable_plot).unwrap());
    assert!(reasonable_plot == generated_plot);
}

#[test]
fn test_g_number_list() {
    let raw_code = r#"
    {
        map: {
            mylist: {
                type: "g-number-list",
                begin: 0,
                end: 3,
                expr: "i * 2"
            }
        },
        data: [{
            type: "scatter",
            x: [0, 1, 2],
            y: "map.mylist"
        }]
    }
    "#;
    let generated_plot = code_handler::handle_json_input(raw_code.to_string()).unwrap();
    println!("Generated: {}", serde_json::to_string(&generated_plot).unwrap());
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0, 2.0], vec![0.0, 2.0, 4.0]);
    reasonable_plot.add_trace(trace);
    println!("Reasonable: {}", serde_json::to_string(&reasonable_plot).unwrap());
    assert!(reasonable_plot == generated_plot);
}

#[test]
fn test_raw() {
    let raw_code = r#"
    {
        map: {
            mydata: {
                type: "raw",
                data: [1, 2, 3]
            }
        },
        data: [{
            type: "scatter",
            x: [0, 1, 2],
            y: "map.mydata"
        }]
    }
    "#;
    let generated_plot = code_handler::handle_json_input(raw_code.to_string()).unwrap();
    println!("Generated: {}", serde_json::to_string(&generated_plot).unwrap());
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0, 2.0], vec![1.0, 2.0, 3.0]);
    reasonable_plot.add_trace(trace);
    println!("Reasonable: {}", serde_json::to_string(&reasonable_plot).unwrap());
    assert!(reasonable_plot == generated_plot);
}