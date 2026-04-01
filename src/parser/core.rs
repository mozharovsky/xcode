use std::borrow::Cow;

use super::escape::unescape_string;
use crate::types::PlistValue;

/// JS MAX_SAFE_INTEGER (2^53 - 1)
const MAX_SAFE_INTEGER: i64 = 9007199254740991;

/// Lookup table for unquoted string literal characters: `[a-zA-Z0-9_$/:.-]`.
static IS_LITERAL_CHAR: [bool; 256] = {
    let mut t = [false; 256];
    let mut i: u8 = b'a';
    while i <= b'z' {
        t[i as usize] = true;
        i += 1;
    }
    i = b'A';
    while i <= b'Z' {
        t[i as usize] = true;
        i += 1;
    }
    i = b'0';
    while i <= b'9' {
        t[i as usize] = true;
        i += 1;
    }
    t[b'_' as usize] = true;
    t[b'$' as usize] = true;
    t[b'/' as usize] = true;
    t[b':' as usize] = true;
    t[b'.' as usize] = true;
    t[b'-' as usize] = true;
    t
};

/// Single-pass recursive descent parser for .pbxproj (Old-Style Plist) files.
///
/// Zero-copy: string values borrow directly from the input where possible.
/// Only escaped quoted strings allocate.
pub struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Parser {
            input: input.as_bytes(),
            pos: 0,
        }
    }

    #[inline]
    fn skip_trivia(&mut self) {
        let bytes = self.input;
        let len = bytes.len();

        loop {
            while self.pos < len {
                match bytes[self.pos] {
                    b' ' | b'\t' | b'\r' | b'\n' => self.pos += 1,
                    _ => break,
                }
            }

            if self.pos >= len {
                return;
            }

            if bytes[self.pos] == b'/' && self.pos + 1 < len {
                if bytes[self.pos + 1] == b'/' {
                    self.pos += 2;
                    while self.pos < len && bytes[self.pos] != b'\n' {
                        self.pos += 1;
                    }
                    continue;
                } else if bytes[self.pos + 1] == b'*' {
                    self.pos += 2;
                    while self.pos + 1 < len {
                        if bytes[self.pos] == b'*' && bytes[self.pos + 1] == b'/' {
                            self.pos += 2;
                            break;
                        }
                        self.pos += 1;
                    }
                    continue;
                }
            }

            break;
        }
    }

    #[inline]
    fn peek_byte(&mut self) -> Option<u8> {
        self.skip_trivia();
        self.input.get(self.pos).copied()
    }

    #[inline]
    fn expect_byte(&mut self, expected: u8) -> Result<(), String> {
        self.skip_trivia();
        if self.pos < self.input.len() && self.input[self.pos] == expected {
            self.pos += 1;
            Ok(())
        } else {
            let found = self.input.get(self.pos).map(|&b| b as char);
            Err(format!(
                "Expected '{}' at offset {}, got {:?}",
                expected as char, self.pos, found
            ))
        }
    }

    /// Read an unquoted string literal as a borrowed slice — zero allocation.
    #[inline]
    fn read_string_literal_ref(&mut self) -> &'a str {
        let start = self.pos;
        let bytes = self.input;
        let len = bytes.len();

        while self.pos < len && IS_LITERAL_CHAR[bytes[self.pos] as usize] {
            self.pos += 1;
        }

        unsafe { std::str::from_utf8_unchecked(&bytes[start..self.pos]) }
    }

    /// Read a quoted string. Returns `Cow::Borrowed` when there are no escapes
    /// (zero-copy), `Cow::Owned` when escape processing is needed.
    #[inline]
    fn read_quoted_string_cow(&mut self) -> Result<Cow<'a, str>, String> {
        let quote = self.input[self.pos];
        self.pos += 1;
        let bytes = self.input;
        let len = bytes.len();
        let start = self.pos;

        let mut has_escape = false;
        let mut end = start;
        while end < len {
            let b = bytes[end];
            if b == quote {
                break;
            }
            if b == b'\\' {
                has_escape = true;
                end += 2;
            } else {
                end += 1;
            }
        }

        if end >= len {
            return Err(format!("Unterminated string at offset {}", start - 1));
        }

        // Safety: input was read as UTF-8 text; the lexer only advances on valid byte boundaries.
        let raw = unsafe { std::str::from_utf8_unchecked(&bytes[start..end]) };
        self.pos = end + 1;

        if has_escape {
            Ok(Cow::Owned(unescape_string(raw)))
        } else {
            Ok(Cow::Borrowed(raw))
        }
    }

    /// Read a data literal `<hex bytes>`.
    fn read_data_literal(&mut self) -> Result<PlistValue<'a>, String> {
        self.pos += 1; // skip <
        let bytes = self.input;
        let len = bytes.len();
        let start = self.pos;

        while self.pos < len && bytes[self.pos] != b'>' {
            self.pos += 1;
        }

        if self.pos >= len {
            return Err(format!("Unterminated data literal at offset {}", start - 1));
        }

        let hex_region = &bytes[start..self.pos];
        self.pos += 1; // skip >

        let mut hex = String::with_capacity(hex_region.len());
        for &b in hex_region {
            if b.is_ascii_hexdigit() {
                hex.push(b as char);
            } else if b.is_ascii_whitespace() {
                // skip
            } else {
                return Err(format!("Invalid character in data literal: {}", b as char));
            }
        }

        let result_bytes = (0..hex.len())
            .step_by(2)
            .map(|i| {
                let end = (i + 2).min(hex.len());
                u8::from_str_radix(&hex[i..end], 16).map_err(|e| format!("Invalid hex: {}", e))
            })
            .collect::<Result<Vec<u8>, _>>()?;

        Ok(PlistValue::Data(result_bytes))
    }

    /// Read an identifier as Cow — zero-copy for unquoted and unescaped quoted strings.
    #[inline]
    fn read_identifier(&mut self) -> Result<Cow<'a, str>, String> {
        self.skip_trivia();
        match self.input.get(self.pos) {
            Some(b'"') | Some(b'\'') => self.read_quoted_string_cow(),
            Some(&b) if IS_LITERAL_CHAR[b as usize] => Ok(Cow::Borrowed(self.read_string_literal_ref())),
            Some(&b) => Err(format!(
                "Expected identifier at offset {}, got '{}'",
                self.pos, b as char
            )),
            None => Err(format!("Expected identifier at offset {}, got EOF", self.pos)),
        }
    }

    pub fn parse_head(&mut self) -> Result<PlistValue<'a>, String> {
        match self.peek_byte() {
            Some(b'{') => self.parse_object(),
            Some(b'(') => self.parse_array(),
            Some(b) => Err(format!("Expected '{{' or '(' at start, got '{}'", b as char)),
            None => Err("Empty input".to_string()),
        }
    }

    fn parse_object(&mut self) -> Result<PlistValue<'a>, String> {
        self.pos += 1; // skip {
        let mut pairs = Vec::new();

        loop {
            match self.peek_byte() {
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(PlistValue::Object(pairs));
                }
                None => return Err("Unterminated object".to_string()),
                _ => {
                    let key = self.read_identifier()?;
                    self.expect_byte(b'=')?;
                    let value = self.parse_value()?;
                    self.expect_byte(b';')?;
                    pairs.push((key, value));
                }
            }
        }
    }

    fn parse_array(&mut self) -> Result<PlistValue<'a>, String> {
        self.pos += 1; // skip (
        let mut items = Vec::new();

        loop {
            match self.peek_byte() {
                Some(b')') => {
                    self.pos += 1;
                    return Ok(PlistValue::Array(items));
                }
                None => return Err("Unterminated array".to_string()),
                _ => {
                    items.push(self.parse_value()?);
                    if let Some(b',') = self.peek_byte() {
                        self.pos += 1;
                    }
                }
            }
        }
    }

    fn parse_value(&mut self) -> Result<PlistValue<'a>, String> {
        match self.peek_byte() {
            Some(b'{') => self.parse_object(),
            Some(b'(') => self.parse_array(),
            Some(b'<') => self.read_data_literal(),
            Some(b'"') | Some(b'\'') => {
                let s = self.read_quoted_string_cow()?;
                Ok(PlistValue::String(s))
            }
            Some(b) if IS_LITERAL_CHAR[b as usize] => {
                let s = self.read_string_literal_ref();
                Ok(parse_type(s))
            }
            Some(b) => Err(format!(
                "Unexpected character '{}' (0x{:02x}) at offset {}",
                b as char, b, self.pos
            )),
            None => Err("Unexpected EOF in value".to_string()),
        }
    }
}

