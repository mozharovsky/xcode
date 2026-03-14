use std::collections::HashMap;
use std::fmt::Write as FmtWrite;

use super::comments::{create_reference_list, is_pbx_build_file, is_pbx_file_reference};
use super::quotes::{add_quotes, format_data};
use crate::types::plist::PlistObject;
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
    pub fn new(project: &PlistValue<'_>) -> Self {
        Self::with_options(project, WriterOptions::default())
    }

    pub fn with_options(project: &PlistValue<'_>, options: WriterOptions) -> Self {
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

    fn write_project(&mut self, project: &PlistValue<'_>) {
        self.write_line("{");
        if let Some(obj) = project.as_object() {
            self.indent += 1;
            self.write_object(obj, true);
            self.indent -= 1;
        }
        self.write_line("}");
    }

    fn write_object(&mut self, object: &PlistObject<'_>, is_base: bool) {
        for (key, value) in object {
            match value {
                PlistValue::Data(data) => {
                    let d = format_data(data);
                    self.write_indent();
                    write_ensure_quotes_to(&mut self.buf, key);
                    self.buf.push_str(" = ");
                    self.buf.push_str(&d);
                    self.buf.push_str(";\n");
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
                    self.write_indent();
                    write_ensure_quotes_to(&mut self.buf, key);
                    self.buf.push_str(" = ");
                    if Self::key_has_float_value(key) {
                        let _ = write!(self.buf, "{}.0", n);
                    } else {
                        let _ = write!(self.buf, "{}", n);
                    }
                    self.buf.push_str(";\n");
                }
                PlistValue::Float(f) => {
                    self.write_indent();
                    write_ensure_quotes_to(&mut self.buf, key);
                    self.buf.push_str(" = ");
                    if Self::key_has_float_value(key) && f.fract() == 0.0 {
                        let _ = write!(self.buf, "{}.0", *f as i64);
                    } else {
                        let _ = write!(self.buf, "{}", f);
                    }
                    self.buf.push_str(";\n");
                }
                PlistValue::String(s) => {
                    self.write_indent();
                    write_ensure_quotes_to(&mut self.buf, key);
                    self.buf.push_str(" = ");
                    if key == "remoteGlobalIDString" || key == "TestTargetID" {
                        write_ensure_quotes_to(&mut self.buf, s);
                    } else {
                        self.write_format_id(s);
                    }
                    self.buf.push_str(";\n");
                }
            }
        }
    }

    fn write_pbx_objects(&mut self, objects: &PlistObject<'_>) {
        // Group by ISA — collect into a BTreeMap for alphabetical ISA ordering
        let mut by_isa: std::collections::BTreeMap<&str, Vec<(&str, &PlistObject<'_>)>> =
            std::collections::BTreeMap::new();

        for (id, obj) in objects {
            if let Some(obj_map) = obj.as_object() {
                let id: &str = id;
                let isa = obj_map
                    .iter()
                    .find(|(k, _)| k.as_ref() == "isa")
                    .and_then(|(_, v)| v.as_str())
                    .unwrap_or("Unknown");
                by_isa.entry(isa).or_default().push((id, obj_map));
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

    fn write_object_inclusive(&mut self, key: &str, value: &PlistObject<'_>) {
        let isa = value
            .iter()
            .find(|(k, _)| k.as_ref() == "isa")
            .and_then(|(_, v)| v.as_str())
            .unwrap_or("");
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
    fn write_object_inline(&mut self, key: &str, value: &PlistObject<'_>) {
        self.write_indent();
        self.write_inline_recursive(key, value);
        // Trim trailing space and add newline
        if self.buf.ends_with(' ') {
            self.buf.pop();
        }
        self.buf.push('\n');
    }

    fn write_inline_recursive(&mut self, key: &str, value: &PlistObject<'_>) {
        self.write_format_id(key);
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
                                let _ = write!(self.buf, "{}", n);
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
                    write_ensure_quotes_to(&mut self.buf, k);
                    self.buf.push_str(" = ");
                    if k == "remoteGlobalIDString" || k == "TestTargetID" {
                        write_ensure_quotes_to(&mut self.buf, s);
                    } else {
                        self.write_format_id(s);
                    }
                    self.buf.push_str("; ");
                }
                PlistValue::Integer(n) => {
                    write_ensure_quotes_to(&mut self.buf, k);
                    self.buf.push_str(" = ");
                    let _ = write!(self.buf, "{}", n);
                    self.buf.push_str("; ");
                }
                PlistValue::Float(f) => {
                    write_ensure_quotes_to(&mut self.buf, k);
                    self.buf.push_str(" = ");
                    let _ = write!(self.buf, "{}", f);
                    self.buf.push_str("; ");
                }
            }
        }

        self.buf.push_str("}; ");
    }

    fn write_array(&mut self, key: &str, items: &[PlistValue<'_>]) {
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
                    let _ = write!(self.buf, "{}", n);
                    self.buf.push_str(",\n");
                }
                PlistValue::Float(f) => {
                    self.write_indent();
                    let _ = write!(self.buf, "{}", f);
                    self.buf.push_str(",\n");
                }
                _ => {}
            }
        }

        self.indent -= 1;
        self.write_line(");");
    }
}

/// Write ensure_quotes directly into a buffer. Zero allocation on the fast path
/// (safe unquoted strings without escaping — the vast majority of pbxproj values).
#[inline]
fn write_ensure_quotes_to(buf: &mut String, value: &str) {
    if is_safe_unquoted(value) {
        buf.push_str(value);
    } else if !needs_escaping(value) {
        buf.push('"');
        buf.push_str(value);
        buf.push('"');
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
fn estimate_size(value: &PlistValue<'_>) -> usize {
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
pub fn build(project: &PlistValue<'_>) -> String {
    Writer::new(project).get_results()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    fn make_simple_project() -> PlistValue<'static> {
        PlistValue::Object(vec![
            (Cow::Borrowed("archiveVersion"), PlistValue::Integer(1)),
            (Cow::Borrowed("objectVersion"), PlistValue::Integer(46)),
            (Cow::Borrowed("classes"), PlistValue::Object(vec![])),
        ])
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
