//! Contract C: the plot generator input schema.
//!
//! `docs/PLOT-SCHEMA.json` (and its `-zh_CN` translation) defines, for
//! each plot type, the fields a user fills in a questionnaire-style form,
//! plus a ready-made example. This module parses that schema, turns form
//! inputs into a `serde_json::Value` plot configuration, and turns an
//! example configuration back into prefilled form inputs. It is
//! dependency-light (serde + serde_json + toml only) so it compiles and is
//! tested in every feature combination.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Schema format version. Bump when the schema grammar changes.
pub const PLOT_SCHEMA_VERSION: &str = "1.0";

/// Embedded English schema.
pub const EMBEDDED_EN: &str = include_str!("../docs/PLOT-SCHEMA.json");
/// Embedded Chinese schema.
pub const EMBEDDED_ZH_CN: &str = include_str!("../docs/PLOT-SCHEMA-zh_CN.json");

/// A parsed plot generator schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotSchema {
    pub schema: String,
    pub plot_types: Vec<PlotTypeSchema>,
}

impl PlotSchema {
    /// Parse a schema document. Fails on malformed JSON or unknown field
    /// kinds, so a broken schema is caught in CI rather than at runtime.
    pub fn parse(source: &str) -> Result<Self, String> {
        serde_json::from_str(source).map_err(|e| format!("invalid plot schema: {e}"))
    }

    /// Look up a plot type by its stable `id`.
    pub fn find(&self, id: &str) -> Option<&PlotTypeSchema> {
        self.plot_types.iter().find(|t| t.id == id)
    }
}

/// One questionnaire: a plot type and the fields used to describe it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotTypeSchema {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Fixed traces (type / mode / yaxis) emitted before any field values.
    #[serde(default)]
    pub traces: Vec<TraceSpec>,
    /// A complete example configuration used to prefill the form.
    #[serde(default)]
    pub example: Option<Value>,
    pub fields: Vec<FieldSchema>,
}

/// A trace header injected into the generated `data` array.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceSpec {
    pub index: usize,
    pub r#type: String,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub yaxis: Option<String>,
}

/// One fillable field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldSchema {
    pub name: String,
    pub label: String,
    #[serde(rename = "type")]
    pub kind: FieldType,
    /// Item type for `Array` / `Array2d` kinds.
    #[serde(default)]
    pub item_type: Option<ItemType>,
    /// Choices for the `Enum` kind.
    #[serde(default)]
    pub options: Vec<EnumOption>,
    /// Inclusive bounds for the `Number` kind.
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<Value>,
    #[serde(default)]
    pub help: String,
    /// Where the value lands in the generated configuration.
    #[serde(default)]
    pub path: Vec<PathSeg>,
    /// Alternative to `path` when one value lands in several places.
    #[serde(default)]
    pub paths: Vec<Vec<PathSeg>>,
}

impl FieldSchema {
    /// All target paths for this field (`paths` wins over `path`).
    pub fn targets(&self) -> Vec<&[PathSeg]> {
        if !self.paths.is_empty() {
            self.paths.iter().map(|p| p.as_slice()).collect()
        } else {
            vec![self.path.as_slice()]
        }
    }

    pub fn is_bool(&self) -> bool {
        self.kind == FieldType::Bool
    }

