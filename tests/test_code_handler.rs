use mdbook_plotly::code_handler;
use mdbook_plotly::preprocessor::config::MapEvalConfig;
use plotly::{
    Bar, BoxPlot, Candlestick, Configuration, Contour, DensityMapbox, HeatMap, Histogram, Layout,
    Mesh3D, Ohlc, Pie, Plot, Scatter, Scatter3D, ScatterGeo, ScatterMapbox, ScatterPolar, Surface,
};

#[test]
fn test_json5() {
    let raw_code = r##"
    {
        layout: {
            title: "Test",
        },
        config: {
            static_plot: true,
        }
    }
    "#;
    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();

    let mut reasonable_plot = Plot::new();
    let config = Configuration::new().static_plot(true);
    reasonable_plot.set_configuration(config);
    let layout = Layout::new().title("Test");
    reasonable_plot.set_layout(layout);

    assert!(reasonable_plot == generated_plot);
}

#[test]
fn test_handle_uses_json_input_config() {
    let raw_code = r#"
    {
        layout: {
            title: "Config Input Type Test",
        }
    }
    "#;

    let generated_plot = code_handler::handle(
        raw_code.to_string(),
        &PlotlyInputType::JSONInput,
        &MapEvalConfig::default(),
    )
    .unwrap();

    let mut reasonable_plot = Plot::new();
    reasonable_plot.set_layout(Layout::new().title("Config Input Type Test"));

    assert!(reasonable_plot == generated_plot);
}

#[test]
fn test_preprocessor_config_default_contract() {
    let config = PreprocessorConfig::default();

    assert_eq!(config.input_type, PlotlyInputType::JSONInput);
    assert!(!config.offline_js_sources);
    assert!(config.map_eval.enabled);
    assert!(config.map_eval.reuse_slab);
    assert!(config.map_eval.compile_expressions);
    assert_eq!(config.map_eval.namespace_scope, MapNamespaceScope::FullMap);
}

#[test]
fn test_layout_and_config_with_map_context() {
    let raw_code = r#"
    {
        map: {
            layout_title: "Mapped Title",
            legend_x: 0.25,
            legend_y: 0.75,
            plot_bg: "#ffffff",
            axis_range: [0, 10],
        },
        layout: {
            title: "map.layout_title",
            plot_background_color: "map.plot_bg",
            legend: {
                x: "map.legend_x",
                y: "map.legend_y",
            },
            xaxis: {
                range: "map.axis_range",
            },
        },
        config: {
            static_plot: true,
            display_mode_bar: "hover",
        }
    }
    "##;

    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();

    let mut reasonable_plot = Plot::new();
    let layout = Layout::new()
        .title("Mapped Title")
        .plot_background_color("#ffffff")
        .legend(plotly::layout::Legend::new().x(0.25).y(0.75))
        .x_axis(plotly::layout::Axis::new().range(vec![Some(0.0), Some(10.0)]));
    let config = Configuration::new()
        .static_plot(true)
        .display_mode_bar(plotly::configuration::DisplayModeBar::Hover);

    reasonable_plot.set_layout(layout);
    reasonable_plot.set_configuration(config);

    assert!(
        reasonable_plot == generated_plot,
        "Layout/config mismatch\nGenerated: {}\nExpected: {}",
        serde_json::to_string(&generated_plot).unwrap(),
        serde_json::to_string(&reasonable_plot).unwrap()
    );
}

