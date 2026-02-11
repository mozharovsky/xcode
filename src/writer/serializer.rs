use std::collections::HashMap;
use std::fmt::Write as FmtWrite;

use indexmap::IndexMap;

use super::comments::{create_reference_list, is_pbx_build_file, is_pbx_file_reference};
use super::quotes::{add_quotes, ensure_quotes, format_data};
use crate::types::PlistValue;

/// Options for the writer.
#[derive(Debug, Clone)]
pub struct WriterOptions {
    pub tab: String,
    pub shebang: String,
}

impl Default for WriterOptions {
    fn default() -> Self {
        WriterOptions {
            tab: "\t".to_string(),
            shebang: "!$*UTF8*$!".to_string(),
        }
    }
}

/// Serializes a PlistValue (representing a parsed .pbxproj) back to text format.
pub struct Writer {
    buf: String,
    indent: usize,
    comments: HashMap<String, String>,
    options: WriterOptions,
    // Pre-computed indent strings for levels 0..MAX_INDENT
    indents: Vec<String>,
}

const MAX_CACHED_INDENT: usize = 8;

impl Writer {
    pub fn new(project: &PlistValue) -> Self {
        Self::with_options(project, WriterOptions::default())
    }

    pub fn with_options(project: &PlistValue, options: WriterOptions) -> Self {
        // Pre-compute indent strings
        let mut indents = Vec::with_capacity(MAX_CACHED_INDENT + 1);
        for i in 0..=MAX_CACHED_INDENT {
            indents.push(options.tab.repeat(i));
        }

        // Estimate output size: typically ~1.05x input representation
        let estimated_size = estimate_size(project);

        let mut writer = Writer {
            buf: String::with_capacity(estimated_size),
            indent: 0,
            comments: create_reference_list(project),
            options,
            indents,
        };
        writer.write_shebang();
        writer.write_project(project);
        writer
    }

    pub fn get_results(self) -> String {
        self.buf
    }

    // ── Core write primitives (zero-allocation hot path) ───────────

    #[inline(always)]
    fn write_indent(&mut self) {
        if self.indent <= MAX_CACHED_INDENT {
            self.buf.push_str(&self.indents[self.indent]);
        } else {
            for _ in 0..self.indent {
                self.buf.push_str(&self.indents[1]);
            }
        }
    }

    #[inline(always)]
    fn write_line(&mut self, s: &str) {
        self.write_indent();
        self.buf.push_str(s);
        self.buf.push('\n');
    }

    #[inline(always)]
    fn write_assign_line(&mut self, key: &str, value: &str) {
        self.write_indent();
        self.buf.push_str(key);
        self.buf.push_str(" = ");
        self.buf.push_str(value);
        self.buf.push_str(";\n");
    }

    // ── Formatting helpers (minimize allocations) ──────────────────

    /// Write a formatted ID with optional comment. Writes directly to buf.
    fn write_format_id(&mut self, id: &str) {
        if let Some(comment) = self.comments.get(id) {
            if !comment.is_empty() {
                self.buf.push_str(id);
                self.buf.push_str(" /* ");
                self.buf.push_str(comment);
                self.buf.push_str(" */");
                return;
            }
        }
        write_ensure_quotes_to(&mut self.buf, id);
    }

    /// Return a formatted ID as a String (needed for inline formatting).
    fn format_id_string(&self, id: &str) -> String {
        if let Some(comment) = self.comments.get(id) {
            if !comment.is_empty() {
                let mut s = String::with_capacity(id.len() + comment.len() + 7);
                s.push_str(id);
                s.push_str(" /* ");
                s.push_str(comment);
                s.push_str(" */");
                return s;
            }
        }
        ensure_quotes(id)
    }

    fn key_has_float_value(key: &str) -> bool {
        // Check all-uppercase without allocating (key must equal its uppercased form)
        key.bytes().all(|b| !b.is_ascii_lowercase())
            && (key.ends_with("SWIFT_VERSION")
                || key.ends_with("MARKETING_VERSION")
                || key.ends_with("_DEPLOYMENT_TARGET"))
    }

    // ── Structure writers ──────────────────────────────────────────

    fn write_shebang(&mut self) {
        self.write_indent();
        self.buf.push_str("// ");
        self.buf.push_str(&self.options.shebang);
        self.buf.push('\n');
    }

    fn write_project(&mut self, project: &PlistValue) {
        self.write_line("{");
        if let Some(obj) = project.as_object() {
            self.indent += 1;
            self.write_object(obj, true);
            self.indent -= 1;
        }
        self.write_line("}");
    }

