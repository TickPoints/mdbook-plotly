use anyhow::{Result, anyhow};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::DeserializeOwned};
use serde_json::{Map as JsonMap, Value, value::Index};
use std::fmt::{Debug, Display};

pub type Map = JsonMap<String, Value>;

#[inline]
pub fn must_translate<T, N>(obj: &mut Value, map: &Map, name: N) -> Result<T>
where
    T: DeserializeOwned + Serialize + Debug + Clone,
    N: Index + Display,
{
    let result = obj
        .get_mut(&name)
        .ok_or(anyhow!("missing `{}` field", name))?;
    let result = serde_json::from_value::<DataPack<T>>(result.take())?.unwrap(map)?;
    Ok(result)
}

#[derive(Clone, Debug)]
pub enum DataPack<T> {
    Data(T),
    Index(String),
}

impl<T> DataPack<T>
where
    T: DeserializeOwned + Serialize + Debug + Clone,
{
    pub fn unwrap(self, map: &Map) -> Result<T> {
        let result = match self {
            Self::Data(data) => data,
            Self::Index(index) => {
                let value = map
                    .get(&index)
                    .ok_or_else(|| anyhow!("Invalid index: {}", &index))?;
                serde_json::from_value::<T>(value.clone())?
            }
        };
        Ok(result)
    }
}

impl<'de, T> Deserialize<'de> for DataPack<T>
where
    T: DeserializeOwned + Serialize + Debug + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if let Value::String(ref s) = value
            && let Some(idx) = s.strip_prefix("map.")
        {
            return Ok(DataPack::Index(idx.to_string()));
        }
        serde_json::from_value::<T>(value)
            .map(DataPack::Data)
            .map_err(serde::de::Error::custom)
    }
}

impl<T> Serialize for DataPack<T>
where
    T: DeserializeOwned + Serialize + Debug + Clone,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Data(data) => data.serialize(serializer),
            Self::Index(index) => serializer.serialize_str(&format!("map.{index}")),
        }
    }
}

use plotly::color;

// This is to make Json look clearer when it is written.
#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Color {
    NamedColor(color::NamedColor),
    RgbColor(color::Rgb),
    RgbaColor(color::Rgba),
}

impl color::Color for Color {}
