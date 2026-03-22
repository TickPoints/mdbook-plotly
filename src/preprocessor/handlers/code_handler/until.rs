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
    fn parse_map(map: &Map, mut value: Value) -> Result<T> {
        if !value.is_object() {
            let direct_result = serde_json::from_value::<T>(value.clone())?;
            return Ok(direct_result);
        }
        let value_type = &value["type"];
        let value_type = if !value_type.is_string() {
            return Err(anyhow!("`type` must be a string"));
        } else {
            // SAFETY: `value_type` is a string
            value_type.as_str().unwrap()
        };
        use fasteval::ez_eval;
        match value_type {
            "raw" => {
                let result = must_translate(&mut value, map, "data")?;
                Ok(result)
            }
            // `g-` means generator
            "g-number-list" => {
                let index_begin: u64 = must_translate(&mut value, map, "begin")?;
                let index_end: u64 = must_translate(&mut value, map, "end")?;
                let expr: String = must_translate(&mut value, map, "expr")?;
                let mut result = vec![];
                let mut namespace = fasteval::StrToF64Namespace::new();
                for i in index_begin..index_end {
                    namespace.insert("i", i as f64);
                    let data = ez_eval(&expr, &mut namespace)?;
                    result.push(Value::from(data));
                }
                Ok(serde_json::from_value(result.into())?)
            }
            "g-number" => {
                let expr: String = must_translate(&mut value, map, "expr")?;
                let data = ez_eval(&expr, &mut fasteval::EmptyNamespace {})?;
                Ok(serde_json::from_value(data.into())?)
            }
            _ => Err(anyhow!("unknown type `{}`", value_type)),
        }
    }

    pub fn unwrap(self, map: &Map) -> Result<T> {
        let result = match self {
            Self::Data(data) => data,
            Self::Index(index) => {
                let value = map
                    .get(&index)
                    .ok_or_else(|| anyhow!("Invalid index: {}", &index))?;
                Self::parse_map(map, value.clone())?
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