    pub fn is_enum(&self) -> bool {
        self.kind == FieldType::Enum
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    String,
    Number,
    Bool,
    Enum,
    Array,
    Array2d,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemType {
    Number,
    String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumOption {
    pub value: String,
    pub label: String,
}

/// One segment of a target path: object key or array index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PathSeg {
    Key(String),
    Index(usize),
}

/// The current value of one form field, keyed by what the field accepts.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FieldInput {
    /// Raw text for `String` / `Number` / `Array` / `Array2d`.
    pub text: String,
    /// Current value for `Bool`.
    pub bool_value: bool,
    /// Selected option index for `Enum`.
    pub enum_index: usize,
}

/// The value a fresh form shows for a field.
pub fn default_input(field: &FieldSchema) -> FieldInput {
    match field.kind {
        FieldType::Bool => FieldInput {
            bool_value: field
                .default
                .as_ref()
                .and_then(Value::as_bool)
                .unwrap_or(false),
            ..FieldInput::default()
        },
        FieldType::Enum => FieldInput {
            enum_index: field
                .default
                .as_ref()
                .and_then(Value::as_str)
                .and_then(|d| field.options.iter().position(|o| o.value == d))
                .unwrap_or(0),
            ..FieldInput::default()
        },
        _ => FieldInput {
            text: default_text(field),
            ..FieldInput::default()
        },
    }
}

/// Prefill the form for a plot type from its embedded example.
pub fn prefill_inputs(plot_type: &PlotTypeSchema) -> Vec<FieldInput> {
    let Some(example) = &plot_type.example else {
        return plot_type.fields.iter().map(default_input).collect();
    };
    plot_type
        .fields
        .iter()
        .map(|field| {
            let found = field
                .targets()
                .into_iter()
                .find_map(|target| read_path(example, target));
            match found {
                Some(value) => value_to_input(field, value),
                None => default_input(field),
            }
        })
        .collect()
}

/// Build the plot configuration from form inputs. Returns the generated
/// JSON value and a per-field error list (index-parallel to `fields`).
pub fn build_config(
    plot_type: &PlotTypeSchema,
    inputs: &[FieldInput],
) -> (Value, Vec<Option<String>>) {
    let mut root = Value::Object(serde_json::Map::new());
    apply_traces(&mut root, &plot_type.traces);
    let mut errors = Vec::with_capacity(plot_type.fields.len());
    for (i, field) in plot_type.fields.iter().enumerate() {
        let input = inputs.get(i).cloned().unwrap_or_default();
        let (value, error) = field_value(field, &input);
        errors.push(error);
        if let Some(value) = value {
            for target in field.targets() {
                set_path(&mut root, target, value.clone());
            }
        }
    }
    (root, errors)
}

/// Whether any of the per-field errors is present.
pub fn has_errors(errors: &[Option<String>]) -> bool {
    errors.iter().any(Option::is_some)
}

/// Serialize a generated configuration as pretty JSON (a valid JSON5
/// document, so it works in a `plot` / `plotly` fenced block as-is).
pub fn config_to_json(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_default()
}

/// Serialize a generated configuration as TOML, for `book.toml`-style
/// embedding. Fails when the value cannot be represented in TOML.
pub fn config_to_toml(value: &Value) -> Result<String, String> {
    toml::to_string(value).map_err(|e| format!("cannot serialize as TOML: {e}"))
}

/// Read the value at a target path.
pub fn read_path<'a>(mut cur: &'a Value, path: &[PathSeg]) -> Option<&'a Value> {
    for seg in path {
        cur = match seg {
            PathSeg::Key(key) => cur.get(key)?,
            PathSeg::Index(index) => cur.as_array()?.get(*index)?,
        };
    }
    Some(cur)
}

