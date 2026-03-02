use super::until::{Map, must_translate};
use crate::translate;
use anyhow::Result;
use plotly::DensityMapbox;

pub fn parse_density_mapbox_data(
    dm_obj: &mut serde_json::Value,
    map: &Map,
) -> Result<Box<DensityMapbox<f64, f64, f64>>> {
    let lat: Vec<f64> = must_translate(dm_obj, "lat")?;
    let lon: Vec<f64> = must_translate(dm_obj, "lon")?;
    let z: Vec<f64> = must_translate(dm_obj, "z")?;
    let dm = DensityMapbox::new(lat, lon, z);
    let dm = translate! {
        dm,
        dm_obj,
        map,
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
