/// NeXTSTEP character mapping table (0x80-0xFF → Unicode code points).
/// Based on http://ftp.unicode.org/Public/MAPPINGS/VENDORS/NEXT/NEXTSTEP.TXT
#[rustfmt::skip]
const NEXT_STEP_MAPPINGS: [(u8, u32); 128] = [
    (0x80, 0x00a0), (0x81, 0x00c0), (0x82, 0x00c1), (0x83, 0x00c2),
    (0x84, 0x00c3), (0x85, 0x00c4), (0x86, 0x00c5), (0x87, 0x00c7),
    (0x88, 0x00c8), (0x89, 0x00c9), (0x8a, 0x00ca), (0x8b, 0x00cb),
    (0x8c, 0x00cc), (0x8d, 0x00cd), (0x8e, 0x00ce), (0x8f, 0x00cf),
    (0x90, 0x00d0), (0x91, 0x00d1), (0x92, 0x00d2), (0x93, 0x00d3),
    (0x94, 0x00d4), (0x95, 0x00d5), (0x96, 0x00d6), (0x97, 0x00d9),
    (0x98, 0x00da), (0x99, 0x00db), (0x9a, 0x00dc), (0x9b, 0x00dd),
    (0x9c, 0x00de), (0x9d, 0x00b5), (0x9e, 0x00d7), (0x9f, 0x00f7),
    (0xa0, 0x00a9), (0xa1, 0x00a1), (0xa2, 0x00a2), (0xa3, 0x00a3),
    (0xa4, 0x2044), (0xa5, 0x00a5), (0xa6, 0x0192), (0xa7, 0x00a7),
    (0xa8, 0x00a4), (0xa9, 0x2019), (0xaa, 0x201c), (0xab, 0x00ab),
    (0xac, 0x2039), (0xad, 0x203a), (0xae, 0xfb01), (0xaf, 0xfb02),
    (0xb0, 0x00ae), (0xb1, 0x2013), (0xb2, 0x2020), (0xb3, 0x2021),
    (0xb4, 0x00b7), (0xb5, 0x00a6), (0xb6, 0x00b6), (0xb7, 0x2022),
    (0xb8, 0x201a), (0xb9, 0x201e), (0xba, 0x201d), (0xbb, 0x00bb),
    (0xbc, 0x2026), (0xbd, 0x2030), (0xbe, 0x00ac), (0xbf, 0x00bf),
    (0xc0, 0x00b9), (0xc1, 0x02cb), (0xc2, 0x00b4), (0xc3, 0x02c6),
    (0xc4, 0x02dc), (0xc5, 0x00af), (0xc6, 0x02d8), (0xc7, 0x02d9),
    (0xc8, 0x00a8), (0xc9, 0x00b2), (0xca, 0x02da), (0xcb, 0x00b8),
    (0xcc, 0x00b3), (0xcd, 0x02dd), (0xce, 0x02db), (0xcf, 0x02c7),
    (0xd0, 0x2014), (0xd1, 0x00b1), (0xd2, 0x00bc), (0xd3, 0x00bd),
    (0xd4, 0x00be), (0xd5, 0x00e0), (0xd6, 0x00e1), (0xd7, 0x00e2),
    (0xd8, 0x00e3), (0xd9, 0x00e4), (0xda, 0x00e5), (0xdb, 0x00e7),
    (0xdc, 0x00e8), (0xdd, 0x00e9), (0xde, 0x00ea), (0xdf, 0x00eb),
    (0xe0, 0x00ec), (0xe1, 0x00c6), (0xe2, 0x00ed), (0xe3, 0x00aa),
    (0xe4, 0x00ee), (0xe5, 0x00ef), (0xe6, 0x00f0), (0xe7, 0x00f1),
    (0xe8, 0x0141), (0xe9, 0x00d8), (0xea, 0x0152), (0xeb, 0x00ba),
    (0xec, 0x00f2), (0xed, 0x00f3), (0xee, 0x00f4), (0xef, 0x00f5),
    (0xf0, 0x00f6), (0xf1, 0x00e6), (0xf2, 0x00f9), (0xf3, 0x00fa),
    (0xf4, 0x00fb), (0xf5, 0x0131), (0xf6, 0x00fc), (0xf7, 0x00fd),
    (0xf8, 0x0142), (0xf9, 0x00f8), (0xfa, 0x0153), (0xfb, 0x00df),
    (0xfc, 0x00fe), (0xfd, 0x00ff), (0xfe, 0xfffd), (0xff, 0xfffd),
];

/// Look up a NeXTSTEP byte value (>= 0x80) to its Unicode code point.
fn nextstep_to_unicode(code: u32) -> u32 {
    if code < 0x80 || code > 0xFF {
        return code;
    }
    for &(byte, unicode) in &NEXT_STEP_MAPPINGS {
        if byte as u32 == code {
            return unicode;
        }
    }
    code
}

