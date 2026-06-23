// Tmp fix
#![allow(unexpected_cfgs)]

use crate::code_handler::parse_context::ParseContext;
use crate::preprocessor::config::{MapEvalConfig, MapNamespaceScope};
use anyhow::{Context, Result, anyhow};
use fasteval::{Compiler, EvalNamespace, Evaler, Parser, Slab};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::DeserializeOwned};
use serde_json::{Map as JsonMap, Value, value::Index};
use std::{collections::BTreeMap, fmt::Debug, fmt::Display};

#[cfg(feature = "map-parser-extensions")]
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeDelta, Utc};
#[cfg(feature = "map-parser-extensions")]
use rand::{Rng, RngExt, SeedableRng, rngs::StdRng};

pub type Map = JsonMap<String, Value>;

type Vars = BTreeMap<String, f64>;

#[derive(Clone, Debug)]
pub enum DataPack<T> {
    Data(T),
    Index(String),
}

#[allow(dead_code)]
#[inline]
pub fn must_translate_from_context<T, N>(
    obj: &mut Value,
    context: &ParseContext<'_>,
    name: N,
) -> Result<T>
where
    T: DeserializeOwned + Serialize + Debug + Clone,
    N: Index + Display,
{
    must_translate_with_config(obj, context.map(), context.map_eval(), name)
}

#[inline]
pub fn must_translate_with_config<T, N>(
    obj: &mut Value,
    map: &Map,
    map_eval: &MapEvalConfig,
    name: N,
) -> Result<T>
where
    T: DeserializeOwned + Serialize + Debug + Clone,
    N: Index + Display,
{
    take_optional(obj, map, map_eval, &name)?.ok_or_else(|| anyhow!("missing `{}` field", name))
}

#[inline]
fn take_optional<T, N>(
    obj: &mut Value,
    map: &Map,
    map_eval: &MapEvalConfig,
    name: &N,
) -> Result<Option<T>>
where
    T: DeserializeOwned + Serialize + Debug + Clone,
    N: Index + Display,
{
    let Some(value) = obj.get_mut(name) else {
        return Ok(None);
    };

    serde_json::from_value::<DataPack<T>>(value.take())
        .with_context(|| format!("failed to deserialize field '{}'", name))?
        .unwrap(map, map_eval)
        .with_context(|| format!("failed to unwrap DataPack for field '{}'", name))
        .map(Some)
}

#[inline]
fn try_deser<T: DeserializeOwned>(value: Value, context: &'static str) -> Result<T> {
    serde_json::from_value(value).context(context)
}

#[inline]
fn json_number(value: f64) -> Result<Value> {
    serde_json::Number::from_f64(value)
        .map(Value::Number)
        .ok_or_else(|| anyhow!("failed to create JSON number from {}", value))
}

#[inline]
fn usize_count(count: u64, field: &str) -> Result<usize> {
    usize::try_from(count).with_context(|| format!("{} is too large for this platform", field))
}

fn value_to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64(),
        Value::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

fn lookup_path<'a>(map: &'a Map, name: &str) -> Option<&'a Value> {
    let path = name.strip_prefix("map.").unwrap_or(name);
    let mut parts = path.split('.');
    let first = parts.next()?;
    let mut value = map.get(first)?;

    for part in parts {
        match value {
            Value::Object(obj) => value = obj.get(part)?,
            Value::Array(arr) => {
                let idx = part.parse::<usize>().ok()?;
                value = arr.get(idx)?;
            }
            _ => return None,
        }
    }

    Some(value)
}

fn map_value<'a>(map: &'a Map, index: &str) -> Result<&'a Value> {
    lookup_path(map, index).ok_or_else(|| anyhow!("missing map value `{}`", index))
}

struct MapNamespace<'a> {
    map: &'a Map,
    vars: &'a Vars,
    scope: &'a MapNamespaceScope,
}

impl<'a> MapNamespace<'a> {
    fn new(map: &'a Map, vars: &'a Vars, scope: &'a MapNamespaceScope) -> Self {
        Self { map, vars, scope }
    }