/// Convert an unquoted string literal to the appropriate PlistValue type.
/// Takes a borrowed `&'a str` — the String case produces `Cow::Borrowed` (zero-copy).
#[inline]
fn parse_type<'a>(literal: &'a str) -> PlistValue<'a> {
    let bytes = literal.as_bytes();
    if bytes.is_empty() {
        return PlistValue::String(Cow::Borrowed(literal));
    }

    let first = bytes[0];

    // Fast path: starts with a letter or non-numeric char → definitely a string.
    // Catches UUIDs, property names, paths — the vast majority of values.
    if !(first.is_ascii_digit() || first == b'+' || first == b'-' || first == b'.') {
        return PlistValue::String(Cow::Borrowed(literal));
    }

    // Starts with a digit — check if all digits
    if first.is_ascii_digit() {
        if bytes.len() == 1 {
            return PlistValue::Integer((first - b'0') as i64);
        }
        let all_digits = bytes[1..].iter().all(|b| b.is_ascii_digit());
        if all_digits {
            if first == b'0' {
                return PlistValue::String(Cow::Borrowed(literal));
            }
            if let Ok(num) = literal.parse::<i64>() {
                if num <= MAX_SAFE_INTEGER {
                    return PlistValue::Integer(num);
                }
            }
            return PlistValue::String(Cow::Borrowed(literal));
        }
        // Falls through to decimal check only if non-digit chars found
    }

    // Decimal number check: only if contains '.'
    let s = if first == b'+' || first == b'-' {
        &literal[1..]
    } else {
        literal
    };
    if let Some(dot_pos) = s.as_bytes().iter().position(|&b| b == b'.') {
        let int_part = &s[..dot_pos];
        let frac_part = &s[dot_pos + 1..];
        let int_ok = int_part.is_empty() || int_part.bytes().all(|b| b.is_ascii_digit());
        let frac_ok = frac_part.is_empty() || frac_part.bytes().all(|b| b.is_ascii_digit());
        if int_ok && frac_ok && !(int_part.is_empty() && frac_part.is_empty()) {
            if literal.ends_with('0') {
                return PlistValue::String(Cow::Borrowed(literal));
            }
            if let Ok(num) = literal.parse::<f64>() {
                if !num.is_nan() {
                    return PlistValue::Float(num);
                }
            }
        }
    }

    PlistValue::String(Cow::Borrowed(literal))
}

