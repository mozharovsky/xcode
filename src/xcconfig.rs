use indexmap::IndexMap;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Condition {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum XCConfigEntry {
    #[serde(rename = "setting")]
    Setting {
        key: String,
        value: String,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        conditions: Vec<Condition>,
    },
    #[serde(rename = "include")]
    Include { path: String, optional: bool },
    #[serde(rename = "comment")]
    Comment { text: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct XCConfig {
    pub entries: Vec<XCConfigEntry>,
}

/// Find the `=` that represents the assignment, skipping `=` inside `[condition=value]` brackets.
fn find_assignment_eq(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, c) in s.char_indices() {
        match c {
            '[' => depth += 1,
            ']' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            '=' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

impl XCConfig {
    pub fn parse(content: &str) -> Result<XCConfig, String> {
        let mut entries = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(comment) = trimmed.strip_prefix("//") {
                entries.push(XCConfigEntry::Comment { text: comment.trim().to_string() });
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("#include?") {
                let path = rest.trim().trim_matches('"').to_string();
                entries.push(XCConfigEntry::Include { path, optional: true });
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("#include") {
                let path = rest.trim().trim_matches('"').to_string();
                entries.push(XCConfigEntry::Include { path, optional: false });
                continue;
            }

            if let Some(eq_pos) = find_assignment_eq(trimmed) {
                let raw_key = trimmed[..eq_pos].trim();
                let value = trimmed[eq_pos + 1..].trim().to_string();

                let mut conditions = Vec::new();
                let key = if let Some(bracket_start) = raw_key.find('[') {
                    let base_key = raw_key[..bracket_start].trim().to_string();
                    let mut rest = &raw_key[bracket_start..];
                    while let Some(start) = rest.find('[') {
                        if let Some(end_offset) = rest[start..].find(']') {
                            let cond_str = &rest[start + 1..start + end_offset];
                            if let Some(eq) = cond_str.find('=') {
                                conditions.push(Condition {
                                    key: cond_str[..eq].to_string(),
                                    value: cond_str[eq + 1..].to_string(),
                                });
                            }
                            rest = &rest[start + end_offset + 1..];
                        } else {
                            break;
                        }
                    }
                    base_key
                } else {
                    raw_key.to_string()
                };
                entries.push(XCConfigEntry::Setting { key, value, conditions });
            }
        }
        Ok(XCConfig { entries })
    }

    pub fn from_file(path: &str) -> Result<XCConfig, String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;
        Self::parse(&content)
    }

    /// Flatten all settings into a key-value map (ignores conditions and includes).
    pub fn flatten(&self) -> IndexMap<String, String> {
        let mut map = IndexMap::new();
        for entry in &self.entries {
            if let XCConfigEntry::Setting { key, value, conditions } = entry {
                if conditions.is_empty() {
                    map.insert(key.clone(), value.clone());
                }
            }
        }
        map
    }

    pub fn build(&self) -> String {
        let mut out = String::new();
        for entry in &self.entries {
            match entry {
                XCConfigEntry::Comment { text } => {
                    out.push_str(&format!("// {}\n", text));
                }
                XCConfigEntry::Include { path, optional } => {
                    if *optional {
                        out.push_str(&format!("#include? \"{}\"\n", path));
                    } else {
                        out.push_str(&format!("#include \"{}\"\n", path));
                    }
                }
                XCConfigEntry::Setting { key, value, conditions } => {
                    out.push_str(key);
                    for cond in conditions {
                        out.push_str(&format!("[{}={}]", cond.key, cond.value));
                    }
                    out.push_str(&format!(" = {}\n", value));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/xcconfigs");

    fn load(name: &str) -> XCConfig {
        XCConfig::from_file(&format!("{}/{}", FIXTURES, name)).unwrap()
    }

    #[test]
    fn parse_simple() {
        let config = load("simple.xcconfig");
        let flat = config.flatten();
        assert_eq!(flat["PRODUCT_NAME"], "MyApp");
        assert_eq!(flat["SWIFT_VERSION"], "5.0");
        assert_eq!(flat["PRODUCT_BUNDLE_IDENTIFIER"], "com.example.myapp");
    }

    #[test]
    fn parse_conditional() {
        let config = load("conditional.xcconfig");
        let conditionals: Vec<_> = config
            .entries
            .iter()
            .filter(|e| matches!(e, XCConfigEntry::Setting { conditions, .. } if !conditions.is_empty()))
            .collect();
        assert!(!conditionals.is_empty());
        if let XCConfigEntry::Setting { key, conditions, .. } = &conditionals[0] {
            assert_eq!(key, "OTHER_LDFLAGS");
            assert_eq!(conditions[0].key, "sdk");
        }
    }

    #[test]
    fn parse_include() {
        let config = load("Parent.xcconfig");
        let includes: Vec<_> = config.entries.iter().filter(|e| matches!(e, XCConfigEntry::Include { .. })).collect();
        assert!(!includes.is_empty());
    }

    #[test]
    fn parse_optional_include() {
        let config = load("optional-include.xcconfig");
        let opt_includes: Vec<_> =
            config.entries.iter().filter(|e| matches!(e, XCConfigEntry::Include { optional: true, .. })).collect();
        assert!(!opt_includes.is_empty());
    }

    #[test]
    fn flatten_ignores_conditionals() {
        let config = load("conditional.xcconfig");
        let flat = config.flatten();
        assert!(flat.contains_key("PRODUCT_NAME"));
        assert!(flat.contains_key("DEBUG_INFORMATION_FORMAT"));
        assert!(!flat.contains_key("OTHER_LDFLAGS"));
    }

    #[test]
    fn parse_comments() {
        let config = load("simple.xcconfig");
        let comments: Vec<_> = config.entries.iter().filter(|e| matches!(e, XCConfigEntry::Comment { .. })).collect();
        assert!(!comments.is_empty());
    }

    #[test]
    fn round_trip_simple() {
        let config = load("simple.xcconfig");
        let output = config.build();
        let reparsed = XCConfig::parse(&output).unwrap();
        assert_eq!(config.flatten(), reparsed.flatten());
    }

    #[test]
    fn combined_conditions() {
        let config = load("conditional.xcconfig");
        let multi_cond: Vec<_> = config
            .entries
            .iter()
            .filter(|e| matches!(e, XCConfigEntry::Setting { conditions, .. } if conditions.len() > 1))
            .collect();
        assert!(!multi_cond.is_empty());
    }
}