    fn write_object(&mut self, object: &IndexMap<String, PlistValue>, is_base: bool) {
        for (key, value) in object {
            match value {
                PlistValue::Data(data) => {
                    let d = format_data(data);
                    self.write_assign_line(&ensure_quotes(key), &d);
                }
                PlistValue::Array(items) => {
                    self.write_array(key, items);
                }
                PlistValue::Object(inner) => {
                    if !is_base && inner.is_empty() {
                        self.write_indent();
                        write_ensure_quotes_to(&mut self.buf, key);
                        self.buf.push_str(" = {};\n");
                        continue;
                    }
                    self.write_indent();
                    write_ensure_quotes_to(&mut self.buf, key);
                    self.buf.push_str(" = {\n");
                    self.indent += 1;
                    if is_base && key == "objects" {
                        self.write_pbx_objects(inner);
                    } else {
                        self.write_object(inner, is_base);
                    }
                    self.indent -= 1;
                    self.write_line("};");
                }
                PlistValue::Integer(n) => {
                    if Self::key_has_float_value(key) {
                        let mut val = String::new();
                        let _ = write!(val, "{}.0", n);
                        self.write_assign_line(&ensure_quotes(key), &ensure_quotes(&val));
                    } else {
                        let val = n.to_string();
                        self.write_assign_line(&ensure_quotes(key), &ensure_quotes(&val));
                    }
                }
                PlistValue::Float(f) => {
                    let val = if Self::key_has_float_value(key) && f.fract() == 0.0 {
                        format!("{}.0", *f as i64)
                    } else {
                        format!("{}", f)
                    };
                    self.write_assign_line(&ensure_quotes(key), &ensure_quotes(&val));
                }
                PlistValue::String(s) => {
                    if key == "remoteGlobalIDString" || key == "TestTargetID" {
                        self.write_assign_line(&ensure_quotes(key), &ensure_quotes(s));
                    } else {
                        let eq_key = ensure_quotes(key);
                        self.write_indent();
                        self.buf.push_str(&eq_key);
                        self.buf.push_str(" = ");
                        self.write_format_id(s);
                        self.buf.push_str(";\n");
                    }
                }
            }
        }
    }

    fn write_pbx_objects(&mut self, objects: &IndexMap<String, PlistValue>) {
        // Group by ISA — collect into a BTreeMap for alphabetical ISA ordering
        let mut by_isa: std::collections::BTreeMap<&str, Vec<(&str, &IndexMap<String, PlistValue>)>> =
            std::collections::BTreeMap::new();

        for (id, obj) in objects {
            if let Some(obj_map) = obj.as_object() {
                let isa = obj_map.get("isa").and_then(|v| v.as_str()).unwrap_or("Unknown");
                by_isa.entry(isa).or_default().push((id.as_str(), obj_map));
            }
        }

        for (isa, entries) in &mut by_isa {
            self.buf.push('\n');
            let _ = write!(self.buf, "/* Begin {} section */\n", isa);

            entries.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

            for &(id, obj) in entries.iter() {
                self.write_object_inclusive(id, obj);
            }

            let _ = write!(self.buf, "/* End {} section */\n", isa);
        }
    }

    fn write_object_inclusive(&mut self, key: &str, value: &IndexMap<String, PlistValue>) {
        let isa = value.get("isa").and_then(|v| v.as_str()).unwrap_or("");
        if is_pbx_build_file(isa) || is_pbx_file_reference(isa) {
            self.write_object_inline(key, value);
            return;
        }
        self.write_indent();
        self.write_format_id(key);
        self.buf.push_str(" = {\n");
        self.indent += 1;
        self.write_object(value, false);
        self.indent -= 1;
        self.write_line("};");
    }

    /// Write an object on a single line (for PBXBuildFile and PBXFileReference).
    /// Writes directly to buf without intermediate Vec<String>.
    fn write_object_inline(&mut self, key: &str, value: &IndexMap<String, PlistValue>) {
        self.write_indent();
        self.write_inline_recursive(key, value);
        // Trim trailing space and add newline
        if self.buf.ends_with(' ') {
            self.buf.pop();
        }
        self.buf.push('\n');
    }

    fn write_inline_recursive(&mut self, key: &str, value: &IndexMap<String, PlistValue>) {
        let fid = self.format_id_string(key);
        self.buf.push_str(&fid);
        self.buf.push_str(" = {");

        for (k, v) in value {
            match v {
                PlistValue::Data(data) => {
                    let d = format_data(data);
                    write_ensure_quotes_to(&mut self.buf, k);
                    self.buf.push_str(" = ");
                    self.buf.push_str(&d);
                    self.buf.push_str("; ");
                }
                PlistValue::Array(items) => {
                    write_ensure_quotes_to(&mut self.buf, k);
                    self.buf.push_str(" = (");
                    for item in items {
                        match item {
                            PlistValue::String(s) => {
                                write_ensure_quotes_to(&mut self.buf, s);
                                self.buf.push_str(", ");
                            }
                            PlistValue::Integer(n) => {
                                let s = n.to_string();
                                write_ensure_quotes_to(&mut self.buf, &s);
                                self.buf.push_str(", ");
                            }
                            _ => {}
                        }
                    }
                    self.buf.push_str("); ");
                }
                PlistValue::Object(inner) => {
                    self.write_inline_recursive(k, inner);
                }
                PlistValue::String(s) => {
                    if k == "remoteGlobalIDString" || k == "TestTargetID" {
                        write_ensure_quotes_to(&mut self.buf, k);
                        self.buf.push_str(" = ");
                        write_ensure_quotes_to(&mut self.buf, s);
                        self.buf.push_str("; ");
                    } else {
                        write_ensure_quotes_to(&mut self.buf, k);
                        self.buf.push_str(" = ");
                        let fid = self.format_id_string(s);
                        self.buf.push_str(&fid);
                        self.buf.push_str("; ");
                    }
                }
                PlistValue::Integer(n) => {
                    write_ensure_quotes_to(&mut self.buf, k);
                    self.buf.push_str(" = ");
                    let s = n.to_string();
                    write_ensure_quotes_to(&mut self.buf, &s);
                    self.buf.push_str("; ");
                }
                PlistValue::Float(f) => {
                    write_ensure_quotes_to(&mut self.buf, k);
                    self.buf.push_str(" = ");
                    let s = format!("{}", f);
                    write_ensure_quotes_to(&mut self.buf, &s);
                    self.buf.push_str("; ");
                }
            }
        }

        self.buf.push_str("}; ");
    }

