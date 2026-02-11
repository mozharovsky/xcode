/// Escape special characters in a string for .pbxproj output.
///
/// Matches `addQuotes` from writer.ts:
/// - Control chars 0x00-0x1F (except \n which uses \n) → \Uxxxx
/// - Standard escapes: \a \b \f \r \t \v \n \" \\
pub fn add_quotes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\x07' => result.push_str("\\a"),
            '\x08' => result.push_str("\\b"),
            '\x0C' => result.push_str("\\f"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\x0B' => result.push_str("\\v"),
            '\n' => result.push_str("\\n"),
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            c if (c as u32) < 0x20 && c != '\n' => {
                result.push_str(&format!("\\U{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result
}

/// Ensure a string value is properly quoted for .pbxproj output.
///
/// Matches `ensureQuotes` from writer.ts:
/// - If value matches `^[\w_$/:.]+$` → no quotes needed
/// - Otherwise → wrap in double quotes
/// - Note: hyphen `-` is NOT in the safe set
pub fn ensure_quotes(value: &str) -> String {
    let escaped = add_quotes(value);
    if is_safe_unquoted(&escaped) {
        escaped
    } else {
        format!("\"{}\"", escaped)
    }
}

/// Check if a string can be written without quotes.
/// Safe chars: word chars (\w = [a-zA-Z0-9_]), $, /, :, .
/// Note: hyphen is NOT safe (differs from lexer's StringLiteral pattern).
fn is_safe_unquoted(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.bytes()
        .all(|b| matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'$' | b'/' | b':' | b'.'))
}

/// Format binary data as a hex data literal.
pub fn format_data(data: &[u8]) -> String {
    let hex: String = data.iter().map(|b| format!("{:02X}", b)).collect();
    format!("<{}>", hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_quotes_basic() {
        assert_eq!(add_quotes("hello"), "hello");
        assert_eq!(add_quotes("hello\nworld"), "hello\\nworld");
        assert_eq!(add_quotes("hello\tworld"), "hello\\tworld");
        assert_eq!(add_quotes(r#"say "hi""#), r#"say \"hi\""#);
        assert_eq!(add_quotes("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn test_add_quotes_control_chars() {
        assert_eq!(add_quotes("\x00"), "\\U0000");
        assert_eq!(add_quotes("\x01"), "\\U0001");
        assert_eq!(add_quotes("\x1F"), "\\U001f");
    }

    #[test]
    fn test_ensure_quotes_safe() {
        assert_eq!(ensure_quotes("hello"), "hello");
        assert_eq!(ensure_quotes("path/to/file.swift"), "path/to/file.swift");
        assert_eq!(ensure_quotes("$inherited"), "$inherited");
        assert_eq!(ensure_quotes("PRODUCT_NAME"), "PRODUCT_NAME");
    }

    #[test]
    fn test_ensure_quotes_needs_quotes() {
        assert_eq!(ensure_quotes("hello world"), "\"hello world\"");
        assert_eq!(ensure_quotes("foo-bar"), "\"foo-bar\"");
        assert_eq!(ensure_quotes(""), "\"\"");
        assert_eq!(ensure_quotes("value with spaces"), "\"value with spaces\"");
    }

    #[test]
    fn test_format_data() {
        assert_eq!(format_data(&[0xAB, 0xCD, 0x12, 0x34]), "<ABCD1234>");
        assert_eq!(format_data(&[]), "<>");
    }
}