    fn lookup_map_value(&self, name: &str) -> Option<f64> {
        match self.scope {
            MapNamespaceScope::FullMap => lookup_path(self.map, name).and_then(value_to_f64),
            MapNamespaceScope::ExportsOnly => {
                let path = if name.starts_with("map.exports.") {
                    name.to_owned()
                } else {
                    format!("exports.{name}")
                };
                lookup_path(self.map, &path).and_then(value_to_f64)
            }
        }
    }
}

impl EvalNamespace for MapNamespace<'_> {
    fn lookup(&mut self, name: &str, _args: Vec<f64>, _keybuf: &mut String) -> Option<f64> {
        self.vars
            .get(name)
            .copied()
            .or_else(|| self.lookup_map_value(name))
    }
}

struct EvalContext {
    parser: Parser,
    slab: Slab,
    config: MapEvalConfig,
}

impl EvalContext {
    fn new(config: &MapEvalConfig) -> Self {
        Self {
            parser: Parser::new(),
            slab: Slab::new(),
            config: config.clone(),
        }
    }

    fn eval(&mut self, expr: &str, map: &Map, vars: &Vars) -> Result<f64> {
        let mut namespace = MapNamespace::new(map, vars, &self.config.namespace_scope);

        if !self.config.enabled || !self.config.compile_expressions {
            let expr_ref = self
                .parser
                .parse(expr, &mut self.slab.ps)
                .with_context(|| format!("failed to parse expression `{}`", expr))?
                .from(&self.slab.ps);

            return expr_ref
                .eval(&self.slab, &mut namespace)
                .with_context(|| format!("failed to evaluate expression `{}`", expr));
        }

        let expr_ref = self
            .parser
            .parse(expr, &mut self.slab.ps)
            .with_context(|| format!("failed to parse expression `{}`", expr))?
            .from(&self.slab.ps);

        let compiled = expr_ref.compile(&self.slab.ps, &mut self.slab.cs);
        Ok(fasteval::eval_compiled!(
            compiled,
            &self.slab,
            &mut namespace
        ))
    }
}

impl<T> DataPack<T>
where
    T: DeserializeOwned + Serialize + Debug + Clone,
{
    pub fn unwrap(self, map: &Map, map_eval: &MapEvalConfig) -> Result<T> {
        match self {
            Self::Data(data) => Ok(data),
            Self::Index(index) => {
                let value = map_value(map, &index)?.clone();
                match serde_json::from_value::<T>(value.clone()) {
                    Ok(data) => Ok(data),
                    Err(_) => Self::parse_value(map, value, map_eval)
                        .with_context(|| format!("failed to resolve map value `{}`", index)),
                }
            }
        }
    }

    pub fn unwrap_from_context(self, context: &ParseContext<'_>) -> Result<T> {
        self.unwrap(context.map(), context.map_eval())
    }

    fn parse_value(map: &Map, value: Value, map_eval: &MapEvalConfig) -> Result<T> {
        if value.is_object() && value.get("type").is_some() {
            Self::parse_map(map, value, map_eval)
        } else {
            serde_json::from_value(value).context("failed to deserialize value")
        }
    }

    fn parse_map(map: &Map, mut value: Value, map_eval: &MapEvalConfig) -> Result<T> {
        let value_type = value
            .get("type")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("`type` must be a string"))?
            .to_owned();

        let mut eval = EvalContext::new(map_eval);
        let vars = Vars::new();
        let context = ParseContext::new(map, map_eval);

        match value_type.as_str() {
            "raw" => must_translate_with_config(&mut value, map, map_eval, "data"),
            "g-number" => parse_g_number(&context, &mut value, &mut eval, &vars),
            "g-number-list" => parse_g_number_list(&context, &mut value, &mut eval),
            "g-range" => parse_g_range(&context, &mut value),
            "g-repeat" => parse_g_repeat(&context, &mut value),
            "g-linear" => parse_g_linear(&context, &mut value),
            "if" => parse_if(&context, &mut value, &mut eval, &vars),
            #[cfg(feature = "map-parser-extensions")]
            "time" => parse_time(&context, &mut value),
            #[cfg(feature = "map-parser-extensions")]
            "g-random" => parse_g_random(&context, &mut value),
            #[cfg(feature = "map-parser-extensions")]
            "g-choose" => parse_g_choose(&context, &mut value),
            "g-env" => parse_g_env(&context, &mut value),
            "g-join" => parse_g_join(&context, &mut value),
            _ => Err(anyhow!("unknown type `{}`", value_type)),
        }
    }
}

