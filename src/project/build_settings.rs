use std::path::Path;

/// Resolve Xcode build setting variable references.
///
/// Build settings can include `$(VARIABLE)` and `$(VARIABLE:transform)` references.
/// This function recursively resolves them.
///
/// Port of `resolveXcodeBuildSetting` from `resolveBuildSettings.ts`.
pub fn resolve_xcode_build_setting<F>(value: &str, lookup: &F) -> String
where
    F: Fn(&str) -> Option<String>,
{
    let result = resolve_once(value, lookup);
    if result != value {
        // Recurse until stable
        resolve_xcode_build_setting(&result, lookup)
    } else {
        result
    }
}

fn resolve_once<F>(value: &str, lookup: &F) -> String
where
    F: Fn(&str) -> Option<String>,
{
    let mut result = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if i + 1 < len && bytes[i] == b'$' && bytes[i + 1] == b'(' {
            // Find the matching close paren
            let start = i + 2;
            let mut depth = 1;
            let mut end = start;
            while end < len && depth > 0 {
                if bytes[end] == b'(' {
                    depth += 1;
                } else if bytes[end] == b')' {
                    depth -= 1;
                }
                if depth > 0 {
                    end += 1;
                }
            }

            if depth == 0 {
                let inner = &value[start..end];
                // Split variable name from transformations
                let parts: Vec<&str> = inner.splitn(2, ':').collect();
                let variable = parts[0];
                let transformations: Vec<&str> = if parts.len() > 1 {
                    parts[1].split(':').collect()
                } else {
                    vec![]
                };

                // Look up the variable
                let mut resolved = lookup(variable);

                // Recursively resolve the looked-up value
                if let Some(ref val) = resolved {
                    let recursed = resolve_xcode_build_setting(val, lookup);
                    resolved = Some(recursed);
                }

                // Apply transformations
                let mut current = resolved.unwrap_or_default();
                for modifier in &transformations {
                    current = apply_transform(&current, modifier);
                }

                // Recursively resolve the result
                let final_val = resolve_xcode_build_setting(&current, lookup);
                result.push_str(&final_val);
                i = end + 1;
            } else {
                // Unmatched paren — keep as-is
                result.push('$');
                i += 1;
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

fn apply_transform(value: &str, modifier: &str) -> String {
    match modifier {
        "lower" => value.to_lowercase(),
        "upper" => value.to_uppercase(),
        "suffix" => Path::new(value)
            .extension()
            .map(|ext| format!(".{}", ext.to_string_lossy()))
            .unwrap_or_default(),
        "file" => Path::new(value)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        "dir" => Path::new(value)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        "base" => Path::new(value)
            .file_stem()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        "rfc1034identifier" => value
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect(),
        "c99extidentifier" => value
            .chars()
            .map(|c| if c == '-' || c == ' ' { '_' } else { c })
            .collect(),
        "standardizepath" => {
            if value.is_empty() {
                String::new()
            } else {
                // Approximate: resolve the path
                let path = Path::new(value);
                path.canonicalize()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| value.to_string())
            }
        }
        other => {
            // Handle default=VALUE
            if let Some(default_val) = other.strip_prefix("default=") {
                if value.is_empty() {
                    default_val.to_string()
                } else {
                    value.to_string()
                }
            } else {
                value.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_simple_substitution() {
        let mut vars = HashMap::new();
        vars.insert("PRODUCT_NAME".to_string(), "MyApp".to_string());

        let result = resolve_xcode_build_setting("$(PRODUCT_NAME)", &|key| vars.get(key).cloned());
        assert_eq!(result, "MyApp");
    }

    #[test]
    fn test_transform_lower() {
        let mut vars = HashMap::new();
        vars.insert("PRODUCT_NAME".to_string(), "MyApp".to_string());

        let result = resolve_xcode_build_setting("$(PRODUCT_NAME:lower)", &|key| vars.get(key).cloned());
        assert_eq!(result, "myapp");
    }

    #[test]
    fn test_transform_rfc1034() {
        let mut vars = HashMap::new();
        vars.insert("PRODUCT_NAME".to_string(), "My App!".to_string());

        let result = resolve_xcode_build_setting("$(PRODUCT_NAME:rfc1034identifier)", &|key| vars.get(key).cloned());
        assert_eq!(result, "My-App-");
    }

    #[test]
    fn test_nested_variables() {
        let mut vars = HashMap::new();
        vars.insert("PRODUCT_NAME".to_string(), "$(TARGET_NAME)".to_string());
        vars.insert("TARGET_NAME".to_string(), "MyTarget".to_string());

        let result = resolve_xcode_build_setting("$(PRODUCT_NAME)", &|key| vars.get(key).cloned());
        assert_eq!(result, "MyTarget");
    }

    #[test]
    fn test_no_substitution() {
        let result = resolve_xcode_build_setting("plain text", &|_| None);
        assert_eq!(result, "plain text");
    }

    #[test]
    fn test_mixed_text_and_vars() {
        let mut vars = HashMap::new();
        vars.insert("NAME".to_string(), "World".to_string());

        let result = resolve_xcode_build_setting("Hello $(NAME)!", &|key| vars.get(key).cloned());
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_file_transform() {
        let mut vars = HashMap::new();
        vars.insert("PATH".to_string(), "/usr/local/bin/tool".to_string());

        let result = resolve_xcode_build_setting("$(PATH:file)", &|key| vars.get(key).cloned());
        assert_eq!(result, "tool");
    }

    #[test]
    fn test_suffix_transform() {
        let mut vars = HashMap::new();
        vars.insert("FILE".to_string(), "main.swift".to_string());

        let result = resolve_xcode_build_setting("$(FILE:suffix)", &|key| vars.get(key).cloned());
        assert_eq!(result, ".swift");
    }
}