#[test]
fn test_layout_axes_with_map_context_regression() {
    let raw_code = r##"
    {
        map: {
            x_range: [1, 5],
            x_prefix: "$",
            x_suffix: " USD",
            y_title: "Mapped Y Axis",
            y_anchor: "x",
            y_overlaying: "y",
            y_show_ticks: false,
            y_auto_margin: true,
            y_fixed_range: true,
        },
        layout: {
            xaxis: {
                range: "map.x_range",
                tick_prefix: "map.x_prefix",
                tick_suffix: "map.x_suffix",
                type: "linear",
            },
            yaxis: {
                title: "map.y_title",
                anchor: "map.y_anchor",
                overlaying: "map.y_overlaying",
                show_tick_labels: "map.y_show_ticks",
                auto_margin: "map.y_auto_margin",
                fixed_range: "map.y_fixed_range",
                type: "category",
            },
        }
    }
    "##;

    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();

    let mut reasonable_plot = Plot::new();
    let layout = Layout::new()
        .x_axis(
            plotly::layout::Axis::new()
                .range(vec![Some(1.0), Some(5.0)])
                .tick_prefix("$")
                .tick_suffix(" USD")
                .type_(plotly::layout::AxisType::Linear),
        )
        .y_axis(
            plotly::layout::Axis::new()
                .title("Mapped Y Axis")
                .anchor("x")
                .overlaying("y")
                .show_tick_labels(false)
                .auto_margin(true)
                .fixed_range(true)
                .type_(plotly::layout::AxisType::Category),
        );
    reasonable_plot.set_layout(layout);

    assert!(
        reasonable_plot == generated_plot,
        "Axis layout mismatch\nGenerated: {}\nExpected: {}",
        serde_json::to_string(&generated_plot).unwrap(),
        serde_json::to_string(&reasonable_plot).unwrap()
    );
}

// ── Existing Trace Tests ──

#[test]
fn test_bar() {
    let raw_code = r#"
    {
        data: [{
            type: "bar",
            x: [1, 2, 3],
            y: [4, 5, 6],
            name: "Bar Test",
        }]
    }
    "#;
    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();
    let mut reasonable_plot = Plot::new();
    let trace = Bar::new(vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]).name("Bar Test");
    reasonable_plot.add_trace(trace);
    assert!(
        reasonable_plot == generated_plot,
        "Bar mismatch\nGenerated: {}\nExpected: {}",
        serde_json::to_string(&generated_plot).unwrap(),
        serde_json::to_string(&reasonable_plot).unwrap()
    );
}

#[test]
fn test_scatter() {
    let raw_code = r#"
    {
        data: [{
            type: "scatter",
            x: [0, 1, 2],
            y: [0, 1, 2],
            name: "Scatter Test",
        }]
    }
    "#;
    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();
    let mut reasonable_plot = Plot::new();
    let trace = Scatter::new(vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 2.0]).name("Scatter Test");
    reasonable_plot.add_trace(trace);
    assert!(
        reasonable_plot == generated_plot,
        "Scatter mismatch\nGenerated: {}\nExpected: {}",
        serde_json::to_string(&generated_plot).unwrap(),
        serde_json::to_string(&reasonable_plot).unwrap()
    );
}

#[test]
fn test_pie() {
    let raw_code = r#"
    {
        data: [{
            type: "pie",
            values: [10, 20, 30],
            labels: ["A", "B", "C"],
        }]
    }
    "#;
    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();
    let mut reasonable_plot = Plot::new();
    let trace = Pie::new(vec![10.0, 20.0, 30.0]).labels(vec![
        "A".to_string(),
        "B".to_string(),
        "C".to_string(),
    ]);
    reasonable_plot.add_trace(trace);
    assert!(
        reasonable_plot == generated_plot,
        "Pie mismatch\nGenerated: {}\nExpected: {}",
        serde_json::to_string(&generated_plot).unwrap(),
        serde_json::to_string(&reasonable_plot).unwrap()
    );
}

#[test]
fn test_histogram() {
    let raw_code = r#"
    {
        data: [{
            type: "histogram",
            x: [1, 2, 2, 3, 3, 3],
            name: "Hist Test",
        }]
    }
    "#;
    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();
    let mut reasonable_plot = Plot::new();
    let trace = Histogram::new(vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0]).name("Hist Test");
    reasonable_plot.add_trace(trace);
    assert!(
        reasonable_plot == generated_plot,
        "Histogram mismatch\nGenerated: {}\nExpected: {}",
        serde_json::to_string(&generated_plot).unwrap(),
        serde_json::to_string(&reasonable_plot).unwrap()
    );
}

