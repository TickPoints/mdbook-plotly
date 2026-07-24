use crate::preprocessor::config::MapNamespaceScope;
use fasteval::EvalNamespace;
use serde_json::{Map as JsonMap, Value};
use std::collections::BTreeMap;

pub type Map = JsonMap<String, Value>;
pub(crate) type Vars = BTreeMap<String, f64>;

pub(crate) fn lookup_path<'a>(map: &'a Map, name: &str) -> Option<&'a Value> {
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

pub(crate) fn map_value<'a>(map: &'a Map, index: &str) -> anyhow::Result<&'a Value> {
    lookup_path(map, index).ok_or_else(|| anyhow::anyhow!("missing map value `{}`", index))
}

fn value_to_f64(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64(),
        Value::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

pub(crate) struct MapNamespace<'a> {
    map: &'a Map,
    vars: &'a Vars,
    scope: &'a MapNamespaceScope,
}

impl<'a> MapNamespace<'a> {
    pub(crate) fn new(map: &'a Map, vars: &'a Vars, scope: &'a MapNamespaceScope) -> Self {
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
