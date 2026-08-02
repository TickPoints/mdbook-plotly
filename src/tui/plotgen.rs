//! Plot generator logic: schema-driven form state, validation, and JSON /
//! TOML output. The view layer (`plotgen_view`) renders this state; this
//! module is pure and unit-testable without a terminal.
//!
//! The schema (`docs/PLOT-SCHEMA.json`, or its `-zh_CN` translation chosen
//! by [`crate::tui::locale`]) is embedded at compile time, so the generator
//! works fully offline with no network dependency.

use crate::plot_schema::{
    FieldInput, FieldSchema, FieldType, PlotSchema, PlotTypeSchema, build_config,
    composite_globals, config_to_json, config_to_toml, default_input, has_errors, prefill,
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
///
/// The form is a flat, scrollable list whose rows are the active trace's
/// [`PlotSchema::trace_fields`] followed by the global fields (the plot
/// type's own fields plus the shared `config` / `map`). `selected` indexes
/// into that flat list; [`PlotGen::is_trace_row`] maps an index to either a
/// trace field or a global field.
#[derive(Debug, Clone)]
pub struct PlotGen {
    pub schema: PlotSchema,
    pub lang: DocLang,
    pub type_index: usize,
    /// Composite global fields in display order (type-specific + shared).
    pub global_fields: Vec<FieldSchema>,
    /// Per-trace inputs, index-parallel to `schema.trace_fields`.
    pub trace_inputs: Vec<Vec<FieldInput>>,
    /// Which trace's fields are shown and edited.
    pub active_trace: usize,
    /// Inputs for `global_fields`.
    pub global_inputs: Vec<FieldInput>,
    /// Flat row index into `trace_fields(active_trace) ++ global_fields`.
    pub selected: usize,
    pub output: OutputFormat,
    /// Validation errors, `[trace][field]`.
    pub trace_errors: Vec<Vec<Option<String>>>,
    /// Validation errors for `global_fields`.
    pub global_errors: Vec<Option<String>>,
    /// The last generated configuration text.
    pub generated: String,
    pub status: Option<GenStatus>,
    /// Read by the caller after the loop exits (clipboard fallback).
    pub print_on_exit: Option<String>,
    /// When set, edits do not regenerate until an explicit `regen()`.
    pub no_preview: bool,
    /// Flat row currently being edited inline, if any.
    pub editing: Option<usize>,
    /// Buffer for the active inline edit.
    pub edit_text: String,
    /// Cursor position in `edit_text` (character index).
    pub edit_cursor: usize,
}

impl PlotGen {
    pub fn new(lang: DocLang) -> Self {
        let schema =
            PlotSchema::parse(lang.schema_source()).expect("embedded plot schema must be valid");
        let mut form = Self {
            schema,
            lang,
            type_index: 0,
            global_fields: Vec::new(),
            trace_inputs: Vec::new(),
            active_trace: 0,
            global_inputs: Vec::new(),
            selected: 0,
            output: OutputFormat::Json,
            trace_errors: Vec::new(),
            global_errors: Vec::new(),
            generated: String::new(),
            status: None,
            print_on_exit: None,
            no_preview: false,
            editing: None,
            edit_text: String::new(),
            edit_cursor: 0,
        };
        form.reset_to_example();
        form
    }

    pub fn current_type(&self) -> &PlotTypeSchema {
        &self.schema.plot_types[self.type_index]
    }

    pub fn trace_fields(&self) -> &[FieldSchema] {
        &self.schema.trace_fields
    }

    /// Total rows of the flat form list.
    pub fn display_len(&self) -> usize {
        self.trace_fields().len() + self.global_fields.len()
    }

    /// Whether a flat row index belongs to the active trace's fields.
    pub fn is_trace_row(&self, idx: usize) -> bool {
        idx < self.trace_fields().len()
    }

    /// The field at a flat row index (active trace field or global field).
    pub fn field_at(&self, idx: usize) -> Option<&FieldSchema> {
        if self.is_trace_row(idx) {
            self.trace_fields().get(idx)
        } else {
            self.global_fields.get(idx - self.trace_fields().len())
        }
    }

    /// The field under the cursor.
    pub fn current_field(&self) -> Option<&FieldSchema> {
        self.field_at(self.selected)
    }

    fn input_at(&self, idx: usize) -> Option<FieldInput> {
        if self.is_trace_row(idx) {
            self.trace_inputs
                .get(self.active_trace)
                .and_then(|t| t.get(idx))
                .cloned()
        } else {
            self.global_inputs
                .get(idx - self.trace_fields().len())
                .cloned()
        }
    }

    fn set_input_text(&mut self, idx: usize, text: String) {
        if self.is_trace_row(idx) {
            if let Some(input) = self
                .trace_inputs
                .get_mut(self.active_trace)
                .and_then(|t| t.get_mut(idx))
            {
                input.text = text;
            }
        } else {
            let n = self.trace_fields().len();
            if let Some(input) = self.global_inputs.get_mut(idx - n) {
                input.text = text;
            }
        }
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

    /// (Re)prefill every trace and the global fields from the current
    /// type's example.
    pub fn reset_to_example(&mut self) {
        self.global_fields = composite_globals(&self.schema, self.current_type());
        let (traces, globals) = prefill(&self.schema, self.current_type());
        self.trace_inputs = traces;
        self.global_inputs = globals;
        self.active_trace = 0;
        self.selected = 0;
        self.editing = None;
        self.regen();
    }

    /// Re-run validation and regeneration from the current inputs.
    pub fn regen(&mut self) {
        let (value, trace_errors, global_errors) = build_config(
            &self.schema,
            self.current_type(),
            &self.trace_inputs,
            &self.global_inputs,
        );
        self.trace_errors = trace_errors;
        self.global_errors = global_errors;
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

    /// Replace the text of one field (string / number / array / array2d /
    /// json).
    pub fn set_text_field(&mut self, field_idx: usize, text: String) {
        self.set_input_text(field_idx, text);
        self.regen_if_preview();
    }

    pub fn toggle_bool(&mut self, field_idx: usize) {
        if let Some(input) = self.input_at(field_idx) {
            let value = !input.bool_value;
            if self.is_trace_row(field_idx) {
                if let Some(t) = self
                    .trace_inputs
                    .get_mut(self.active_trace)
                    .and_then(|t| t.get_mut(field_idx))
                {
                    t.bool_value = value;
                }
            } else {
                let n = self.trace_fields().len();
                if let Some(g) = self.global_inputs.get_mut(field_idx - n) {
                    g.bool_value = value;
                }
            }
        }
        self.regen_if_preview();
    }

    pub fn cycle_enum(&mut self, field_idx: usize, delta: isize) {
        let count = self
            .field_at(field_idx)
            .map(|f| f.options.len() as isize)
            .unwrap_or(0);
        if count > 0
            && let Some(input) = self.input_at(field_idx)
        {
            let next = (input.enum_index as isize + delta).rem_euclid(count) as usize;
            if self.is_trace_row(field_idx) {
                if let Some(t) = self
                    .trace_inputs
                    .get_mut(self.active_trace)
                    .and_then(|t| t.get_mut(field_idx))
                {
                    t.enum_index = next;
                }
            } else {
                let n = self.trace_fields().len();
                if let Some(g) = self.global_inputs.get_mut(field_idx - n) {
                    g.enum_index = next;
                }
            }
        }
        self.regen_if_preview();
    }

    /// Append another `data` trace and edit it.
    pub fn add_trace(&mut self) {
        let defaults: Vec<FieldInput> =
            self.schema.trace_fields.iter().map(default_input).collect();
        self.trace_inputs.push(defaults);
        self.active_trace = self.trace_inputs.len() - 1;
        self.selected = 0;
        self.editing = None;
        self.regen_if_preview();
    }

    /// Remove the active `data` trace (never below one).
    pub fn remove_trace(&mut self) {
        if self.trace_inputs.len() > 1 {
            self.trace_inputs.remove(self.active_trace);
            self.active_trace = self.active_trace.min(self.trace_inputs.len() - 1);
            self.editing = None;
            self.regen_if_preview();
        }
    }

    /// Move the active trace by `delta` (-1 / +1), wrapping around.
    pub fn switch_trace(&mut self, delta: isize) {
        let n = self.trace_inputs.len();
        if n > 0 {
            self.active_trace =
                (self.active_trace as isize + delta).rem_euclid(n as isize) as usize;
            self.editing = None;
        }
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
        self.trace_errors.iter().any(|t| has_errors(t)) || has_errors(&self.global_errors)
    }

    /// The error message for a flat row, if any.
    pub fn error_at(&self, idx: usize) -> Option<&str> {
        if self.is_trace_row(idx) {
            self.trace_errors
                .get(self.active_trace)
                .and_then(|t| t.get(idx))
                .and_then(|o| o.as_deref())
        } else {
            self.global_errors
                .get(idx - self.trace_fields().len())
                .and_then(|o| o.as_deref())
        }
    }

    fn first_error_msg(&self) -> String {
        for trace in &self.trace_errors {
            if let Some(msg) = trace.iter().flatten().next() {
                return msg.clone();
            }
        }
        self.global_errors
            .iter()
            .flatten()
            .next()
            .cloned()
            .unwrap_or_else(|| "fix the errors first".to_string())
    }

    /// The display value of a field (text, checkbox, chosen option, or a
    /// placeholder for a JSON object).
    pub fn field_display(&self, field_idx: usize) -> String {
        let Some(field) = self.field_at(field_idx) else {
            return String::new();
        };
        let input = self.input_at(field_idx).unwrap_or_default();
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
            FieldType::Json => {
                if input.text.trim().is_empty() {
                    String::new()
                } else {
                    "{ … }".to_string()
                }
            }
            _ => input.text.clone(),
        }
    }

    pub fn copy(&mut self, clipboard: &mut Option<arboard::Clipboard>) {
        if self.has_errors() {
            self.status = Some(GenStatus::Error(self.first_error_msg()));
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
            self.status = Some(GenStatus::Error(self.first_error_msg()));
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

    // -- inline editing ---------------------------------------------------

    pub fn is_editing(&self) -> bool {
        self.editing.is_some()
    }

    /// Enter inline editing for a text-capable field, seeding the buffer
    /// with the field's raw value.
    pub fn start_edit(&mut self, idx: usize) {
        let Some(field) = self.field_at(idx) else {
            return;
        };
        if field.is_bool() || field.is_enum() {
            return;
        }
        self.edit_text = self.input_at(idx).map(|i| i.text).unwrap_or_default();
        self.edit_cursor = self.edit_text.chars().count();
        self.editing = Some(idx);
    }

    /// Commit the inline edit to the field and regenerate.
    pub fn commit_edit(&mut self) {
        if let Some(idx) = self.editing.take() {
            let text = std::mem::take(&mut self.edit_text);
            self.edit_cursor = 0;
            self.set_text_field(idx, text);
        }
    }

    /// Abandon the inline edit without touching the field.
    pub fn cancel_edit(&mut self) {
        self.editing = None;
        self.edit_text.clear();
        self.edit_cursor = 0;
    }

    fn edit_len(&self) -> usize {
        self.edit_text.chars().count()
    }

    pub fn insert_char(&mut self, c: char) {
        let pos = self.edit_cursor.min(self.edit_len());
        let mut chars: Vec<char> = self.edit_text.chars().collect();
        chars.insert(pos, c);
        self.edit_text = chars.into_iter().collect();
        self.edit_cursor = pos + 1;
    }

    pub fn backspace(&mut self) {
        if self.edit_cursor == 0 {
            return;
        }
        let pos = self.edit_cursor - 1;
        let mut chars: Vec<char> = self.edit_text.chars().collect();
        chars.remove(pos);
        self.edit_text = chars.into_iter().collect();
        self.edit_cursor = pos;
    }

    pub fn delete(&mut self) {
        let len = self.edit_len();
        if self.edit_cursor >= len {
            return;
        }
        let mut chars: Vec<char> = self.edit_text.chars().collect();
        chars.remove(self.edit_cursor);
        self.edit_text = chars.into_iter().collect();
    }

    pub fn cursor_left(&mut self) {
        self.edit_cursor = self.edit_cursor.saturating_sub(1);
    }

    pub fn cursor_right(&mut self) {
        self.edit_cursor = (self.edit_cursor + 1).min(self.edit_len());
    }

    pub fn cursor_home(&mut self) {
        self.edit_cursor = 0;
    }

    pub fn cursor_end(&mut self) {
        self.edit_cursor = self.edit_len();
    }
}
