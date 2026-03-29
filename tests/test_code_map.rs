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
    println!(
        "Generated: {}",
        serde_json::to_string(&generated_plot).unwrap()
    );
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(
        vec![0.0, 1.0, 2.0, 3.0, 4.0],
        vec![0.0, 0.25, 0.5, 0.75, 1.0],
    );
    reasonable_plot.add_trace(trace);
    println!(
        "Reasonable: {}",
        serde_json::to_string(&reasonable_plot).unwrap()
    );
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
    println!(
        "Generated: {}",
        serde_json::to_string(&generated_plot).unwrap()
    );
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0, 2.0, 3.0, 4.0], vec![0.0, 1.0, 2.0, 3.0, 4.0]);
    reasonable_plot.add_trace(trace);
    println!(
        "Reasonable: {}",
        serde_json::to_string(&reasonable_plot).unwrap()
    );
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
    println!(
        "Generated: {}",
        serde_json::to_string(&generated_plot).unwrap()
    );
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0, 2.0], vec![42.0, 42.0, 42.0]);
    reasonable_plot.add_trace(trace);
    println!(
        "Reasonable: {}",
        serde_json::to_string(&reasonable_plot).unwrap()
    );
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
    println!(
        "Generated: {}",
        serde_json::to_string(&generated_plot).unwrap()
    );
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0, 2.0], vec![0.0, 0.0, 0.0]).opacity(5.0);
    reasonable_plot.add_trace(trace);
    println!(
        "Reasonable: {}",
        serde_json::to_string(&reasonable_plot).unwrap()
    );
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
    println!(
        "Generated: {}",
        serde_json::to_string(&generated_plot).unwrap()
    );
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0, 2.0], vec![0.0, 2.0, 4.0]);
    reasonable_plot.add_trace(trace);
    println!(
        "Reasonable: {}",
        serde_json::to_string(&reasonable_plot).unwrap()
    );
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
    println!(
        "Generated: {}",
        serde_json::to_string(&generated_plot).unwrap()
    );
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0, 2.0], vec![1.0, 2.0, 3.0]);
    reasonable_plot.add_trace(trace);
    println!(
        "Reasonable: {}",
        serde_json::to_string(&reasonable_plot).unwrap()
    );
    assert!(reasonable_plot == generated_plot);
}

#[test]
fn test_boolean_type() {
    let raw_code = r#"
    {
        map: {
            mybool: {
                type: "raw",
                data: true
            }
        },
        data: [{
            type: "scatter",
            x: [0, 1, 2],
            y: [0, 0, 0],
            show_legend: "map.mybool"
        }]
    }
    "#;
    let generated_plot = code_handler::handle_json_input(raw_code.to_string()).unwrap();
    println!(
        "Generated boolean: {}",
        serde_json::to_string(&generated_plot).unwrap()
    );
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0, 2.0], vec![0.0, 0.0, 0.0]).show_legend(true);
    reasonable_plot.add_trace(trace);
    println!(
        "Reasonable: {}",
        serde_json::to_string(&reasonable_plot).unwrap()
    );
    assert!(reasonable_plot == generated_plot);
}

#[test]
fn test_integer_type() {
    let raw_code = r#"
    {
        map: {
            myint: {
                type: "raw",
                data: 42
            }
        },
        data: [{
            type: "scatter",
            x: [0, 1, 2],
            y: [0, 0, 0],
            opacity: "map.myint"
        }]
    }
    "#;
    let generated_plot = code_handler::handle_json_input(raw_code.to_string()).unwrap();
    println!(
        "Generated integer: {}",
        serde_json::to_string(&generated_plot).unwrap()
    );
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0, 2.0], vec![0.0, 0.0, 0.0]).opacity(42.0);
    reasonable_plot.add_trace(trace);
    println!(
        "Reasonable: {}",
        serde_json::to_string(&reasonable_plot).unwrap()
    );
    assert!(reasonable_plot == generated_plot);
}