/// Parse a .pbxproj string into a PlistValue.
pub fn parse<'a>(text: &'a str) -> Result<PlistValue<'a>, String> {
    let mut parser = Parser::new(text);
    parser.parse_head()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object() {
        let input = r#"{ key = value; }"#;
        let result = parse(input).unwrap();
        assert!(result.as_object().is_some());
        assert_eq!(result.get("key").and_then(|v| v.as_str()), Some("value"));
    }

    #[test]
    fn test_parse_nested_object() {
        let input = r#"{ outer = { inner = 42; }; }"#;
        let result = parse(input).unwrap();
        let inner_val = result.get("outer").unwrap();
        assert!(inner_val.as_object().is_some());
        assert_eq!(inner_val.get("inner").and_then(|v| v.as_integer()), Some(42));
    }

    #[test]
    fn test_parse_array() {
        let input = r#"{ items = (one, two, three); }"#;
        let result = parse(input).unwrap();
        let items = result.get("items").unwrap().as_array().unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].as_str(), Some("one"));
    }

    #[test]
    fn test_parse_quoted_strings() {
        let input = r#"{ "quoted key" = "quoted value"; }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("quoted key").and_then(|v| v.as_str()), Some("quoted value"));
    }

    #[test]
    fn test_parse_integer() {
        let input = r#"{ version = 46; }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("version").and_then(|v| v.as_integer()), Some(46));
    }

    #[test]
    fn test_parse_octal_preserved_as_string() {
        let input = r#"{ mode = 0755; }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("mode").and_then(|v| v.as_str()), Some("0755"));
    }

    #[test]
    fn test_parse_data_literal() {
        let input = r#"{ data = <ABCD1234>; }"#;
        let result = parse(input).unwrap();
        match result.get("data").unwrap() {
            PlistValue::Data(bytes) => assert_eq!(bytes, &[0xAB, 0xCD, 0x12, 0x34]),
            other => panic!("Expected Data, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_with_comments() {
        let input = r#"// !$*UTF8*$!
{
    /* comment */ archiveVersion = 1; // line comment
    objectVersion = 46;
}"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("archiveVersion").and_then(|v| v.as_integer()), Some(1));
        assert_eq!(result.get("objectVersion").and_then(|v| v.as_integer()), Some(46));
    }

    #[test]
    fn test_parse_empty_object() {
        let result = parse("{}").unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.is_empty());
    }

    #[test]
    fn test_parse_empty_array() {
        let input = "{ items = (); }";
        let result = parse(input).unwrap();
        let items = result.get("items").unwrap().as_array().unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_parse_large_uuid_as_string() {
        let input = "{ id = 13B07F961A680F5B00A75B9A; }";
        let result = parse(input).unwrap();
        assert_eq!(
            result.get("id").and_then(|v| v.as_str()),
            Some("13B07F961A680F5B00A75B9A")
        );
    }

    #[test]
    fn test_parse_type_function() {
        assert_eq!(parse_type("hello"), PlistValue::String(Cow::Borrowed("hello")));
        assert_eq!(parse_type("42"), PlistValue::Integer(42));
        assert_eq!(parse_type("0"), PlistValue::Integer(0));
        assert_eq!(parse_type("0755"), PlistValue::String(Cow::Borrowed("0755")));
        assert_eq!(parse_type("00"), PlistValue::String(Cow::Borrowed("00")));
        assert_eq!(parse_type("5.0"), PlistValue::String(Cow::Borrowed("5.0")));
        assert_eq!(parse_type("3.14"), PlistValue::Float(3.14));
    }
}