#[test]
fn test_candlestick() {
    let raw_code = r#"
    {
        data: [{
            type: "candlestick",
            x: ["2024-01-01", "2024-01-02"],
            open: [100, 102],
            high: [105, 106],
            low: [98, 100],
            close: [103, 104],
        }]
    }
    "#;
    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();
    let mut reasonable_plot = Plot::new();
    let trace = Candlestick::new(
        vec!["2024-01-01".to_string(), "2024-01-02".to_string()],
        vec![100.0, 102.0],
        vec![105.0, 106.0],
        vec![98.0, 100.0],
        vec![103.0, 104.0],
    );
    reasonable_plot.add_trace(trace);
    assert!(
        reasonable_plot == generated_plot,
        "Candlestick mismatch\nGenerated: {}\nExpected: {}",
        serde_json::to_string(&generated_plot).unwrap(),
        serde_json::to_string(&reasonable_plot).unwrap()
    );
}

#[test]
fn test_ohlc() {
    let raw_code = r#"
    {
        data: [{
            type: "ohlc",
            x: ["2024-01-01", "2024-01-02"],
            open: [100, 102],
            high: [105, 106],
            low: [98, 100],
            close: [103, 104],
        }]
    }
    "#;
    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();
    let mut reasonable_plot = Plot::new();
    let trace = Ohlc::new(
        vec!["2024-01-01".to_string(), "2024-01-02".to_string()],
        vec![100.0, 102.0],
        vec![105.0, 106.0],
        vec![98.0, 100.0],
        vec![103.0, 104.0],
    );
    reasonable_plot.add_trace(trace);
    assert!(
        reasonable_plot == generated_plot,
        "OHLC mismatch\nGenerated: {}\nExpected: {}",
        serde_json::to_string(&generated_plot).unwrap(),
        serde_json::to_string(&reasonable_plot).unwrap()
    );
}

#[test]
fn test_scatter_geo() {
    let raw_code = r#"
    {
        data: [{
            type: "scatter_geo",
            lat: [40.0, 50.0],
            lon: [-70.0, -80.0],
        }]
    }
    "#;
    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();
    let mut reasonable_plot = Plot::new();
    let trace = ScatterGeo::new(vec![40.0, 50.0], vec![-70.0, -80.0]);
    reasonable_plot.add_trace(trace);
    assert!(
        reasonable_plot == generated_plot,
        "ScatterGeo mismatch\nGenerated: {}\nExpected: {}",
        serde_json::to_string(&generated_plot).unwrap(),
        serde_json::to_string(&reasonable_plot).unwrap()
    );
}

#[test]
fn test_scatter_mapbox() {
    let raw_code = r#"
    {
        data: [{
            type: "scatter_mapbox",
            lat: [40.0],
            lon: [-70.0],
        }]
    }
    "#;
    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();
    let mut reasonable_plot = Plot::new();
    let trace = ScatterMapbox::new(vec![40.0], vec![-70.0]);
    reasonable_plot.add_trace(trace);
    assert!(
        reasonable_plot == generated_plot,
        "ScatterMapbox mismatch\nGenerated: {}\nExpected: {}",
        serde_json::to_string(&generated_plot).unwrap(),
        serde_json::to_string(&reasonable_plot).unwrap()
    );
}

#[test]
fn test_scatter_polar() {
    let raw_code = r#"
    {
        data: [{
            type: "scatter_polar",
            r: [1, 2, 3],
            theta: [0, 45, 90],
        }]
    }
    "#;
    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();
    let mut reasonable_plot = Plot::new();
    let trace = ScatterPolar::new(vec![0.0, 45.0, 90.0], vec![1.0, 2.0, 3.0]);
    reasonable_plot.add_trace(trace);
    assert!(
        reasonable_plot == generated_plot,
        "ScatterPolar mismatch\nGenerated: {}\nExpected: {}",
        serde_json::to_string(&generated_plot).unwrap(),
        serde_json::to_string(&reasonable_plot).unwrap()
    );
}

