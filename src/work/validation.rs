//! validation — Schema-driven validation against the JSON Schemas in
//! `schemas/`.
//!
//! IMPLEMENTATION NOTE: this is a real schema validator for the JSON Schema
//! subset this repository authors (type / required / properties / enum /
//! const / minimum / items / `$ref` "#/definitions/X"). Because we author the
//! schemas, we control the subset; a future migration can swap in the
//! `jsonschema` crate without changing the schemas or the tests.

use serde_json::Value;

/// A parsed JSON Schema document with a validation entry point.
///
/// `document` is the full schema file (kept for `$ref` resolution);
/// `entry_pointer` is the JSON Pointer where validation starts (empty for the
/// file root, `/definitions/Work` for a named definition).
#[derive(Debug, Clone)]
pub struct Schema {
    document: Value,
    entry_pointer: String,
}

/// Validation errors, one per violated keyword.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationErrors(pub Vec<String>);

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for err in &self.0 {
            writeln!(f, "- {err}")?;
        }
        Ok(())
    }
}

impl Schema {
    /// Parses a schema document; validation starts at the file root.
    pub fn parse(json: &str) -> Result<Self, String> {
        let document: Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        Ok(Self {
            document,
            entry_pointer: String::new(),
        })
    }

    /// Loads the bundled Work contract (compile-time, from `schemas/`).
    ///
    /// CARGO_MANIFEST_DIR is the root package dir (repo root), so the bundled
    /// schemas resolve to `<repo>/schemas/` for this crate wherever it is
    /// compiled (directly or as a dependency).
    pub fn work() -> Self {
        let json = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/schemas/work.schema.json"
        ));
        Schema::parse(json)
            .expect("bundled work schema must parse")
            .at("/definitions/Work")
    }

    /// Points validation at a named definition inside the document.
    pub fn at(mut self, pointer: &str) -> Self {
        self.entry_pointer = pointer.to_string();
        self
    }

    /// Validates an instance against the entry schema. Returns all violations.
    pub fn validate(&self, instance: &Value) -> Result<(), ValidationErrors> {
        let entry = resolve_pointer(&self.document, &self.entry_pointer).unwrap_or(&self.document);
        let mut errors = Vec::new();
        validate_value(entry, instance, "", &mut errors, &self.document);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(ValidationErrors(errors))
        }
    }
}

fn resolve_pointer<'a>(document: &'a Value, pointer: &str) -> Option<&'a Value> {
    if pointer.is_empty() {
        return Some(document);
    }
    // References arrive as "#/definitions/X" (JSON Schema) while entry
    // pointers arrive as "/definitions/X" (JSON Pointer); accept both.
    let path = pointer
        .strip_prefix("#/")
        .or_else(|| pointer.strip_prefix('/'))?;
    let mut current = document;
    for segment in path.split('/') {
        current = current.get(segment)?;
    }
    Some(current)
}

fn type_name_matches(expected: &str, value: &Value) -> bool {
    match expected {
        "object" => value.is_object(),
        "string" => value.is_string(),
        "integer" => value.is_i64() || value.is_u64(),
        "number" => value.is_number(),
        "array" => value.is_array(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => true,
    }
}

fn type_matches(schema: &Value, value: &Value) -> bool {
    match schema.get("type") {
        Some(Value::String(expected)) => type_name_matches(expected, value),
        Some(Value::Array(variants)) => variants
            .iter()
            .filter_map(|v| v.as_str())
            .any(|variant| type_name_matches(variant, value)),
        _ => true,
    }
}

fn validate_value(
    schema: &Value,
    value: &Value,
    path: &str,
    errors: &mut Vec<String>,
    root: &Value,
) {
    // COMPATIBILITY: this repo's schemas use local "#/definitions/X" refs to
    // stay self-contained per file; resolve them against the full document.
    if let Some(reference) = schema.get("$ref").and_then(|r| r.as_str()) {
        match resolve_pointer(root, reference) {
            Some(resolved) => {
                validate_value(resolved, value, path, errors, root);
                return;
            }
            None => {
                errors.push(format!("{path}: unresolvable $ref {reference}"));
                return;
            }
        }
    }

    if !type_matches(schema, value) {
        let expected = schema
            .get("type")
            .map(|t| t.to_string())
            .unwrap_or_else(|| "any".to_string());
        errors.push(format!("{path}: expected type {expected}"));
        return;
    }

    if let Some(allowed) = schema.get("enum").and_then(|e| e.as_array()) {
        if !allowed.contains(value) {
            errors.push(format!("{path}: {value} not in enum"));
        }
    }

    if let Some(const_value) = schema.get("const") {
        if const_value != value {
            errors.push(format!("{path}: {value} != const {const_value}"));
        }
    }

    if let Some(minimum) = schema.get("minimum").and_then(|m| m.as_f64()) {
        if let Some(number) = value.as_f64() {
            if number < minimum {
                errors.push(format!("{path}: {number} < minimum {minimum}"));
            }
        }
    }

    if let Some(object) = value.as_object() {
        let required = schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for key in required {
            if object.get(&key).is_none() {
                errors.push(format!("{path}: missing required property '{key}'"));
            }
        }
        if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
            for (key, property_schema) in properties {
                if let Some(property_value) = object.get(key) {
                    let property_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{path}.{key}")
                    };
                    validate_value(
                        property_schema,
                        property_value,
                        &property_path,
                        errors,
                        root,
                    );
                }
            }
        }
    }

    if let Some(items) = value.as_array() {
        if let Some(item_schema) = schema.get("items") {
            for (index, item) in items.iter().enumerate() {
                let item_path = format!("{path}[{index}]");
                validate_value(item_schema, item, &item_path, errors, root);
            }
        }
    }
}
