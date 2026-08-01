//! Plot generator logic: schema-driven form state, validation, and JSON /
//! TOML output. The view layer (`plotgen_view`) renders this state; this
//! module is pure and unit-testable without a terminal.
//!
//! The schema (`docs/PLOT-SCHEMA.json`, or its `-zh_CN` translation chosen
//! by [`crate::tui::locale`]) is embedded at compile time, so the generator
//! works fully offline with no network dependency.

use crate::plot_schema::{
    FieldInput, FieldType, PlotSchema, PlotTypeSchema, build_config, config_to_json,
    config_to_toml, has_errors, prefill_inputs,
};
use crate::tui::locale::DocLang;

/// The serialization format of the generated configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Toml,
}

impl OutputFormat {
    pub fn extension(self) -> &'static str {
        match self {
            OutputFormat::Json => "json",
            OutputFormat::Toml => "toml",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            OutputFormat::Json => "JSON",
            OutputFormat::Toml => "TOML",
        }
    }
}

/// One-shot action results shown in the status line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenStatus {
    Copied,
    ClipboardUnavailable,
    Saved(String),
    Error(String),
}

/// The form state for one generator session.
#[derive(Debug, Clone)]
pub struct PlotGen {
    pub schema: PlotSchema,
    pub lang: DocLang,
    pub type_index: usize,
    /// Field values, index-parallel to the current plot type's fields.
    pub inputs: Vec<FieldInput>,
    pub selected: usize,
    pub output: OutputFormat,
    /// Validation errors, index-parallel to the fields.
    pub errors: Vec<Option<String>>,
    /// The last generated configuration text.
    pub generated: String,
    pub status: Option<GenStatus>,
    /// Read by the caller after the loop exits (clipboard fallback).
    pub print_on_exit: Option<String>,
    /// When set, edits do not regenerate until an explicit `regen()`.
    pub no_preview: bool,
}

impl PlotGen {
    pub fn new(lang: DocLang) -> Self {
        let schema =
            PlotSchema::parse(lang.schema_source()).expect("embedded plot schema must be valid");
        let mut form = Self {
            schema,
            lang,
            type_index: 0,
            inputs: Vec::new(),
            selected: 0,
            output: OutputFormat::Json,
            errors: Vec::new(),
            generated: String::new(),
            status: None,
            print_on_exit: None,
            no_preview: false,
        };
        form.reset_to_example();
        form
    }

    pub fn current_type(&self) -> &PlotTypeSchema {
        &self.schema.plot_types[self.type_index]
    }

    /// Switch to another plot type, resetting the form.
    pub fn set_type(&mut self, index: usize) {
        if index >= self.schema.plot_types.len() {
            return;
        }
        self.type_index = index;
        self.status = None;
        self.reset_to_example();
    }

    /// (Re)prefill every field from the current type's example.
    pub fn reset_to_example(&mut self) {
        self.inputs = prefill_inputs(self.current_type());
        self.selected = 0;
        self.regen();
    }

    /// Re-run validation and regeneration from the current inputs.
    pub fn regen(&mut self) {
        let (value, errors) = build_config(self.current_type(), &self.inputs);
        self.errors = errors;
        self.generated = match self.output {
            OutputFormat::Json => config_to_json(&value),
            OutputFormat::Toml => config_to_toml(&value).unwrap_or_else(|e| format!("// {e}")),
        };
    }

    pub fn cycle_output(&mut self) {
        self.output = match self.output {
            OutputFormat::Json => OutputFormat::Toml,
            OutputFormat::Toml => OutputFormat::Json,
        };
        self.regen();
    }

    pub fn current_field(&self) -> Option<&crate::plot_schema::FieldSchema> {
        self.current_type().fields.get(self.selected)
    }

    /// Replace the text of one field (string / number / array / array2d).
    pub fn set_text_field(&mut self, field_idx: usize, text: String) {
        if let Some(input) = self.inputs.get_mut(field_idx) {
            input.text = text;
        }
        self.regen_if_preview();
    }

    pub fn toggle_bool(&mut self, field_idx: usize) {
        if let Some(input) = self.inputs.get_mut(field_idx) {
            input.bool_value = !input.bool_value;
        }
        self.regen_if_preview();
    }

    pub fn cycle_enum(&mut self, field_idx: usize, delta: isize) {
        let count = self.current_type().fields[field_idx].options.len() as isize;
        if count > 0
            && let Some(input) = self.inputs.get_mut(field_idx)
        {
            input.enum_index = (input.enum_index as isize + delta).rem_euclid(count) as usize;
        }
        self.regen_if_preview();
    }

    fn regen_if_preview(&mut self) {
        if !self.no_preview {
            self.regen();
        }
    }

    /// Toggle whether edits regenerate the preview live (`--no-preview`).
    pub fn set_no_preview(&mut self, on: bool) {
        self.no_preview = on;
    }

    pub fn has_errors(&self) -> bool {
        has_errors(&self.errors)
    }

    /// The display value of a field (text, checkbox, or chosen option).
    pub fn field_display(&self, field_idx: usize) -> String {
        let field = &self.current_type().fields[field_idx];
        let input = &self.inputs[field_idx];
        match field.kind {
            FieldType::Bool => {
                if input.bool_value {
                    "[x]".to_string()
                } else {
                    "[ ]".to_string()
                }
            }
            FieldType::Enum => match field.options.get(input.enum_index) {
                Some(option) => option.label.clone(),
                None => String::new(),
            },
            _ => input.text.clone(),
        }
    }

    pub fn copy(&mut self, clipboard: &mut Option<arboard::Clipboard>) {
        if self.has_errors() {
            let msg = self
                .errors
                .iter()
                .flatten()
                .next()
                .cloned()
                .unwrap_or_else(|| "fix the errors first".to_string());
            self.status = Some(GenStatus::Error(msg));
            return;
        }
        let text = self.generated.clone();
        match clipboard {
            Some(clipboard) => match clipboard.set_text(text) {
                Ok(()) => self.status = Some(GenStatus::Copied),
                Err(_) => {
                    self.status = Some(GenStatus::ClipboardUnavailable);
                    self.print_on_exit = Some(self.generated.clone());
                }
            },
            None => {
                self.status = Some(GenStatus::ClipboardUnavailable);
                self.print_on_exit = Some(self.generated.clone());
            }
        }
    }

    /// The file name `save` writes, derived from the type and format.
    pub fn save_file_name(&self) -> String {
        format!(
            "plot-{}.{}",
            self.current_type().id,
            self.output.extension()
        )
    }

    pub fn save(&mut self) {
        if self.has_errors() {
            let msg = self
                .errors
                .iter()
                .flatten()
                .next()
                .cloned()
                .unwrap_or_else(|| "fix the errors first".to_string());
            self.status = Some(GenStatus::Error(msg));
            return;
        }
        let path = std::env::current_dir()
            .unwrap_or_default()
            .join(self.save_file_name());
        match crate::tui::book_toml::atomic_write(&path, &self.generated) {
            Ok(()) => self.status = Some(GenStatus::Saved(path.display().to_string())),
            Err(e) => {
                self.status = Some(GenStatus::Error(format!(
                    "cannot write {}: {e}",
                    path.display()
                )));
            }
        }
    }
}
