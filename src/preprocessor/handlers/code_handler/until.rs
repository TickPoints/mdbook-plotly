use anyhow::{Context, Result, anyhow};
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

/// Attempt to deserialize a `Value` into `T`, attaching a static context message on failure.
fn try_deser<T: serde::de::DeserializeOwned>(value: Value, context: &str) -> Result<T> {
    serde_json::from_value(value).with_context(|| context.to_string())
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
                try_deser(
                    result.into(),
                    &format!("failed to deserialize generated list for type '{}'", value_type),
                )
            }

            "g-number" => {
                let expr: String = must_translate(&mut value, map, "expr")?;
                let data = ez_eval(&expr, &mut fasteval::EmptyNamespace {})?;
                try_deser(
                    data.into(),
                    &format!("failed to deserialize generated number for type '{}'", value_type),
                )
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
                try_deser(
                    result.into(),
                    &format!("failed to deserialize generated range for type '{}'", value_type),
                )
            }

            "g-repeat" => {
                let val: Value = must_translate(&mut value, map, "value")?;
                let count: u64 = must_translate(&mut value, map, "count")?;
                let result = std::iter::repeat_n(val, count as usize).collect::<Vec<_>>();
                try_deser(
                    result.into(),
                    &format!("failed to deserialize repeated values for type '{}'", value_type),
                )
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
                try_deser(
                    result.into(),
                    &format!("failed to deserialize linear spaced values for type '{}'", value_type),
                )
            }

            "if" => {
                let condition: String = must_translate(&mut value, map, "condition")?;
                let true_val: Value = must_translate(&mut value, map, "true")?;
                let false_val: Value = must_translate(&mut value, map, "false")?;

                let mut namespace = fasteval::StrToF64Namespace::new();
                let result = ez_eval(&condition, &mut namespace)?;
                let selected = if result != 0.0 { true_val } else { false_val };

                if !selected.is_object() {
                    serde_json::from_value::<T>(selected.clone())
                        .with_context(|| "failed to deserialize non-object if result")
                } else {
                    Self::parse_map(map, selected)
                }
            }

            "time" => {
                let start: String = must_translate(&mut value, map, "start")?;
                let end: String = must_translate(&mut value, map, "end")?;
                let interval: String = must_translate(&mut value, map, "interval")?;
                let format: Option<String> = if value.get("format").is_some() {
                    Some(must_translate(&mut value, map, "format")?)
                } else {
                    None
                };

                let start_dt = Self::parse_time_str(&start)?;
                let end_dt = Self::parse_time_str(&end)?;
                let step = Self::parse_duration_str(&interval)?;

                if step <= TimeDelta::zero() {
                    return Err(anyhow!("interval must be positive"));
                }

                let mut result: Vec<Value> = Vec::new();
                let mut current = start_dt;
                while current <= end_dt {
                    let ts_str = if let Some(ref fmt) = format {
                        current.format(fmt).to_string()
                    } else {
                        current.to_rfc3339()
                    };
                    result.push(Value::from(ts_str));
                    match current.checked_add_signed(step) {
                        Some(next) => current = next,
                        None => break,
                    }
                }

                try_deser(result.into(), "failed to deserialize time values")
            }

            "g-random" => {
                let min: f64 = must_translate(&mut value, map, "min")?;
                let max: f64 = must_translate(&mut value, map, "max")?;
                if min >= max {
                    return Err(anyhow!("min ({}) must be less than max ({})", min, max));
                }

                let integer: bool = value
                    .get("integer")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let seed: Option<u64> = if value.get("seed").is_some() {
                    Some(must_translate(&mut value, map, "seed")?)
                } else {
                    None
                };

                let has_count = value.get("count").is_some();

                let mut gen_value = |rng: &mut dyn RngCore| -> Result<Value> {
                    if integer {
                        Ok(Value::from(rng.gen_range(min as i64..max as i64)))
                    } else {
                        let v: f64 = rng.gen_range(min..max);
                        let n = serde_json::Number::from_f64(v)
                            .ok_or_else(|| anyhow!("failed to create JSON number from {}", v))?;
                        Ok(Value::from(n))
                    }
                };

                if has_count {
                    let count: u64 = must_translate(&mut value, map, "count")?;
                    if count == 0 {
                        return Err(anyhow!("count must be positive"));
                    }
                    let values = Self::with_rng(seed, |rng| {
                        (0..count)
                            .map(|_| gen_value(rng))
                            .collect::<Result<Vec<_>>>()
                    })??; // two-level Result: with_rng returned Result<Result<Vec<_>>>
                    try_deser(values.into(), "failed to deserialize g-random array")
                } else {
                    let single = Self::with_rng(seed, |rng| gen_value(rng))??;
                    try_deser(single, "failed to deserialize g-random single value")
                }
            }

            "g-choose" => {
                let options: Vec<Value> = must_translate(&mut value, map, "options")?;
                if options.is_empty() {
                    return Err(anyhow!("options must not be empty for g-choose"));
                }

                let seed: Option<u64> = if value.get("seed").is_some() {
                    Some(must_translate(&mut value, map, "seed")?)
                } else {
                    None
                };

                let has_count = value.get("count").is_some();

                if has_count {
                    let count: u64 = must_translate(&mut value, map, "count")?;
                    if count == 0 {
                        return Err(anyhow!("count must be positive"));
                    }
                    let selected = Self::with_rng(seed, |rng| {
                        (0..count)
                            .map(|_| {
                                let idx = rng.gen_range(0..options.len());
                                options[idx].clone()
                            })
                            .collect::<Vec<_>>()
                    })?;
                    try_deser(selected.into(), "failed to deserialize g-choose array")
                } else {
                    let picked = Self::with_rng(seed, |rng| {
                        let idx = rng.gen_range(0..options.len());
                        options[idx].clone()
                    })?;
                    try_deser(picked, "failed to deserialize g-choose single value")
                }
            }

            "g-env" => {
                let name: String = must_translate(&mut value, map, "name")?;
                let default: Option<String> = if value.get("default").is_some() {
                    Some(must_translate(&mut value, map, "default")?)
                } else {
                    None
                };

                let env_val = std::env::var(&name).ok().or(default).ok_or_else(|| {
                    anyhow!(
                        "environment variable '{}' is not set and no default provided",
                        name
                    )
                })?;

                try_deser(
                    Value::from(env_val),
                    &format!("failed to deserialize env value for '{}'", name),
                )
            }

            "g-join" => {
                let values: Vec<String> = must_translate(&mut value, map, "values")?;
                let separator: String = if value.get("separator").is_some() {
                    must_translate(&mut value, map, "separator")?
                } else {
                    "".to_string()
                };

                let joined = values.join(&separator);
                try_deser(Value::from(joined), "failed to deserialize joined string")
            }

            _ => Err(anyhow!("unknown type `{}`", value_type)),
        }
    }

    /// Parse a human-readable duration string like `1h`, `30m`, `1h30m15s`
    /// or combinations like `2d6h`.  Supports units: s, m, h, d, w.
    fn parse_duration_str(s: &str) -> Result<TimeDelta> {
        let s = s.trim();
        if s.is_empty() {
            return Err(anyhow!("duration string is empty"));
        }

        let mut total = TimeDelta::zero();
        let mut num_str = String::new();

        for ch in s.chars() {
            if ch.is_ascii_digit() || ch == '.' {
                num_str.push(ch);
            } else if ch.is_alphabetic() {
                let num: f64 = num_str
                    .parse()
                    .with_context(|| format!("invalid number in duration: '{}'", num_str))?;
                num_str.clear();

                let delta = match ch.to_ascii_lowercase() {
                    's' => TimeDelta::try_seconds(num as i64)
                        .ok_or_else(|| anyhow!("duration seconds overflow: {}", num))?,
                    'm' => {
                        let secs = (num * 60.0) as i64;
                        TimeDelta::try_seconds(secs)
                            .ok_or_else(|| anyhow!("duration minutes overflow: {}", num))?
                    }
                    'h' => {
                        let secs = (num * 3600.0) as i64;
                        TimeDelta::try_seconds(secs)
                            .ok_or_else(|| anyhow!("duration hours overflow: {}", num))?
                    }
                    'd' => {
                        let secs = (num * 86400.0) as i64;
                        TimeDelta::try_seconds(secs)
                            .ok_or_else(|| anyhow!("duration days overflow: {}", num))?
                    }
                    'w' => {
                        let secs = (num * 604800.0) as i64;
                        TimeDelta::try_seconds(secs)
                            .ok_or_else(|| anyhow!("duration weeks overflow: {}", num))?
                    }
                    other => return Err(anyhow!("unknown duration unit: '{}'", other)),
                };

                total = total
                    .checked_add(&delta)
                    .ok_or_else(|| anyhow!("duration overflow"))?;
            } else if ch.is_whitespace() {
                // skip whitespace
                continue;
            } else {
                return Err(anyhow!(
                    "unexpected character '{}' in duration string",
                    ch
                ));
            }
        }

        if !num_str.is_empty() {
            return Err(anyhow!("trailing number without unit: '{}'", num_str));
        }

        if total.is_zero() {
            return Err(anyhow!("duration must be positive, got: '{}'", s));
        }

        Ok(total)
    }

    /// Parse a time string into a `DateTime<Utc>`.
    ///
    /// Supported formats:
    /// - ISO 8601 / RFC 3339 (e.g. `2024-01-01T00:00:00Z`, `2024-01-01T00:00:00+08:00`)
    /// - Common date/time formats (`2024-01-01T00:00:00`, `2024-01-01 12:00:00`, `2024-01-01`)
    /// - `now` for the current UTC time
    /// - `now+…` or `now-…` with a duration string (e.g. `now-1h`, `now+2d`)
    fn parse_time_str(s: &str) -> Result<DateTime<Utc>> {
        let s = s.trim();

        // `now` and `now±<duration>`
        if let Some(rest) = s.strip_prefix("now") {
            let base = Utc::now();
            if rest.is_empty() {
                return Ok(base);
            }
            let sign_char = rest.chars().next().unwrap();
            let duration_str = &rest[1..]; // skip '+' or '-'
            let delta = Self::parse_duration_str(duration_str)?;
            return match sign_char {
                '+' => base
                    .checked_add_signed(delta)
                    .ok_or_else(|| anyhow!("time overflow for '{}'", s)),
                '-' => base
                    .checked_sub_signed(delta)
                    .ok_or_else(|| anyhow!("time overflow for '{}'", s)),
                _ => Err(anyhow!(
                    "expected '+' or '-' after 'now', got '{}'",
                    rest
                )),
            };
        }

        // RFC 3339 / ISO 8601
        if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
            return Ok(dt.with_timezone(&Utc));
        }

        // Fallback common formats
        let formats = [
            "%Y-%m-%dT%H:%M:%S%.f%:z",
            "%Y-%m-%dT%H:%M:%S%.f",
            "%Y-%m-%dT%H:%M:%S%:z",
            "%Y-%m-%dT%H:%M:%S",
            "%Y-%m-%d %H:%M:%S",
            "%Y-%m-%d",
        ];

        for fmt in &formats {
            if let Ok(naive) = NaiveDateTime::parse_from_str(s, fmt) {
                return Ok(naive.and_utc());
            }
            if let Ok(dt) = DateTime::parse_from_str(s, fmt) {
                return Ok(dt.with_timezone(&Utc));
            }
        }

        Err(anyhow!(
            "unable to parse time string: '{}'. Supported formats: RFC 3339, \
             'YYYY-MM-DDTHH:MM:SS', 'YYYY-MM-DD HH:MM:SS', 'YYYY-MM-DD', \
             'now', 'now±duration'",
            s
        ))
    }

    /// Execute a closure that needs a random number generator.
    ///
    /// If `seed` is `Some(u64)`, a seeded `StdRng` is created;
    /// otherwise the thread‑local RNG is used.
    fn with_rng<F, R>(seed: Option<u64>, f: F) -> R
    where
        F: FnOnce(&mut dyn RngCore) -> R,
    {
        if let Some(s) = seed {
            let mut rng = StdRng::seed_from_u64(s);
            f(&mut rng)
        } else {
            let mut rng = rand::thread_rng();
            f(&mut rng)
        }
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
