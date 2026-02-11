use super::escape::unescape_string;

/// Lookup table for unquoted string literal characters: `[a-zA-Z0-9_$/:.-]`.
/// Single array index per byte — branchless classification in hot loops.
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

/// Token types produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    Equals,
    Semicolon,
    Comma,
    StringLiteral(String),
    QuotedString(String),
    DataLiteral(Vec<u8>),
}

/// Fast tokenizer for .pbxproj files.
///
/// Uses direct byte scanning with memchr-style loops instead of
/// per-character function calls. Skips whitespace and comments in bulk.
pub struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.as_bytes(),
            pos: 0,
        }
    }

    /// Skip whitespace and comments in bulk using fast byte scanning.
    #[inline]
    fn skip_trivia(&mut self) {
        let bytes = self.input;
        let len = bytes.len();

        loop {
            // Skip whitespace bytes in bulk
            while self.pos < len {
                match bytes[self.pos] {
                    b' ' | b'\t' | b'\r' | b'\n' => self.pos += 1,
                    _ => break,
                }
            }

            if self.pos >= len {
                return;
            }

            // Check for comments
            if bytes[self.pos] == b'/' && self.pos + 1 < len {
                if bytes[self.pos + 1] == b'/' {
                    // Line comment: find next newline using memchr-style scan
                    self.pos += 2;
                    while self.pos < len && bytes[self.pos] != b'\n' {
                        self.pos += 1;
                    }
                    continue;
                } else if bytes[self.pos + 1] == b'*' {
                    // Block comment: scan for */
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

    /// Read a quoted string. The opening quote is at self.pos.
    fn read_quoted_string(&mut self) -> Result<Token, String> {
        let quote = self.input[self.pos];
        self.pos += 1;
        let bytes = self.input;
        let len = bytes.len();
        let start = self.pos;

        // Fast path: scan for the closing quote without escapes
        let mut has_escape = false;
        let mut end = start;
        while end < len {
            let b = bytes[end];
            if b == quote {
                break;
            }
            if b == b'\\' {
                has_escape = true;
                end += 1; // skip the escaped char
                if end < len {
                    end += 1;
                }
            } else {
                end += 1;
            }
        }

        if end >= len {
            return Err(format!("Unterminated string at offset {}", start - 1));
        }

        let raw = std::str::from_utf8(&bytes[start..end]).map_err(|e| format!("Invalid UTF-8 in string: {}", e))?;
        self.pos = end + 1; // skip closing quote

        let unescaped = if has_escape {
            unescape_string(raw)
        } else {
            raw.to_string()
        };
        Ok(Token::QuotedString(unescaped))
    }

    /// Read a data literal `<hex bytes>`.
    fn read_data_literal(&mut self) -> Result<Token, String> {
        self.pos += 1; // skip <
        let bytes = self.input;
        let len = bytes.len();
        let start = self.pos;

        // Scan to closing >
        while self.pos < len && bytes[self.pos] != b'>' {
            self.pos += 1;
        }

        if self.pos >= len {
            return Err(format!("Unterminated data literal at offset {}", start - 1));
        }

        // Extract hex digits (skip whitespace)
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

        Ok(Token::DataLiteral(result_bytes))
    }

    /// Read an unquoted string literal matching `[\w_$/:.-]+`.
    #[inline]
    fn read_string_literal(&mut self) -> Token {
        let start = self.pos;
        let bytes = self.input;
        let len = bytes.len();

        while self.pos < len && IS_LITERAL_CHAR[bytes[self.pos] as usize] {
            self.pos += 1;
        }

        let s = unsafe { std::str::from_utf8_unchecked(&bytes[start..self.pos]) };
        Token::StringLiteral(s.to_string())
    }

    /// Get the next token, or None at EOF.
    pub fn next_token(&mut self) -> Result<Option<Token>, String> {
        self.skip_trivia();

        if self.pos >= self.input.len() {
            return Ok(None);
        }

        let b = self.input[self.pos];
        match b {
            b'{' => {
                self.pos += 1;
                Ok(Some(Token::OpenBrace))
            }
            b'}' => {
                self.pos += 1;
                Ok(Some(Token::CloseBrace))
            }
            b'(' => {
                self.pos += 1;
                Ok(Some(Token::OpenParen))
            }
            b')' => {
                self.pos += 1;
                Ok(Some(Token::CloseParen))
            }
            b'=' => {
                self.pos += 1;
                Ok(Some(Token::Equals))
            }
            b';' => {
                self.pos += 1;
                Ok(Some(Token::Semicolon))
            }
            b',' => {
                self.pos += 1;
                Ok(Some(Token::Comma))
            }
            b'<' => self.read_data_literal().map(Some),
            b'"' | b'\'' => self.read_quoted_string().map(Some),
            _ if IS_LITERAL_CHAR[b as usize] => Ok(Some(self.read_string_literal())),
            _ => Err(format!(
                "Unexpected character '{}' (0x{:02x}) at offset {}",
                b as char, b, self.pos
            )),
        }
    }

    /// Tokenize the entire input.
    pub fn tokenize_all(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::with_capacity(self.input.len() / 8); // rough estimate
        while let Some(tok) = self.next_token()? {
            tokens.push(tok);
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structural_tokens() {
        let mut lexer = Lexer::new("{ } ( ) = ; ,");
        let tokens = lexer.tokenize_all().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::OpenBrace,
                Token::CloseBrace,
                Token::OpenParen,
                Token::CloseParen,
                Token::Equals,
                Token::Semicolon,
                Token::Comma,
            ]
        );
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new("hello_world foo/bar.txt $PATH 0755");
        let tokens = lexer.tokenize_all().unwrap();
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0], Token::StringLiteral("hello_world".to_string()));
        assert_eq!(tokens[1], Token::StringLiteral("foo/bar.txt".to_string()));
        assert_eq!(tokens[2], Token::StringLiteral("$PATH".to_string()));
        assert_eq!(tokens[3], Token::StringLiteral("0755".to_string()));
    }

    #[test]
    fn test_quoted_string() {
        let mut lexer = Lexer::new(r#""hello world" 'single'"#);
        let tokens = lexer.tokenize_all().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], Token::QuotedString("hello world".to_string()));
        assert_eq!(tokens[1], Token::QuotedString("single".to_string()));
    }

    #[test]
    fn test_data_literal() {
        let mut lexer = Lexer::new("<0123 ABCD ef>");
        let tokens = lexer.tokenize_all().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0], Token::DataLiteral(vec![0x01, 0x23, 0xAB, 0xCD, 0xEF]));
    }

    #[test]
    fn test_skip_line_comment() {
        let mut lexer = Lexer::new("// this is a comment\n{ }");
        let tokens = lexer.tokenize_all().unwrap();
        assert_eq!(tokens, vec![Token::OpenBrace, Token::CloseBrace]);
    }

    #[test]
    fn test_skip_block_comment() {
        let mut lexer = Lexer::new("/* block */ { /* inner\nmultiline */ }");
        let tokens = lexer.tokenize_all().unwrap();
        assert_eq!(tokens, vec![Token::OpenBrace, Token::CloseBrace]);
    }

    #[test]
    fn test_pbxproj_snippet() {
        let input = r#"// !$*UTF8*$!
{
	archiveVersion = 1;
	objectVersion = 46;
}"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize_all().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::OpenBrace,
                Token::StringLiteral("archiveVersion".to_string()),
                Token::Equals,
                Token::StringLiteral("1".to_string()),
                Token::Semicolon,
                Token::StringLiteral("objectVersion".to_string()),
                Token::Equals,
                Token::StringLiteral("46".to_string()),
                Token::Semicolon,
                Token::CloseBrace,
            ]
        );
    }
}
