use super::until::must_translate;
use crate::translate;
use anyhow::Result;
use plotly::Scatter;

pub fn parse_scatter_data(scatter_obj: &mut serde_json::Value) -> Result<Box<Scatter<f64, f64>>> {
    let x: Vec<f64> = must_translate(scatter_obj, "x")?;
    let y: Vec<f64> = must_translate(scatter_obj, "y")?;
    let scatter = Scatter::new(x, y);
    let scatter = translate! {
        scatter,
        scatter_obj,
        (web_gl_mode, bool),
    }?;
    Ok(scatter)
}