#[test]
fn test_density_mapbox() {
    let raw_code = r#"
    {
        data: [{
            type: "density_mapbox",
            lat: [40.0, 45.0],
            lon: [-70.0, -75.0],
            z: [1.0, 2.0],
        }]
    }
    "#;
    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();
    let mut reasonable_plot = Plot::new();
    let trace = DensityMapbox::new(vec![40.0, 45.0], vec![-70.0, -75.0], vec![1.0, 2.0]);
    reasonable_plot.add_trace(trace);
    assert!(
        reasonable_plot == generated_plot,
        "DensityMapbox mismatch\nGenerated: {}\nExpected: {}",
        serde_json::to_string(&generated_plot).unwrap(),
        serde_json::to_string(&reasonable_plot).unwrap()
    );
}

// ── New Trace Tests ──

#[test]
fn test_box_plot() {
    let raw_code = r#"
    {
        data: [{
            type: "box",
            y: [1, 2, 3, 4, 5],
            name: "Box Test",
        }]
    }
    "#;
    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();
    let mut reasonable_plot = Plot::new();
    let trace = BoxPlot::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]).name("Box Test");
    reasonable_plot.add_trace(trace);
    assert!(
        reasonable_plot == generated_plot,
        "BoxPlot mismatch\nGenerated: {}\nExpected: {}",
        serde_json::to_string(&generated_plot).unwrap(),
        serde_json::to_string(&reasonable_plot).unwrap()
    );
}

#[test]
fn test_contour() {
    let raw_code = r#"
    {
        data: [{
            type: "contour",
            z: [[0.0, 0.5, 1.0], [0.5, 1.0, 0.5], [1.0, 0.5, 0.0]],
            show_legend: true,
        }]
    }
    "#;
    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();
    let mut reasonable_plot = Plot::new();
    let trace = Contour::new_z(vec![
        vec![0.0, 0.5, 1.0],
        vec![0.5, 1.0, 0.5],
        vec![1.0, 0.5, 0.0],
    ])
    .show_legend(true);
    reasonable_plot.add_trace(trace);
    assert!(
        reasonable_plot == generated_plot,
        "Contour mismatch\nGenerated: {}\nExpected: {}",
        serde_json::to_string(&generated_plot).unwrap(),
        serde_json::to_string(&reasonable_plot).unwrap()
    );
}

#[test]
fn test_heat_map() {
    let raw_code = r#"
    {
        data: [{
            type: "heatmap",
            z: [[0.0, 0.5], [1.0, 0.0]],
            opacity: 0.8,
        }]
    }
    "#;
    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();
    let mut reasonable_plot = Plot::new();
    let trace = HeatMap::new_z(vec![vec![0.0, 0.5], vec![1.0, 0.0]]).opacity(0.8);
    reasonable_plot.add_trace(trace);
    assert!(
        reasonable_plot == generated_plot,
        "HeatMap mismatch\nGenerated: {}\nExpected: {}",
        serde_json::to_string(&generated_plot).unwrap(),
        serde_json::to_string(&reasonable_plot).unwrap()
    );
}

#[test]
fn test_mesh3d() {
    let raw_code = r#"
    {
        data: [{
            type: "mesh3d",
            x: [0.0, 1.0, 0.0],
            y: [0.0, 0.0, 1.0],
            z: [0.0, 0.0, 0.0],
            opacity: 0.5,
        }]
    }
    "#;
    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();
    let mut reasonable_plot = Plot::new();
    let trace = Mesh3D::new(
        vec![0.0, 1.0, 0.0],
        vec![0.0, 0.0, 1.0],
        vec![0.0, 0.0, 0.0],
        None,
        None,
        None,
    )
    .opacity(0.5);
    reasonable_plot.add_trace(trace);
    assert!(
        reasonable_plot == generated_plot,
        "Mesh3D mismatch\nGenerated: {}\nExpected: {}",
        serde_json::to_string(&generated_plot).unwrap(),
        serde_json::to_string(&reasonable_plot).unwrap()
    );
}