fn parse_g_number<T>(
    context: &ParseContext<'_>,
    value: &mut Value,
    eval: &mut EvalContext,
    vars: &Vars,
) -> Result<T>
where
    T: DeserializeOwned,
{
    let expr: String = must_translate_from_context(value, context, "expr")?;
    try_deser(
        json_number(eval.eval(&expr, context.map(), vars)?)?,
        "failed to deserialize generated number",
    )
}

fn parse_g_number_list<T>(
    context: &ParseContext<'_>,
    value: &mut Value,
    eval: &mut EvalContext,
) -> Result<T>
where
    T: DeserializeOwned,
{
    let index_begin: u64 = must_translate_from_context(value, context, "begin")?;
    let index_end: u64 = must_translate_from_context(value, context, "end")?;
    let expr: String = must_translate_from_context(value, context, "expr")?;

    let len = index_end.saturating_sub(index_begin);
    let mut result = Vec::with_capacity(usize_count(len, "g-number-list length")?);
    let mut vars = Vars::new();

    let compiled = eval
        .parser
        .parse(&expr, &mut eval.slab.ps)
        .with_context(|| format!("failed to parse expression `{}`", expr))?
        .from(&eval.slab.ps)
        .compile(&eval.slab.ps, &mut eval.slab.cs);

    for i in index_begin..index_end {
        vars.insert("i".to_owned(), i as f64);
        let mut namespace = MapNamespace::new(context.map(), &vars, &eval.config.namespace_scope);
        let value = if eval.config.enabled && eval.config.compile_expressions {
            fasteval::eval_compiled!(compiled, &eval.slab, &mut namespace)
        } else {
            eval.eval(&expr, context.map(), &vars)?
        };
        result.push(json_number(value)?);
    }

    try_deser(
        Value::Array(result),
        "failed to deserialize generated number list",
    )
}

