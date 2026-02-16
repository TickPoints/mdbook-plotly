use anyhow::{Result, anyhow};
use serde::de::DeserializeOwned;
use serde_json::{Value, value::Index};
use std::fmt::Display;

pub fn must_translate<T, N>(obj: &mut Value, name: N) -> Result<T>
where
    T: DeserializeOwned,
    N: Index + Display,
{
    let result = obj
        .get_mut(&name)
        .ok_or(anyhow!("missing `{}` field", name))?;
    let result = serde_json::from_value::<T>(result.take())?;
    Ok(result)
}
