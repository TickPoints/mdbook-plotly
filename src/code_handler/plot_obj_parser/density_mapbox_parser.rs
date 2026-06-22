use super::until::must_translate_from_context;
use crate::code_handler::parse_context::ParseContext;
use crate::translate_with_config;
use anyhow::Result;
use plotly::{DensityMapbox, Trace};

pub fn parse_density_mapbox_data(
    dm_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<DensityMapbox<f64, f64, f64>>> {
    let lat: Vec<f64> = must_translate_from_context(dm_obj, context, "lat")?;
    let lon: Vec<f64> = must_translate_from_context(dm_obj, context, "lon")?;
    let z: Vec<f64> = must_translate_from_context(dm_obj, context, "z")?;
    let dm = DensityMapbox::new(lat, lon, z);
    let dm = translate_with_config! {
        dm,
        dm_obj,
        context.map(),
        context.map_eval(),
        (show_legend, bool),
        (name, String),
        (legend_group, String),
        (legend_rank, usize),
        (opacity, f64),
        (radius, u8),
        (zoom, u8),
        (zauto, bool),
        (zmin, f64),
        (zmid, f64),
        (zmax, f64),
        (subplot, String),
    }?;
    Ok(dm)
}

pub fn parse_density_mapbox_trace(
    dm_obj: &mut serde_json::Value,
    context: &ParseContext<'_>,
) -> Result<Box<dyn Trace>> {
    Ok(parse_density_mapbox_data(dm_obj, context)?)
}