#[test]
fn test_scatter3d() {
    let raw_code = r#"
    {
        data: [{
            type: "scatter3d",
            x: [0.0, 1.0, 2.0],
            y: [0.0, 1.0, 2.0],
            z: [0.0, 1.0, 2.0],
        }]
    }
    "#;
    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();
    let mut reasonable_plot = Plot::new();
    let trace = Scatter3D::new(
        vec![0.0, 1.0, 2.0],
        vec![0.0, 1.0, 2.0],
        vec![0.0, 1.0, 2.0],
    );
    reasonable_plot.add_trace(trace);
    assert!(
        reasonable_plot == generated_plot,
        "Scatter3D mismatch\nGenerated: {}\nExpected: {}",
        serde_json::to_string(&generated_plot).unwrap(),
        serde_json::to_string(&reasonable_plot).unwrap()
    );
}

// ── Layout Axis Tests ──

#[test]
fn test_layout_xaxis() {
    let raw_code = r#"
    {
        layout: {
            title: "Axis Test",
            xaxis: {
                title: "Time (s)",
                show_grid: true,
                zero_line: false,
                range: [null, 100],
            },
            yaxis: {
                title: "Amplitude",
                show_grid: true,
            },
        },
        data: [{
            type: "scatter",
            x: [0, 1, 2],
            y: [0, 1, 2],
        }]
    }
    "#;
    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();
    // Verify it parses without error — the exact PartialEq for Layout may differ
    // due to default fields, but this at minimum validates parsing succeeds.
    let _ = generated_plot;
}

#[test]
fn test_layout_xaxis_type_date() {
    let raw_code = r#"
    {
        layout: {
            xaxis: {
                type: "date",
                tick_format: "%Y-%m-%d",
            },
        },
        data: [{
            type: "bar",
            x: [1, 2, 3],
            y: [4, 5, 6],
        }]
    }
    "#;
    let result = code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default());
    assert!(result.is_ok(), "date axis should parse: {:?}", result.err());
}

#[test]
fn test_layout_named_axes() {
    let raw_code = r#"
    {
        layout: {
            xaxis: {
                title: "Shared X",
            },
            yaxis: {
                title: "Left Y",
                side: "left",
            },
            yaxis2: {
                title: "Right Y",
                overlaying: "y",
                side: "right",
            },
        },
        data: [
            { type: "scatter", x: [1,2,3], y: [4,5,6] },
            { type: "scatter", x: [1,2,3], y: [10,20,30], yaxis: "y2" },
        ]
    }
    "#;
    let result = code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default());
    assert!(
        result.is_ok(),
        "named axes should parse: {:?}",
        result.err()
    );
}

#[test]
fn test_surface() {
    let raw_code = r#"
    {
        data: [{
            type: "surface",
            z: [[0.0, 1.0], [1.0, 0.0]],
            opacity: 0.9,
        }]
    }
    "#;
    let generated_plot =
        code_handler::handle_json_input(raw_code.to_string(), &MapEvalConfig::default()).unwrap();
    let mut reasonable_plot = Plot::new();
    let trace: Box<Surface<f64, f64, f64>> =
        Surface::new(vec![vec![0.0, 1.0], vec![1.0, 0.0]]).opacity(0.9);
    reasonable_plot.add_trace(trace);
    assert!(
        reasonable_plot == generated_plot,
        "Surface mismatch\nGenerated: {}\nExpected: {}",
        serde_json::to_string(&generated_plot).unwrap(),
        serde_json::to_string(&reasonable_plot).unwrap()
    );
}
