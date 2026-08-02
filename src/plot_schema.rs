//! The plot generator input schema.
//!
//! `docs/PLOT-SCHEMA.json` (and its `-zh_CN` translation) defines, for
//! each plot type, the fields a user fills in a questionnaire-style form,
//! plus a ready-made example. This module parses that schema, turns form
//! inputs into a `serde_json::Value` plot configuration, and turns an
//! example configuration back into prefilled form inputs. It is compiled
//! only with the `tui` feature (the preprocessor never needs it).
//!
//! The schema describes two kinds of fields:
//! - **trace fields** (shared, top-level `trace_fields`): relative paths
//!   that repeat for every entry in `data`. A plot may have any number of
//!   traces, each filled from the same questionnaire.
//! - **global fields** (per-type `fields` plus the shared `global_fields`
//!   `config` / `map`): absolute paths applied once to the generated plot
//!   object.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Schema format version. Bump when the schema grammar changes.
pub const PLOT_SCHEMA_VERSION: &str = "2.0";

/// Embedded English schema.
pub const EMBEDDED_EN: &str = include_str!("../docs/PLOT-SCHEMA.json");
/// Embedded Chinese schema.
pub const EMBEDDED_ZH_CN: &str = include_str!("../docs/PLOT-SCHEMA-zh_CN.json");

/// A parsed plot generator schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotSchema {
    pub schema: String,
    /// Shared per-trace questionnaire, repeated for every entry in `data`.
    #[serde(default)]
    pub trace_fields: Vec<FieldSchema>,
    /// Shared global fields (`config`, `map`) appended to every plot type.
    #[serde(default)]
    pub global_fields: Vec<FieldSchema>,
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
    /// Base trace header (type / mode / yaxis) applied to every `data`
    /// entry; per-trace fields may override it.
    #[serde(default)]
    pub trace_defaults: Option<TraceSpec>,
    /// Names of [`PlotSchema::trace_fields`] that must be filled for each
    /// trace (e.g. `["x", "y"]` for a line chart).
    #[serde(default)]
    pub required_data: Vec<String>,
    /// Global (absolute-path) fields specific to this plot type.
    pub fields: Vec<FieldSchema>,
    /// A complete example configuration used to prefill the form.
    #[serde(default)]
    pub example: Option<Value>,
}

/// A trace header injected into every generated `data` entry.
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
    /// Free-form JSON5 (used for `config` and `map`).
    Json,
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
    /// Raw text for `String` / `Number` / `Array` / `Array2d` / `Json`.
    pub text: String,
    /// Current value for `Bool`.
    pub bool_value: bool,
    /// Selected option index for `Enum`.
    pub enum_index: usize,
}

/// Per-field validation errors, index-parallel to a list of fields.
pub type FieldErrors = Vec<Option<String>>;
/// Validation errors for every trace, `[trace][field]`.
pub type TraceErrors = Vec<FieldErrors>;

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

/// Prefill one trace from a `data[i]` object, falling back to defaults.
pub fn prefill_trace(trace_fields: &[FieldSchema], trace_obj: &Value) -> Vec<FieldInput> {
    trace_fields
        .iter()
        .map(|field| {
            let found = field
                .targets()
                .into_iter()
                .find_map(|target| read_path(trace_obj, target));
            match found {
                Some(value) => value_to_input(field, value),
                None => default_input(field),
            }
        })
        .collect()
}

/// Prefill the global fields (per-type + shared `config`/`map`) from an
/// example plot object, falling back to defaults.
pub fn prefill_globals(globals: &[FieldSchema], example: Option<&Value>) -> Vec<FieldInput> {
    globals
        .iter()
        .map(|field| {
            let found = example.and_then(|root| {
                field
                    .targets()
                    .into_iter()
                    .find_map(|target| read_path(root, target))
            });
            match found {
                Some(value) => value_to_input(field, value),
                None => default_input(field),
            }
        })
        .collect()
}

/// Prefill all traces and the global fields from a plot type's example.
/// Returns `(traces, globals)`, where `traces` has one entry per `data`
/// element in the example (at least one) and `globals` is index-parallel to
/// [`composite_globals`].
pub fn prefill(
    schema: &PlotSchema,
    plot_type: &PlotTypeSchema,
) -> (Vec<Vec<FieldInput>>, Vec<FieldInput>) {
    let example = plot_type.example.as_ref();
    let data = example
        .and_then(|e| e.get("data"))
        .and_then(Value::as_array);
    let trace_count = data.map(|a| a.len().max(1)).unwrap_or(1);
    let traces = (0..trace_count)
        .map(|i| {
            let trace_obj = data.and_then(|a| a.get(i)).cloned().unwrap_or(Value::Null);
            prefill_trace(&schema.trace_fields, &trace_obj)
        })
        .collect();
    let globals = prefill_globals(&composite_globals(schema, plot_type), example);
    (traces, globals)
}