    fn write_array(&mut self, key: &str, items: &[PlistValue]) {
        self.write_indent();
        write_ensure_quotes_to(&mut self.buf, key);
        self.buf.push_str(" = (\n");
        self.indent += 1;

        for item in items {
            match item {
                PlistValue::Data(data) => {
                    let d = format_data(data);
                    self.write_indent();
                    self.buf.push_str(&d);
                    self.buf.push_str(",\n");
                }
                PlistValue::Object(inner) => {
                    self.write_line("{");
                    self.indent += 1;
                    self.write_object(inner, false);
                    self.indent -= 1;
                    self.write_line("},");
                }
                PlistValue::String(s) => {
                    self.write_indent();
                    self.write_format_id(s);
                    self.buf.push_str(",\n");
                }
                PlistValue::Integer(n) => {
                    self.write_indent();
                    let s = n.to_string();
                    self.write_format_id(&s);
                    self.buf.push_str(",\n");
                }
                PlistValue::Float(f) => {
                    self.write_indent();
                    let s = format!("{}", f);
                    self.write_format_id(&s);
                    self.buf.push_str(",\n");
                }
                _ => {}
            }
        }

        self.indent -= 1;
        self.write_line(");");
    }
}

/// Write ensure_quotes directly into a buffer without allocating when no quotes needed.
#[inline]
fn write_ensure_quotes_to(buf: &mut String, value: &str) {
    if is_safe_unquoted(value) {
        // Fast path: check if escaping is needed
        if needs_escaping(value) {
            buf.push_str(&add_quotes(value));
        } else {
            buf.push_str(value);
        }
    } else {
        buf.push('"');
        buf.push_str(&add_quotes(value));
        buf.push('"');
    }
}

/// Check if a string can be written without quotes.
#[inline]
fn is_safe_unquoted(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'$' || b == b'/' || b == b':' || b == b'.')
}

/// Check if a string contains characters that need escaping.
#[inline]
fn needs_escaping(s: &str) -> bool {
    s.bytes().any(|b| b < 0x20 || b == b'"' || b == b'\\' || b == 0x7f)
}

/// Rough estimate of output size from a PlistValue tree.
fn estimate_size(value: &PlistValue) -> usize {
    match value {
        PlistValue::String(s) => s.len() + 4,
        PlistValue::Integer(_) => 12,
        PlistValue::Float(_) => 16,
        PlistValue::Data(d) => d.len() * 2 + 4,
        PlistValue::Array(items) => items.iter().map(estimate_size).sum::<usize>() + 8,
        PlistValue::Object(map) => map.iter().map(|(k, v)| k.len() + estimate_size(v) + 6).sum::<usize>() + 8,
    }
}

/// Build a .pbxproj string from a PlistValue.
pub fn build(project: &PlistValue) -> String {
    Writer::new(project).get_results()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_simple_project() -> PlistValue {
        let mut root = IndexMap::new();
        root.insert("archiveVersion".to_string(), PlistValue::Integer(1));
        root.insert("objectVersion".to_string(), PlistValue::Integer(46));
        root.insert("classes".to_string(), PlistValue::Object(IndexMap::new()));
        PlistValue::Object(root)
    }

    #[test]
    fn test_basic_output() {
        let project = make_simple_project();
        let output = build(&project);
        assert!(output.starts_with("// !$*UTF8*$!\n"));
        assert!(output.contains("archiveVersion = 1;"));
        assert!(output.contains("objectVersion = 46;"));
        // At base level, empty objects are expanded (not inlined)
        assert!(output.contains("classes = {\n"));
    }

    #[test]
    fn test_float_key_formatting() {
        assert!(Writer::key_has_float_value("SWIFT_VERSION"));
        assert!(Writer::key_has_float_value("IPHONEOS_DEPLOYMENT_TARGET"));
        assert!(Writer::key_has_float_value("MARKETING_VERSION"));
        assert!(!Writer::key_has_float_value("name"));
        assert!(!Writer::key_has_float_value("swift_version")); // lowercase
    }
}
