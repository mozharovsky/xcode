use indexmap::IndexMap;

use super::lexer::{Lexer, Token};
use crate::types::PlistValue;

/// JS MAX_SAFE_INTEGER (2^53 - 1)
const MAX_SAFE_INTEGER: i64 = 9007199254740991;

/// Recursive descent parser for .pbxproj (Old-Style Plist) files.
///
/// Grammar:
/// ```text
/// head       = object | array
/// object     = '{' objectItem* '}'
/// array      = '(' (value ','?)* ')'
/// objectItem = identifier '=' value ';'
/// identifier = QuotedString | StringLiteral
/// value      = object | array | DataLiteral | identifier
/// ```
///
/// Produces `PlistValue` directly (no intermediate CST).
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        let pos = self.pos;
        match self.advance() {
            Some(tok) if tok == expected => Ok(()),
            Some(tok) => Err(format!(
                "Expected {:?}, got {:?} at token position {}",
                expected, tok, pos
            )),
            None => Err(format!(
                "Expected {:?}, got EOF at token position {}",
                expected, pos
            )),
        }
    }

    /// Parse the top-level head rule: object | array
    pub fn parse_head(&mut self) -> Result<PlistValue, String> {
        match self.peek() {
            Some(Token::OpenBrace) => self.parse_object(),
            Some(Token::OpenParen) => self.parse_array(),
            Some(tok) => Err(format!("Expected '{{' or '(' at start, got {:?}", tok)),
            None => Err("Empty input".to_string()),
        }
    }

    /// Parse an object: `{ objectItem* }`
    fn parse_object(&mut self) -> Result<PlistValue, String> {
        self.expect(&Token::OpenBrace)?;
        let mut map = IndexMap::new();

        loop {
            match self.peek() {
                Some(Token::CloseBrace) => {
                    self.advance();
                    return Ok(PlistValue::Object(map));
                }
                None => return Err("Unterminated object".to_string()),
                _ => {
                    let (key, value) = self.parse_object_item()?;
                    map.insert(key, value);
                }
            }
        }
    }

    /// Parse an object item: `identifier = value ;`
    fn parse_object_item(&mut self) -> Result<(String, PlistValue), String> {
        // Keys are always strings (even if numeric)
        let key = self.parse_identifier_as_string()?;
        self.expect(&Token::Equals)?;
        let value = self.parse_value()?;
        self.expect(&Token::Semicolon)?;
        Ok((key, value))
    }

    /// Parse an array: `( (value ','?)* )`
    fn parse_array(&mut self) -> Result<PlistValue, String> {
        self.expect(&Token::OpenParen)?;
        let mut items = Vec::new();

        loop {
            match self.peek() {
                Some(Token::CloseParen) => {
                    self.advance();
                    return Ok(PlistValue::Array(items));
                }
                None => return Err("Unterminated array".to_string()),
                _ => {
                    items.push(self.parse_value()?);
                    // Optional trailing comma
                    if let Some(Token::Comma) = self.peek() {
                        self.advance();
                    }
                }
            }
        }
    }

    /// Parse a value: object | array | DataLiteral | identifier
    fn parse_value(&mut self) -> Result<PlistValue, String> {
        match self.peek() {
            Some(Token::OpenBrace) => self.parse_object(),
            Some(Token::OpenParen) => self.parse_array(),
            Some(Token::DataLiteral(_)) => {
                if let Some(Token::DataLiteral(data)) = self.advance().cloned() {
                    Ok(PlistValue::Data(data))
                } else {
                    unreachable!()
                }
            }
            Some(Token::QuotedString(_)) => {
                if let Some(Token::QuotedString(s)) = self.advance().cloned() {
                    Ok(PlistValue::String(s))
                } else {
                    unreachable!()
                }
            }
            Some(Token::StringLiteral(_)) => {
                if let Some(Token::StringLiteral(s)) = self.advance().cloned() {
                    Ok(parse_type(&s))
                } else {
                    unreachable!()
                }
            }
            Some(tok) => Err(format!("Unexpected token in value: {:?}", tok)),
            None => Err("Unexpected EOF in value".to_string()),
        }
    }

    /// Parse an identifier, always returning a String (for object keys).
    fn parse_identifier_as_string(&mut self) -> Result<String, String> {
        match self.advance().cloned() {
            Some(Token::QuotedString(s)) => Ok(s),
            Some(Token::StringLiteral(s)) => Ok(s),
            Some(tok) => Err(format!("Expected identifier, got {:?}", tok)),
            None => Err("Expected identifier, got EOF".to_string()),
        }
    }
}