/// The shared global fields (`config`, `map`) appended to a plot type's own
/// global fields, in display/build order.
pub fn composite_globals(schema: &PlotSchema, plot_type: &PlotTypeSchema) -> Vec<FieldSchema> {
    let mut fields = plot_type.fields.clone();
    fields.extend(schema.global_fields.iter().cloned());
    fields
}

/// Build the plot configuration from form inputs. `traces` is index-parallel
/// to the shared [`PlotSchema::trace_fields`] and has one entry per `data`
/// element; `globals` is index-parallel to [`composite_globals`]. Returns the
/// generated JSON value and parallel error lists (trace errors are
/// `[trace][field]`, global errors are `[field]`).
pub fn build_config(
    schema: &PlotSchema,
    plot_type: &PlotTypeSchema,
    traces: &[Vec<FieldInput>],
    globals: &[FieldInput],
) -> (Value, TraceErrors, FieldErrors) {
    let mut root = Value::Object(serde_json::Map::new());
    let trace_fields = &schema.trace_fields;

    let mut trace_errors = Vec::with_capacity(traces.len());
    let mut data = Vec::with_capacity(traces.len());
    for trace_inputs in traces {
        let mut trace_obj = Value::Object(trace_defaults_object(plot_type.trace_defaults.as_ref()));
        let mut errors = Vec::with_capacity(trace_fields.len());
        for (i, field) in trace_fields.iter().enumerate() {
            let input = trace_inputs.get(i).cloned().unwrap_or_default();
            let (value, mut error) = field_value(field, &input);
            if plot_type.required_data.iter().any(|r| r == &field.name)
                && input.text.trim().is_empty()
                && value.is_none()
            {
                error = Some("required".to_string());
            }
            errors.push(error);
            if let Some(value) = value {
                if is_default_sentinel(field, &input) {
                    continue;
                }
                for target in field.targets() {
                    set_path(&mut trace_obj, target, value.clone());
                }
            }
        }
        trace_errors.push(errors);
        data.push(trace_obj);
    }
    if !data.is_empty() {
        set_path(
            &mut root,
            &[PathSeg::Key("data".into())],
            Value::Array(data),
        );
    }

    let global_fields = composite_globals(schema, plot_type);
    let mut global_errors = Vec::with_capacity(global_fields.len());
    for (i, field) in global_fields.iter().enumerate() {
        let input = globals.get(i).cloned().unwrap_or_default();
        let (value, error) = field_value(field, &input);
        global_errors.push(error);
        if let Some(value) = value {
            for target in field.targets() {
                set_path(&mut root, target, value.clone());
            }
        }
    }

    (root, trace_errors, global_errors)
}

fn trace_defaults_object(trace_defaults: Option<&TraceSpec>) -> serde_json::Map<String, Value> {
    let mut obj = serde_json::Map::new();
    if let Some(defaults) = trace_defaults {
        if !defaults.r#type.is_empty() {
            obj.insert("type".into(), Value::String(defaults.r#type.clone()));
        }
        if let Some(mode) = &defaults.mode {
            obj.insert("mode".into(), Value::String(mode.clone()));
        }
        if let Some(yaxis) = &defaults.yaxis {
            obj.insert("yaxis".into(), Value::String(yaxis.clone()));
        }
    }
    obj
}

/// A trace field whose enum picked the `default` sentinel inherits the
/// plot type's `trace_defaults` instead of writing an override.
fn is_default_sentinel(field: &FieldSchema, input: &FieldInput) -> bool {
    field.kind == FieldType::Enum
        && field
            .options
            .get(input.enum_index)
            .map(|o| o.value == "default")
            .unwrap_or(false)
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
        FieldType::Json => {
            let text = input.text.trim();
            if text.is_empty() {
                return if field.required {
                    (None, Some("required".to_string()))
                } else {
                    (None, None)
                };
            }
            match json5::from_str::<Value>(text) {
                Ok(value) if value.is_object() => (Some(value), None),
                Ok(_) => (None, Some("must be a JSON object".to_string())),
                Err(e) => (None, Some(format!("invalid JSON: {e}"))),
            }
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
        FieldType::Json => FieldInput {
            text: serde_json::to_string_pretty(value).unwrap_or_default(),
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
