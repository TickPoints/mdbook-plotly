use anyhow::{Result, anyhow, Context};
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
    let result = serde_json::from_value::<DataPack<T>>(result.take())
        .with_context(|| format!("failed to deserialize field '{}'", name))?
        .unwrap(map)
        .with_context(|| format!("failed to unwrap DataPack for field '{}'", name))?;
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
            let direct_result = serde_json::from_value::<T>(value.clone())
                .with_context(|| "failed to deserialize non-object value")?;
            return Ok(direct_result);
        }
        let value_type = if let Some(Value::String(s)) = value.get("type") {
            s.clone()
        } else {
            return Err(anyhow!("`type` must be a string"));
        };
        use fasteval::ez_eval;
        match value_type.as_str() {
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
                serde_json::from_value(result.into())
                    .with_context(|| format!("failed to deserialize generated list for type '{}'", value_type))
            }
            "g-number" => {
                let expr: String = must_translate(&mut value, map, "expr")?;
                let data = ez_eval(&expr, &mut fasteval::EmptyNamespace {})?;
                serde_json::from_value(data.into())
                    .with_context(|| format!("failed to deserialize generated number for type '{}'", value_type))
            }
            "g-range" => {
                let begin: f64 = must_translate(&mut value, map, "begin")?;
                let end: f64 = must_translate(&mut value, map, "end")?;
                let step: f64 = if value.get("step").is_some() {
                    must_translate(&mut value, map, "step")?
                } else {
                    1.0
                };
                if step <= 0.0 {
                    return Err(anyhow!("step must be positive"));
                }
                let mut result = vec![];
                let mut current = begin;
                while current < end {
                    result.push(Value::from(current));
                    current += step;
                }
                serde_json::from_value(result.into())
                    .with_context(|| format!("failed to deserialize generated range for type '{}'", value_type))
            }
            "g-repeat" => {
                let val: Value = must_translate(&mut value, map, "value")?;
                let count: u64 = must_translate(&mut value, map, "count")?;
                let result = std::iter::repeat_n(val, count as usize).collect::<Vec<_>>();
                serde_json::from_value(result.into())
                    .with_context(|| format!("failed to deserialize repeated values for type '{}'", value_type))
            }
            "g-linear" => {
                let begin: f64 = must_translate(&mut value, map, "begin")?;
                let end: f64 = must_translate(&mut value, map, "end")?;
                let count: u64 = must_translate(&mut value, map, "count")?;
                if count == 0 {
                    return Err(anyhow!("count must be positive"));
                }
                let mut result = Vec::with_capacity(count as usize);
                if count == 1 {
                    result.push(Value::from(begin));
                } else {
                    let step = (end - begin) / ((count - 1) as f64);
                    for i in 0..count {
                        let val = begin + (i as f64) * step;
                        result.push(Value::from(val));
                    }
                }
                serde_json::from_value(result.into())
                    .with_context(|| format!("failed to deserialize linear spaced values for type '{}'", value_type))
            }
            _ => Err(anyhow!("unknown type `{}`", value_type)),
        }
    }

    pub fn unwrap(self, map: &Map) -> Result<T> {
        let result = match self {
            Self::Data(data) => data,
            Self::Index(index) => {
                let value = map.get(&index).ok_or_else(|| {
                    let available_keys: Vec<_> = map.keys().take(5).cloned().collect();
                    anyhow!(
                        "Invalid index: '{}' (available keys: {}{})",
                        &index,
                        available_keys.join(", "),
                        if map.keys().len() > 5 { ", ..." } else { "" }
                    )
                })?;
                Self::parse_map(map, value.clone())
                    .with_context(|| format!("failed to parse map entry '{}'", index))?
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
