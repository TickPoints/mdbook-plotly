use mdbook_plotly::preprocessor::handlers::code_handler::until::Color;
use plotly::color::{NamedColor, Rgba};

#[test]
fn test_color() {
    let color = serde_json::to_string(&Color::NamedColor(NamedColor::Black)).unwrap();
    assert_eq!(&color, r#"{"named_color":"black"}"#);
    let color = serde_json::to_string(&Color::RgbaColor(Rgba::new(0, 0, 0, 0.0))).unwrap();
    assert_eq!(&color, r#"{"rgba_color":"rgba(0, 0, 0, 0)"}"#);
}