#[test]
fn test_string_type() {
    let raw_code = r#"
    {
        map: {
            mystring: {
                type: "raw",
                data: "hello"
            }
        },
        data: [{
            type: "scatter",
            x: [0, 1, 2],
            y: [0, 0, 0],
            name: "map.mystring"
        }]
    }
    "#;
    let generated_plot = code_handler::handle_json_input(raw_code.to_string()).unwrap();
    println!(
        "Generated string: {}",
        serde_json::to_string(&generated_plot).unwrap()
    );
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0, 2.0], vec![0.0, 0.0, 0.0]).name("hello");
    reasonable_plot.add_trace(trace);
    println!(
        "Reasonable: {}",
        serde_json::to_string(&reasonable_plot).unwrap()
    );
    assert!(reasonable_plot == generated_plot);
}

#[test]
fn test_array_of_strings() {
    let raw_code = r#"
    {
        map: {
            mystrings: {
                type: "raw",
                data: ["a", "b", "c"]
            }
        },
        data: [{
            type: "scatter",
            x: [0, 1, 2],
            y: [0, 0, 0],
            text_array: "map.mystrings"
        }]
    }
    "#;
    let generated_plot = code_handler::handle_json_input(raw_code.to_string()).unwrap();
    println!(
        "Generated string array: {}",
        serde_json::to_string(&generated_plot).unwrap()
    );
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0, 2.0], vec![0.0, 0.0, 0.0]).text_array(vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string(),
    ]);
    reasonable_plot.add_trace(trace);
    println!(
        "Reasonable: {}",
        serde_json::to_string(&reasonable_plot).unwrap()
    );
    assert!(reasonable_plot == generated_plot);
}

#[test]
fn test_g_repeat_with_string() {
    let raw_code = r#"
    {
        map: {
            myrepeats: {
                type: "g-repeat",
                value: "test",
                count: 3
            }
        },
        data: [{
            type: "scatter",
            x: [0, 1, 2],
            y: [0, 0, 0],
            text_array: "map.myrepeats"
        }]
    }
    "#;
    let generated_plot = code_handler::handle_json_input(raw_code.to_string()).unwrap();
    println!(
        "Generated repeat string: {}",
        serde_json::to_string(&generated_plot).unwrap()
    );
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0, 2.0], vec![0.0, 0.0, 0.0]).text_array(vec![
        "test".to_string(),
        "test".to_string(),
        "test".to_string(),
    ]);
    reasonable_plot.add_trace(trace);
    println!(
        "Reasonable: {}",
        serde_json::to_string(&reasonable_plot).unwrap()
    );
    assert!(reasonable_plot == generated_plot);
}

#[test]
fn test_if_type() {
    let raw_code = r#"
    {
        map: {
            mycond: {
                type: "if",
                condition: "1 > 0",
                true: 0.5,
                false: 0.0
            }
        },
        data: [{
            type: "scatter",
            x: [0, 1, 2],
            y: [0, 0, 0],
            opacity: "map.mycond"
        }]
    }
    "#;
    let generated_plot = code_handler::handle_json_input(raw_code.to_string()).unwrap();
    println!(
        "Generated if: {}",
        serde_json::to_string(&generated_plot).unwrap()
    );
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0, 2.0], vec![0.0, 0.0, 0.0]).opacity(0.5);
    reasonable_plot.add_trace(trace);
    println!(
        "Reasonable: {}",
        serde_json::to_string(&reasonable_plot).unwrap()
    );
    assert!(reasonable_plot == generated_plot);
}

#[test]
fn test_time_type() {
    let raw_code = r#"
    {
        map: {
            mytime: {
                type: "time",
                start: "2023-01-01",
                end: "2023-01-02",
                interval: "1 day"
            }
        },
        data: [{
            type: "scatter",
            x: [0, 1],
            y: [0, 0],
            text_array: "map.mytime"
        }]
    }
    "#;
    let generated_plot = code_handler::handle_json_input(raw_code.to_string()).unwrap();
    println!(
        "Generated time: {}",
        serde_json::to_string(&generated_plot).unwrap()
    );
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0], vec![0.0, 0.0])
        .text_array(vec!["2023-01-01".to_string(), "2023-01-02".to_string()]);
    reasonable_plot.add_trace(trace);
    println!(
        "Reasonable: {}",
        serde_json::to_string(&reasonable_plot).unwrap()
    );
    assert!(reasonable_plot == generated_plot);
}