fn parse_g_range<T>(context: &ParseContext<'_>, value: &mut Value) -> Result<T>
where
    T: DeserializeOwned,
{
    let begin: f64 = must_translate_from_context(value, context, "begin")?;
    let end: f64 = must_translate_from_context(value, context, "end")?;
    let step: f64 =
        take_optional(value, context.map(), context.map_eval(), &"step")?.unwrap_or(1.0);

    if step <= 0.0 {
        return Err(anyhow!("step must be positive"));
    }

    let capacity = if end > begin {
        ((end - begin) / step).ceil() as usize
    } else {
        0
    };
    let mut result = Vec::with_capacity(capacity);
    let mut current = begin;

    while current < end {
        result.push(json_number(current)?);
        current += step;
    }

    try_deser(
        Value::Array(result),
        "failed to deserialize generated range",
    )
}

fn parse_g_repeat<T>(context: &ParseContext<'_>, value: &mut Value) -> Result<T>
where
    T: DeserializeOwned + Serialize + Debug + Clone,
{
    let val: Value = must_translate_from_context(value, context, "value")?;
    let count: u64 = must_translate_from_context(value, context, "count")?;
    let count = usize_count(count, "count")?;
    let result = vec![val; count];
    try_deser(
        Value::Array(result),
        "failed to deserialize repeated values",
    )
}

fn parse_g_linear<T>(context: &ParseContext<'_>, value: &mut Value) -> Result<T>
where
    T: DeserializeOwned,
{
    let begin: f64 = must_translate_from_context(value, context, "begin")?;
    let end: f64 = must_translate_from_context(value, context, "end")?;
    let count: u64 = must_translate_from_context(value, context, "count")?;

    if count == 0 {
        return Err(anyhow!("count must be positive"));
    }

    let count_usize = usize_count(count, "count")?;
    let mut result = Vec::with_capacity(count_usize);

    if count == 1 {
        result.push(json_number(begin)?);
    } else {
        let step = (end - begin) / ((count - 1) as f64);
        for i in 0..count {
            result.push(json_number(begin + (i as f64) * step)?);
        }
    }

    try_deser(
        Value::Array(result),
        "failed to deserialize linear spaced values",
    )
}

fn parse_if<T>(
    context: &ParseContext<'_>,
    value: &mut Value,
    eval: &mut EvalContext,
    vars: &Vars,
) -> Result<T>
where
    T: DeserializeOwned + Serialize + Debug + Clone,
{
    let condition: String = must_translate_from_context(value, context, "condition")?;
    let true_val: Value = must_translate_from_context(value, context, "true")?;
    let false_val: Value = must_translate_from_context(value, context, "false")?;
    let selected = if eval.eval(&condition, context.map(), vars)? != 0.0 {
        true_val
    } else {
        false_val
    };

    DataPack::<T>::parse_value(context.map(), selected, &eval.config)
}

#[cfg(feature = "map-parser-extensions")]
fn parse_time<T>(context: &ParseContext<'_>, value: &mut Value) -> Result<T>
where
    T: DeserializeOwned,
{
    let start: String = must_translate_from_context(value, context, "start")?;
    let end: String = must_translate_from_context(value, context, "end")?;
    let interval: String = must_translate_from_context(value, context, "interval")?;
    let format: Option<String> =
        take_optional(value, context.map(), context.map_eval(), &"format")?;

    let start_dt = parse_time_str(&start)?;
    let end_dt = parse_time_str(&end)?;
    let step = parse_duration_str(&interval)?;

    if step <= TimeDelta::zero() {
        return Err(anyhow!("interval must be positive"));
    }

    let mut result = Vec::new();
    let mut current = start_dt;

    while current <= end_dt {
        let ts = format.as_ref().map_or_else(
            || current.to_rfc3339(),
            |fmt| current.format(fmt).to_string(),
        );
        result.push(Value::String(ts));

        let Some(next) = current.checked_add_signed(step) else {
            break;
        };
        current = next;
    }

    try_deser(Value::Array(result), "failed to deserialize time values")
}

#[cfg(feature = "map-parser-extensions")]
fn parse_g_random<T>(context: &ParseContext<'_>, value: &mut Value) -> Result<T>
where
    T: DeserializeOwned,
{
    let min: f64 = must_translate_from_context(value, context, "min")?;
    let max: f64 = must_translate_from_context(value, context, "max")?;

    if min >= max {
        return Err(anyhow!("min ({}) must be less than max ({})", min, max));
    }

    let integer = value
        .get("integer")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let seed: Option<u64> = take_optional(value, context.map(), context.map_eval(), &"seed")?;
    let count: Option<u64> = take_optional(value, context.map(), context.map_eval(), &"count")?;

    let gen_value = |rng: &mut dyn Rng| -> Result<Value> {
        if integer {
            if min.fract() != 0.0 || max.fract() != 0.0 {
                return Err(anyhow!("integer random bounds must be whole numbers"));
            }
            Ok(Value::from(rng.random_range(min as i64..max as i64)))
        } else {
            json_number(rng.random_range(min..max))
        }
    };

    match count {
        Some(0) => Err(anyhow!("count must be positive")),
        Some(count) => {
            let count = usize_count(count, "count")?;
            let values = with_rng(seed, |rng| {
                (0..count)
                    .map(|_| gen_value(rng))
                    .collect::<Result<Vec<_>>>()
            })?;
            try_deser(Value::Array(values), "failed to deserialize g-random array")
        }
        None => {
            let value = with_rng(seed, |rng| gen_value(rng))?;
            try_deser(value, "failed to deserialize g-random single value")
        }
    }
}

#[cfg(feature = "map-parser-extensions")]
fn parse_g_choose<T>(context: &ParseContext<'_>, value: &mut Value) -> Result<T>
where
    T: DeserializeOwned + Serialize + Debug + Clone,
{
    let options: Vec<Value> = must_translate_from_context(value, context, "options")?;
    if options.is_empty() {
        return Err(anyhow!("options must not be empty for g-choose"));
    }

    let seed: Option<u64> = take_optional(value, context.map(), context.map_eval(), &"seed")?;
    let count: Option<u64> = take_optional(value, context.map(), context.map_eval(), &"count")?;

    match count {
        Some(0) => Err(anyhow!("count must be positive")),
        Some(count) => {
            let count = usize_count(count, "count")?;
            let selected = with_rng(seed, |rng| {
                (0..count)
                    .map(|_| options[rng.random_range(0..options.len())].clone())
                    .collect::<Vec<_>>()
            });
            try_deser(
                Value::Array(selected),
                "failed to deserialize g-choose array",
            )
        }
        None => {
            let picked = with_rng(seed, |rng| {
                options[rng.random_range(0..options.len())].clone()
            });
            try_deser(picked, "failed to deserialize g-choose single value")
        }
    }
}

fn parse_g_env<T>(context: &ParseContext<'_>, value: &mut Value) -> Result<T>
where
    T: DeserializeOwned,
{
    let name: String = must_translate_from_context(value, context, "name")?;
    let default: Option<String> =
        take_optional(value, context.map(), context.map_eval(), &"default")?;
    let env_val = std::env::var(&name).ok().or(default).ok_or_else(|| {
        anyhow!(
            "environment variable '{}' is not set and no default provided",
            name
        )
    })?;

    try_deser(Value::String(env_val), "failed to deserialize env value")
}

fn parse_g_join<T>(context: &ParseContext<'_>, value: &mut Value) -> Result<T>
where
    T: DeserializeOwned,
{
    let values: Vec<String> = must_translate_from_context(value, context, "values")?;
    let separator: String =
        take_optional(value, context.map(), context.map_eval(), &"separator")?.unwrap_or_default();
    try_deser(
        Value::String(values.join(&separator)),
        "failed to deserialize joined string",
    )
}

#[cfg(feature = "map-parser-extensions")]
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
            continue;
        }

        if ch.is_whitespace() {
            continue;
        }

        if !ch.is_alphabetic() {
            return Err(anyhow!("unexpected character '{}' in duration string", ch));
        }

        if num_str.is_empty() {
            return Err(anyhow!("missing number before duration unit '{}'", ch));
        }

        let num: f64 = num_str
            .parse()
            .with_context(|| format!("invalid number in duration: '{}'", num_str))?;
        if num <= 0.0 {
            return Err(anyhow!("duration components must be positive"));
        }
        num_str.clear();

        let seconds = match ch.to_ascii_lowercase() {
            's' => num,
            'm' => num * 60.0,
            'h' => num * 3_600.0,
            'd' => num * 86_400.0,
            'w' => num * 604_800.0,
            other => return Err(anyhow!("unknown duration unit: '{}'", other)),
        };

        let delta = TimeDelta::try_seconds(seconds as i64)
            .ok_or_else(|| anyhow!("duration overflow: {}{}", num, ch))?;
        total = total
            .checked_add(&delta)
            .ok_or_else(|| anyhow!("duration overflow"))?;
    }

    if !num_str.is_empty() {
        return Err(anyhow!("trailing number without unit: '{}'", num_str));
    }
    if total.is_zero() {
        return Err(anyhow!("duration must be positive, got: '{}'", s));
    }

    Ok(total)
}

#[cfg(feature = "map-parser-extensions")]
fn parse_time_str(s: &str) -> Result<DateTime<Utc>> {
    let s = s.trim();

    if let Some(rest) = s.strip_prefix("now") {
        let base = Utc::now();
        if rest.is_empty() {
            return Ok(base);
        }

        let sign_char = rest
            .chars()
            .next()
            .ok_or_else(|| anyhow!("expected '+' or '-' after 'now'"))?;
        let duration_str = &rest[sign_char.len_utf8()..];
        let delta = parse_duration_str(duration_str)?;

        return match sign_char {
            '+' => base
                .checked_add_signed(delta)
                .ok_or_else(|| anyhow!("time overflow for '{}'", s)),
            '-' => base
                .checked_sub_signed(delta)
                .ok_or_else(|| anyhow!("time overflow for '{}'", s)),
            _ => Err(anyhow!("expected '+' or '-' after 'now', got '{}'", rest)),
        };
    }

    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }

    for fmt in [
        "%Y-%m-%dT%H:%M:%S%.f%:z",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S%:z",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S",
    ] {
        if let Ok(dt) = DateTime::parse_from_str(s, fmt) {
            return Ok(dt.with_timezone(&Utc));
        }
        if let Ok(naive) = NaiveDateTime::parse_from_str(s, fmt) {
            return Ok(naive.and_utc());
        }
    }

    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return date
            .and_hms_opt(0, 0, 0)
            .map(|s| s.and_utc())
            .ok_or_else(|| anyhow!("invalid date: '{}'", s));
    }

    Err(anyhow!(
        "unable to parse time string: '{}'. Supported formats: RFC 3339, \
         'YYYY-MM-DDTHH:MM:SS', 'YYYY-MM-DD HH:MM:SS', 'YYYY-MM-DD', \
         'now', 'now+duration', 'now-duration'",
        s
    ))
}

#[cfg(feature = "map-parser-extensions")]
fn with_rng<F, R>(seed: Option<u64>, f: F) -> R
where
    F: FnOnce(&mut dyn Rng) -> R,
{
    if let Some(seed) = seed {
        let mut rng = StdRng::seed_from_u64(seed);
        f(&mut rng)
    } else {
        let mut rng = rand::rng();
        f(&mut rng)
    }
}

impl<'de, T> Deserialize<'de> for DataPack<T>
where
    T: DeserializeOwned + Serialize + Debug + Clone,
{
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        if let Some(index) = value.as_str().and_then(|s| s.strip_prefix("map.")) {
            return Ok(Self::Index(index.to_owned()));
        }

        serde_json::from_value::<T>(value)
            .map(Self::Data)
            .map_err(serde::de::Error::custom)
    }
}

impl<T> Serialize for DataPack<T>
where
    T: DeserializeOwned + Serialize + Debug + Clone,
{
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
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
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Color {
    NamedColor(color::NamedColor),
    RgbColor(color::Rgb),
    RgbaColor(color::Rgba),
}

impl color::Color for Color {}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;

        if let Some(s) = value.as_str()
            && let Ok(named) = serde_json::from_str::<color::NamedColor>(&format!("\"{s}\""))
        {
            return Ok(Self::NamedColor(named));
        }

        if let Some(s) = value.as_str()
            && let Some(rgb) = parse_hex_color(s)
        {
            return Ok(Self::RgbColor(rgb));
        }

        if let Ok(rgb) = serde_json::from_value::<color::Rgb>(value.clone()) {
            return Ok(Self::RgbColor(rgb));
        }

        if let Ok(rgba) = serde_json::from_value::<color::Rgba>(value) {
            return Ok(Self::RgbaColor(rgba));
        }

        Err(serde::de::Error::custom("invalid color format"))
    }
}

fn parse_hex_color(value: &str) -> Option<color::Rgb> {
    let hex = value.strip_prefix('#')?;

    let (r, g, b) = match hex.len() {
        3 => {
            let mut chars = hex.chars();
            let r = chars.next()?;
            let g = chars.next()?;
            let b = chars.next()?;
            let rr = u8::from_str_radix(&format!("{r}{r}"), 16).ok()?;
            let gg = u8::from_str_radix(&format!("{g}{g}"), 16).ok()?;
            let bb = u8::from_str_radix(&format!("{b}{b}"), 16).ok()?;
            (rr, gg, bb)
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            (r, g, b)
        }
        _ => return None,
    };

    Some(color::Rgb::new(r, g, b))
}