/// Convert an unquoted string literal to the appropriate PlistValue type.
///
/// Matches the `parseType` function from JsonVisitor.ts:
/// - Leading-zero digit strings (like "0755") → preserve as String
/// - Digit-only strings within MAX_SAFE_INTEGER → Integer
/// - Decimal numbers without trailing zero → Float
/// - Everything else → String
fn parse_type(literal: &str) -> PlistValue {
    // Preserve octal literals with leading zeros (e.g., "0755")
    if literal.len() > 1 && literal.starts_with('0') && literal.chars().all(|c| c.is_ascii_digit())
    {
        return PlistValue::String(literal.to_string());
    }

    // Handle integers
    if literal.chars().all(|c| c.is_ascii_digit()) && !literal.is_empty() {
        if let Ok(num) = literal.parse::<i64>() {
            if num <= MAX_SAFE_INTEGER {
                return PlistValue::Integer(num);
            }
        }
        // Too large — preserve as string
        return PlistValue::String(literal.to_string());
    }

    // Handle decimal numbers
    let is_numeric = {
        let s = literal.strip_prefix('+').or_else(|| literal.strip_prefix('-')).unwrap_or(literal);
        if s.is_empty() {
            false
        } else if s.contains('.') {
            let parts: Vec<&str> = s.splitn(2, '.').collect();
            let int_ok = parts[0].is_empty() || parts[0].chars().all(|c| c.is_ascii_digit());
            let frac_ok = parts.get(1).map_or(true, |f| f.chars().all(|c| c.is_ascii_digit()));
            int_ok && frac_ok && !(parts[0].is_empty() && parts.get(1).map_or(true, |f| f.is_empty()))
        } else {
            false
        }
    };

    if is_numeric {
        // Preserve trailing zeros (e.g., "5.0" stays as string)
        if literal.ends_with('0') && literal.contains('.') {
            return PlistValue::String(literal.to_string());
        }
        if let Ok(num) = literal.parse::<f64>() {
            if !num.is_nan() {
                return PlistValue::Float(num);
            }
        }
    }

    PlistValue::String(literal.to_string())
}

/// Parse a .pbxproj string into a PlistValue.
pub fn parse(text: &str) -> Result<PlistValue, String> {
    let mut lexer = Lexer::new(text);
    let tokens = lexer.tokenize_all()?;
    let mut parser = Parser::new(tokens);
    parser.parse_head()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object() {
        let input = r#"{ key = value; }"#;
        let result = parse(input).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(
            obj.get("key").and_then(|v| v.as_str()),
            Some("value")
        );
    }

    #[test]
    fn test_parse_nested_object() {
        let input = r#"{ outer = { inner = 42; }; }"#;
        let result = parse(input).unwrap();
        let outer = result.as_object().unwrap();
        let inner_obj = outer.get("outer").unwrap().as_object().unwrap();
        assert_eq!(inner_obj.get("inner").unwrap().as_integer(), Some(42));
    }

    #[test]
    fn test_parse_array() {
        let input = r#"{ items = (one, two, three); }"#;
        let result = parse(input).unwrap();
        let obj = result.as_object().unwrap();
        let items = obj.get("items").unwrap().as_array().unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].as_str(), Some("one"));
    }

    #[test]
    fn test_parse_quoted_strings() {
        let input = r#"{ "quoted key" = "quoted value"; }"#;
        let result = parse(input).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(
            obj.get("quoted key").and_then(|v| v.as_str()),
            Some("quoted value")
        );
    }

    #[test]
    fn test_parse_integer() {
        let input = r#"{ version = 46; }"#;
        let result = parse(input).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("version").unwrap().as_integer(), Some(46));
    }

    #[test]
    fn test_parse_octal_preserved_as_string() {
        let input = r#"{ mode = 0755; }"#;
        let result = parse(input).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("mode").unwrap().as_str(), Some("0755"));
    }

    #[test]
    fn test_parse_data_literal() {
        let input = r#"{ data = <ABCD1234>; }"#;
        let result = parse(input).unwrap();
        let obj = result.as_object().unwrap();
        match obj.get("data").unwrap() {
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
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("archiveVersion").unwrap().as_integer(), Some(1));
        assert_eq!(obj.get("objectVersion").unwrap().as_integer(), Some(46));
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
        let obj = result.as_object().unwrap();
        let items = obj.get("items").unwrap().as_array().unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn test_parse_large_uuid_as_string() {
        // UUIDs larger than MAX_SAFE_INTEGER should stay as strings
        let input = "{ id = 13B07F961A680F5B00A75B9A; }";
        let result = parse(input).unwrap();
        let obj = result.as_object().unwrap();
        // This is alphanumeric, not all digits, so stays as string
        assert_eq!(
            obj.get("id").unwrap().as_str(),
            Some("13B07F961A680F5B00A75B9A")
        );
    }

    #[test]
    fn test_parse_type_function() {
        // String literals
        assert_eq!(parse_type("hello"), PlistValue::String("hello".to_string()));

        // Integers
        assert_eq!(parse_type("42"), PlistValue::Integer(42));
        assert_eq!(parse_type("0"), PlistValue::Integer(0));

        // Octal preservation
        assert_eq!(parse_type("0755"), PlistValue::String("0755".to_string()));
        assert_eq!(parse_type("00"), PlistValue::String("00".to_string()));

        // Floats with trailing zero preserved
        assert_eq!(parse_type("5.0"), PlistValue::String("5.0".to_string()));

        // Floats
        assert_eq!(parse_type("3.14"), PlistValue::Float(3.14));
    }
}
