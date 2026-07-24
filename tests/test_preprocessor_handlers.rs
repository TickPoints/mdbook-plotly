use mdbook_plotly::preprocessor::config::PreprocessorConfig;
use mdbook_plotly::preprocessor::handlers;
use mdbook_preprocessor::book::Chapter;
use pulldown_cmark::Event;
use std::path::Path;

fn chapter(content: &str) -> Chapter {
    Chapter::new("Test", content.to_string(), "test.md", Vec::new())
}

fn handle_content(content: &str) -> String {
    let mut chapter = chapter(content);
    handlers::handle(&mut chapter, &PreprocessorConfig::default(), Path::new("."));
    chapter.content
}

#[cfg(feature = "plotly-html-handler")]
fn plotly_header() -> String {
    match handlers::plotly_html_handler::inject_header() {
        Event::Html(html) => html.to_string(),
        _ => unreachable!(),
    }
}

#[test]
fn handle_leaves_plain_chapter_unchanged() {
    let content = "# Title\n\nPlain content with `inline code`.\n";

    assert_eq!(handle_content(content), content);
}

#[test]
fn handle_leaves_non_plot_fenced_code_unchanged() {
    let content = "# Title\n\n```rust\nfn main() {}\n```\n";

    assert_eq!(handle_content(content), content);
}

#[test]
fn handle_leaves_plain_chapter_with_plot_word_unchanged() {
    let content = "# Title\n\nThis paragraph mentions plotly but has no chart fence.\n";

    assert_eq!(handle_content(content), content);
}

#[cfg(feature = "plotly-html-handler")]
#[test]
fn handle_injects_html_header_only_for_generated_plots() {
    let content = r#"```plot
{
    data: [{
        type: "scatter",
        x: [0, 1],
        y: [1, 2]
    }]
}
```
"#;

    let output = handle_content(content);
    let header = plotly_header();

    assert!(output.contains(&header));
    assert_eq!(output.matches(&header).count(), 1);
}

#[cfg(feature = "plotly-html-handler")]
#[test]
fn handle_detects_plot_fences_inside_lists() {
    let content = r#"- chart

  ```plot
  {
      data: [{
          type: "scatter",
          x: [0, 1],
          y: [1, 2]
      }]
  }
  ```
"#;

    let output = handle_content(content);
    let header = plotly_header();

    assert!(output.contains(&header));
}

#[cfg(feature = "plotly-html-handler")]
#[test]
fn handle_processes_complete_markdown_chapter() {
    let content = r#"# Metrics

Intro paragraph before the chart.

```rust
let untouched = true;
```

```plotly
{
    layout: {
        title: "Trend"
    },
    data: [{
        type: "scatter",
        x: [0, 1],
        y: [1, 2]
    }]
}
```

## After

Closing paragraph.
"#;

    let output = handle_content(content);
    let header = plotly_header();

    assert!(output.contains(&header));
    assert!(output.contains("# Metrics"));
    assert!(output.contains("Intro paragraph before the chart."));
    assert!(output.contains("```rust"));
    assert!(output.contains("let untouched = true;"));
    assert!(output.contains("Trend"));
    assert!(output.contains("## After"));
    assert!(output.contains("Closing paragraph."));
    assert!(!output.contains("```plotly"));
}

#[cfg(feature = "plotly-html-handler")]
#[test]
fn handle_does_not_inject_html_header_for_invalid_plot() {
    let content = "```plot\nnot valid json\n```\n";

    let output = handle_content(content);
    let header = plotly_header();

    assert!(!output.contains(&header));
}