fn apply_traces(root: &mut Value, traces: &[TraceSpec]) {
    for trace in traces {
        let mut obj = serde_json::Map::new();
        obj.insert("type".into(), Value::String(trace.r#type.clone()));
        if let Some(mode) = &trace.mode {
            obj.insert("mode".into(), Value::String(mode.clone()));
        }
        if let Some(yaxis) = &trace.yaxis {
            obj.insert("yaxis".into(), Value::String(yaxis.clone()));
        }
        let path = vec![PathSeg::Key("data".into()), PathSeg::Index(trace.index)];
        set_path(root, &path, Value::Object(obj));
    }
}

fn field_value(field: &FieldSchema, input: &FieldInput) -> (Option<Value>, Option<String>) {
    match field.kind {
        FieldType::Bool => (Some(Value::Bool(input.bool_value)), None),
        FieldType::Enum => match field.options.get(input.enum_index) {
            Some(option) => (Some(Value::String(option.value.clone())), None),
            None => (None, Some("invalid choice".to_string())),
        },
        FieldType::String => {
            let text = input.text.trim();
            if text.is_empty() {
                return if field.required {
                    (None, Some("required".to_string()))
                } else {
                    (None, None)
                };
            }
            (Some(Value::String(text.to_string())), None)
        }
        FieldType::Number => {
            let text = input.text.trim();
            if text.is_empty() {
                return if field.required {
                    (None, Some("required".to_string()))
                } else {
                    (None, None)
                };
            }
            match text.parse::<f64>() {
                Ok(number) => {
                    if let Some(min) = field.min
                        && number < min
                    {
                        return (None, Some(format!("must be at least {min}")));
                    }
                    if let Some(max) = field.max
                        && number > max
                    {
                        return (None, Some(format!("must be at most {max}")));
                    }
                    match number_value(text) {
                        Some(num) => (Some(Value::Number(num)), None),
                        None => (None, Some("not a finite number".to_string())),
                    }
                }
                Err(_) => (None, Some(format!("'{text}' is not a number"))),
            }
        }
        FieldType::Array => {
            let text = input.text.trim();
            if text.is_empty() {
                return if field.required {
                    (None, Some("required".to_string()))
                } else {
                    (None, None)
                };
            }
            let items = split_list(text, ',');
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                match parse_item(item, field.item_type.unwrap_or(ItemType::Number)) {
                    Ok(value) => out.push(value),
                    Err(e) => return (None, Some(e)),
                }
            }
            (Some(Value::Array(out)), None)
        }
        FieldType::Array2d => {
            let text = input.text.trim();
            if text.is_empty() {
                return if field.required {
                    (None, Some("required".to_string()))
                } else {
                    (None, None)
                };
            }
            let rows = split_list(text, ';');
            let mut out = Vec::with_capacity(rows.len());
            for row in rows {
                let items = split_list(row, ',');
                let mut row_values = Vec::with_capacity(items.len());
                for item in items {
                    match parse_item(item, field.item_type.unwrap_or(ItemType::Number)) {
                        Ok(value) => row_values.push(value),
                        Err(e) => return (None, Some(format!("row: {e}"))),
                    }
                }
                out.push(Value::Array(row_values));
            }
            (Some(Value::Array(out)), None)
        }
    }
}

fn parse_item(text: &str, item_type: ItemType) -> Result<Value, String> {
    match item_type {
        ItemType::Number => match number_value(text) {
            Some(num) => Ok(Value::Number(num)),
            None => Err(format!("'{text}' is not a number")),
        },
        ItemType::String => Ok(Value::String(text.to_string())),
    }
}

/// Parse a numeric string into a `serde_json::Number`, preferring the
/// integer representation so plotly fields that expect `usize` (e.g.
/// `marker.size`) accept it.
fn number_value(text: &str) -> Option<serde_json::Number> {
    if let Ok(integer) = text.parse::<i64>() {
        return Some(integer.into());
    }
    let number = text.parse::<f64>().ok()?;
    if number.fract() == 0.0 && number.is_finite() {
        return Some((number as u64).into());
    }
    serde_json::Number::from_f64(number)
}

fn split_list(text: &str, separator: char) -> Vec<&str> {
    text.split(separator)
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .collect()
}

/// Set a value at a target path, creating intermediate containers and
/// replacing `null` placeholders as needed.
pub fn set_path(root: &mut Value, path: &[PathSeg], value: Value) {
    if path.is_empty() {
        return;
    }
    let mut cur = root;
    for (i, seg) in path.iter().enumerate() {
        let is_last = i + 1 == path.len();
        match seg {
            PathSeg::Key(key) => {
                let obj = ensure_object(cur);
                if is_last {
                    obj.insert(key.clone(), value);
                    return;
                }
                let entry = obj.entry(key.clone()).or_insert(Value::Null);
                cur = entry;
            }
            PathSeg::Index(index) => {
                let arr = ensure_array(cur);
                while arr.len() <= *index {
                    arr.push(Value::Null);
                }
                if is_last {
                    arr[*index] = value;
                    return;
                }
                let next = arr.get_mut(*index).unwrap();
                if next.is_null() {
                    match &path[i + 1] {
                        PathSeg::Key(_) => *next = Value::Object(serde_json::Map::new()),
                        PathSeg::Index(_) => *next = Value::Array(Vec::new()),
                    }
                }
                cur = next;
            }
        }
    }
}

fn ensure_object(cur: &mut Value) -> &mut serde_json::Map<String, Value> {
    if !cur.is_object() {
        *cur = Value::Object(serde_json::Map::new());
    }
    cur.as_object_mut().unwrap()
}

fn ensure_array(cur: &mut Value) -> &mut Vec<Value> {
    if !cur.is_array() {
        *cur = Value::Array(Vec::new());
    }
    cur.as_array_mut().unwrap()
}

fn default_text(field: &FieldSchema) -> String {
    let Some(value) = &field.default else {
        return String::new();
    };
    match field.kind {
        FieldType::String => value.as_str().unwrap_or_default().to_string(),
        FieldType::Number => value.as_f64().map(|n| n.to_string()).unwrap_or_default(),
        FieldType::Array => array_to_text(value),
        FieldType::Array2d => array2d_to_text(value),
        _ => String::new(),
    }
}

fn value_to_input(field: &FieldSchema, value: &Value) -> FieldInput {
    match field.kind {
        FieldType::Bool => FieldInput {
            bool_value: value.as_bool().unwrap_or(false),
            ..FieldInput::default()
        },
        FieldType::Enum => FieldInput {
            enum_index: value
                .as_str()
                .and_then(|s| field.options.iter().position(|o| o.value == s))
                .unwrap_or(0),
            ..FieldInput::default()
        },
        FieldType::Array => FieldInput {
            text: array_to_text(value),
            ..FieldInput::default()
        },
        FieldType::Array2d => FieldInput {
            text: array2d_to_text(value),
            ..FieldInput::default()
        },
        _ => FieldInput {
            text: scalar_to_text(value),
            ..FieldInput::default()
        },
    }
}

fn scalar_to_text(value: &Value) -> String {
    match value {
        Value::Number(n) => n.as_f64().map(|n| n.to_string()).unwrap_or_default(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

fn array_to_text(value: &Value) -> String {
    value
        .as_array()
        .map(|a| a.iter().map(scalar_to_text).collect::<Vec<_>>().join(", "))
        .unwrap_or_default()
}

fn array2d_to_text(value: &Value) -> String {
    value
        .as_array()
        .map(|rows| {
            rows.iter()
                .map(array_to_text)
                .collect::<Vec<_>>()
                .join("; ")
        })
        .unwrap_or_default()
}