/// Process escape sequences in a quoted string (with quotes already stripped).
///
/// Replicates the `stripQuotes` function from identifiers.ts.
/// Handles:
/// - Standard escapes: \a \b \f \n \r \t \v \" \' \\ \<newline>
/// - Unicode escapes: \Uxxxx (4 hex digits)
/// - Octal escapes: \NNN (1-3 octal digits, values >= 0x80 go through NeXTSTEP mapping)
pub fn unescape_string(input: &str) -> String {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut result = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        if bytes[i] == b'\\' && i + 1 < len {
            let next = bytes[i + 1];
            match next {
                b'a' => {
                    result.push('\x07');
                    i += 2;
                }
                b'b' => {
                    result.push('\x08');
                    i += 2;
                }
                b'f' => {
                    result.push('\x0C');
                    i += 2;
                }
                b'n' => {
                    result.push('\n');
                    i += 2;
                }
                b'r' => {
                    result.push('\r');
                    i += 2;
                }
                b't' => {
                    result.push('\t');
                    i += 2;
                }
                b'v' => {
                    result.push('\x0B');
                    i += 2;
                }
                b'"' => {
                    result.push('"');
                    i += 2;
                }
                b'\'' => {
                    result.push('\'');
                    i += 2;
                }
                b'\\' => {
                    result.push('\\');
                    i += 2;
                }
                b'\n' => {
                    result.push('\n');
                    i += 2;
                }
                // Unicode escape: \Uxxxx
                b'U' if i + 5 < len => {
                    let hex = &input[i + 2..i + 6];
                    if hex.len() == 4 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
                        let code = u32::from_str_radix(hex, 16).unwrap();
                        if let Some(ch) = char::from_u32(code) {
                            result.push(ch);
                        }
                        i += 6;
                    } else {
                        result.push('\\');
                        i += 1;
                    }
                }
                // Octal escape: \NNN (1-3 digits)
                b'0'..=b'7' => {
                    let mut octal = String::new();
                    let mut j = i + 1;
                    while j < len && j < i + 4 && bytes[j] >= b'0' && bytes[j] <= b'7' {
                        octal.push(bytes[j] as char);
                        j += 1;
                    }
                    let code = u32::from_str_radix(&octal, 8).unwrap_or(0);
                    let mapped = if code >= 0x80 { nextstep_to_unicode(code) } else { code };
                    if let Some(ch) = char::from_u32(mapped) {
                        result.push(ch);
                    }
                    i = j;
                }
                // Unknown escape — preserve both characters
                _ => {
                    result.push('\\');
                    result.push(next as char);
                    i += 2;
                }
            }
        } else {
            // Safety: we're iterating byte by byte but need to handle UTF-8.
            // For non-ASCII bytes, decode the full character.
            let ch = input[i..].chars().next().unwrap();
            result.push(ch);
            i += ch.len_utf8();
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_escapes() {
        assert_eq!(unescape_string(r"\n"), "\n");
        assert_eq!(unescape_string(r"\t"), "\t");
        assert_eq!(unescape_string(r"\r"), "\r");
        assert_eq!(unescape_string(r"\\"), "\\");
        assert_eq!(unescape_string(r#"\""#), "\"");
        assert_eq!(unescape_string(r"\'"), "'");
        assert_eq!(unescape_string(r"\a"), "\x07");
        assert_eq!(unescape_string(r"\b"), "\x08");
        assert_eq!(unescape_string(r"\f"), "\x0C");
        assert_eq!(unescape_string(r"\v"), "\x0B");
    }

    #[test]
    fn test_unicode_escape() {
        assert_eq!(unescape_string(r"\U0041"), "A");
        assert_eq!(unescape_string(r"\U00e9"), "é");
        assert_eq!(unescape_string(r"\U0000"), "\0");
    }

    #[test]
    fn test_octal_escape() {
        // Simple ASCII octal
        assert_eq!(unescape_string(r"\101"), "A"); // 0o101 = 65 = 'A'
        assert_eq!(unescape_string(r"\0"), "\0");
        assert_eq!(unescape_string(r"\12"), "\n"); // 0o12 = 10 = '\n'
    }

    #[test]
    fn test_nextstep_octal() {
        // 0o200 = 0x80 = NeXTSTEP non-breaking space (U+00A0)
        assert_eq!(unescape_string(r"\200"), "\u{00a0}");
        // 0o341 = 0xE1 = NeXTSTEP Æ (U+00C6)
        assert_eq!(unescape_string(r"\341"), "\u{00c6}");
    }

    #[test]
    fn test_no_escape() {
        assert_eq!(unescape_string("hello world"), "hello world");
        assert_eq!(unescape_string(""), "");
    }

    #[test]
    fn test_mixed_escapes() {
        assert_eq!(unescape_string(r"hello\nworld"), "hello\nworld");
        assert_eq!(unescape_string(r"path/to/\tfile"), "path/to/\tfile");
    }
}
