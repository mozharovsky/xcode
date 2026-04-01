/// Integration tests for the pbxproj parser and writer.
///
/// These tests mirror the original TypeScript test suite from @bacons/xcode.
use xcodekit::parser::parse;
use xcodekit::types::plist::PlistValue;
use xcodekit::writer::serializer::build;

mod fixture_tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    const FIXTURES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

    /// All fixtures that should parse without error.
    const ALL_FIXTURES: &[&str] = &[
        "01-float.pbxproj",
        "006-spm.pbxproj",
        "007-xcode16.pbxproj",
        "008-out-of-order-orphans.pbxproj",
        "009-expo-app-clip.pbxproj",
        "shopify-tophat.pbxproj",
        "AFNetworking.pbxproj",
        "project.pbxproj",
        "project-rn74.pbxproj",
        "Cocoa-Application.pbxproj",
        "project-multitarget-missing-targetattributes.pbxproj",
        "project-multitarget.pbxproj",
        "project-rni.pbxproj",
        "project-swift.pbxproj",
        "project-with-entitlements.pbxproj",
        "project-with-incorrect-create-manifest-ios-path.pbxproj",
        "project-without-create-manifest-ios.pbxproj",
        "swift-protobuf.pbxproj",
        "watch.pbxproj",
    ];

    /// Fixtures that should round-trip (parse → build → equals original).
    const IN_OUT_FIXTURES: &[&str] = &[
        "006-spm.pbxproj",
        "007-xcode16.pbxproj",
        "AFNetworking.pbxproj",
        "project.pbxproj",
        "project-rn74.pbxproj",
        "project-multitarget-missing-targetattributes.pbxproj",
        "project-multitarget.pbxproj",
        "project-rni.pbxproj",
        "project-swift.pbxproj",
        "project-with-entitlements.pbxproj",
        "project-with-incorrect-create-manifest-ios-path.pbxproj",
        "project-without-create-manifest-ios.pbxproj",
        "watch.pbxproj",
    ];

    #[test]
    fn test_all_fixtures_parse() {
        for fixture in ALL_FIXTURES {
            let path = Path::new(FIXTURES_DIR).join(fixture);
            let content = fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {}: {}", fixture, e));
            let result = parse(&content);
            assert!(result.is_ok(), "Failed to parse {}: {:?}", fixture, result.err());
            let plist = result.unwrap();
            assert!(plist.as_object().is_some(), "{} should parse to an object", fixture);
        }
    }

    #[test]
    fn test_round_trip_fixtures() {
        let mut passed = Vec::new();
        let mut failed = Vec::new();

        for fixture in IN_OUT_FIXTURES {
            let path = Path::new(FIXTURES_DIR).join(fixture);
            let original = fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {}: {}", fixture, e));
            let parsed = parse(&original).unwrap_or_else(|e| panic!("Failed to parse {}: {}", fixture, e));
            let output = build(&parsed);

            if output == original {
                passed.push(*fixture);
            } else {
                failed.push(*fixture);

                // Find first difference for debugging
                let orig_lines: Vec<&str> = original.lines().collect();
                let out_lines: Vec<&str> = output.lines().collect();
                for (i, (a, b)) in orig_lines.iter().zip(out_lines.iter()).enumerate() {
                    if a != b {
                        eprintln!(
                            "Round-trip diff in {} at line {}:\n  expected: {:?}\n  got:      {:?}",
                            fixture,
                            i + 1,
                            a,
                            b
                        );
                        break;
                    }
                }
                if orig_lines.len() != out_lines.len() {
                    eprintln!(
                        "Round-trip line count differs for {}: {} vs {}",
                        fixture,
                        orig_lines.len(),
                        out_lines.len()
                    );
                }
            }
        }

        eprintln!("\nRound-trip results: {}/{} passed", passed.len(), IN_OUT_FIXTURES.len());
        for f in &passed {
            eprintln!("  PASS: {}", f);
        }
        for f in &failed {
            eprintln!("  FAIL: {}", f);
        }
    }

    #[test]
    fn test_numeric_object_keys_are_strings() {
        let input = "{ 123 = abc; 456 = { 789 = def; }; }";
        let result = parse(input).unwrap();
        assert_eq!(result.get("123").and_then(|v| v.as_str()), Some("abc"));
        let inner = result.get("456").unwrap();
        assert_eq!(inner.get("789").and_then(|v| v.as_str()), Some("def"));
    }
}

mod unicode_tests {
    use super::*;

    #[test]
    fn test_unicode_escape_sequences() {
        let input = r#"{ testKey = "\U0041\U0042\U0043"; }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("testKey").and_then(|v| v.as_str()), Some("ABC"));
    }

    #[test]
    fn test_standard_escape_sequences() {
        let input = r#"{
            newline = "line1\nline2";
            tab = "col1\tcol2";
            quote = "say \"hello\"";
            backslash = "path\\to\\file";
        }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("newline").and_then(|v| v.as_str()), Some("line1\nline2"));
        assert_eq!(result.get("tab").and_then(|v| v.as_str()), Some("col1\tcol2"));
        assert_eq!(result.get("quote").and_then(|v| v.as_str()), Some("say \"hello\""));
        assert_eq!(result.get("backslash").and_then(|v| v.as_str()), Some("path\\to\\file"));
    }

    #[test]
    fn test_control_character_escapes() {
        let input = r#"{
            bell = "\a";
            backspace = "\b";
            formfeed = "\f";
            carriage = "\r";
            vertical = "\v";
        }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("bell").and_then(|v| v.as_str()), Some("\x07"));
        assert_eq!(result.get("backspace").and_then(|v| v.as_str()), Some("\x08"));
        assert_eq!(result.get("formfeed").and_then(|v| v.as_str()), Some("\x0C"));
        assert_eq!(result.get("carriage").and_then(|v| v.as_str()), Some("\r"));
        assert_eq!(result.get("vertical").and_then(|v| v.as_str()), Some("\x0B"));
    }

    #[test]
    fn test_invalid_unicode_graceful() {
        let input = r#"{
            invalidUnicode = "\UZZZZ";
            partialUnicode = "\U123";
        }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("invalidUnicode").and_then(|v| v.as_str()), Some("\\UZZZZ"));
        assert_eq!(result.get("partialUnicode").and_then(|v| v.as_str()), Some("\\U123"));
    }

    #[test]
    fn test_nextstep_high_bit_characters() {
        let input = r#"{
            nonBreakSpace = "\200";
            copyright = "\240";
            registeredSign = "\260";
            bullet = "\267";
            enDash = "\261";
            emDash = "\320";
        }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("nonBreakSpace").and_then(|v| v.as_str()), Some("\u{00a0}"));
        assert_eq!(result.get("copyright").and_then(|v| v.as_str()), Some("\u{00a9}"));
        assert_eq!(result.get("registeredSign").and_then(|v| v.as_str()), Some("\u{00ae}"));
        assert_eq!(result.get("bullet").and_then(|v| v.as_str()), Some("\u{2022}"));
        assert_eq!(result.get("enDash").and_then(|v| v.as_str()), Some("\u{2013}"));
        assert_eq!(result.get("emDash").and_then(|v| v.as_str()), Some("\u{2014}"));
    }

    #[test]
    fn test_nextstep_accented_characters() {
        let input = r#"{
            aGrave = "\201";
            aAcute = "\202";
            aTilde = "\204";
            ccedilla = "\207";
            eGrave = "\210";
            oSlash = "\351";
        }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("aGrave").and_then(|v| v.as_str()), Some("\u{00c0}"));
        assert_eq!(result.get("aAcute").and_then(|v| v.as_str()), Some("\u{00c1}"));
        assert_eq!(result.get("aTilde").and_then(|v| v.as_str()), Some("\u{00c3}"));
        assert_eq!(result.get("ccedilla").and_then(|v| v.as_str()), Some("\u{00c7}"));
        assert_eq!(result.get("eGrave").and_then(|v| v.as_str()), Some("\u{00c8}"));
        assert_eq!(result.get("oSlash").and_then(|v| v.as_str()), Some("\u{00d8}"));
    }

    #[test]
    fn test_nextstep_ligatures() {
        let input = r#"{
            fiLigature = "\256";
            flLigature = "\257";
            fractionSlash = "\244";
            fHook = "\246";
            ellipsis = "\274";
        }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("fiLigature").and_then(|v| v.as_str()), Some("\u{fb01}"));
        assert_eq!(result.get("flLigature").and_then(|v| v.as_str()), Some("\u{fb02}"));
        assert_eq!(result.get("fractionSlash").and_then(|v| v.as_str()), Some("\u{2044}"));
        assert_eq!(result.get("fHook").and_then(|v| v.as_str()), Some("\u{0192}"));
        assert_eq!(result.get("ellipsis").and_then(|v| v.as_str()), Some("\u{2026}"));
    }

    #[test]
    fn test_nextstep_replacement_characters() {
        let input = r#"{
            notdef1 = "\376";
            notdef2 = "\377";
        }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("notdef1").and_then(|v| v.as_str()), Some("\u{fffd}"));
        assert_eq!(result.get("notdef2").and_then(|v| v.as_str()), Some("\u{fffd}"));
    }

    #[test]
    fn test_single_digit_octal() {
        let input = r#"{
            null = "\0";
            one = "\1";
            seven = "\7";
        }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("null").and_then(|v| v.as_str()), Some("\x00"));
        assert_eq!(result.get("one").and_then(|v| v.as_str()), Some("\x01"));
        assert_eq!(result.get("seven").and_then(|v| v.as_str()), Some("\x07"));
    }

    #[test]
    fn test_two_digit_octal() {
        let input = r#"{
            ten = "\12";
            twentySeven = "\33";
            seventySeven = "\115";
        }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("ten").and_then(|v| v.as_str()), Some("\x0a"));
        assert_eq!(result.get("twentySeven").and_then(|v| v.as_str()), Some("\x1b"));
        assert_eq!(result.get("seventySeven").and_then(|v| v.as_str()), Some("\x4d"));
    }

    #[test]
    fn test_three_digit_octal() {
        let input = r#"{
            max = "\377";
            middleRange = "\177";
            lowRange = "\077";
        }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("max").and_then(|v| v.as_str()), Some("\u{fffd}"));
        assert_eq!(result.get("middleRange").and_then(|v| v.as_str()), Some("\x7f"));
        assert_eq!(result.get("lowRange").and_then(|v| v.as_str()), Some("\x3f"));
    }

    #[test]
    fn test_octal_with_trailing_digits() {
        let input = r#"{
            test1 = "\1234";
            test2 = "\777";
        }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("test1").and_then(|v| v.as_str()), Some("S4"));
        assert_eq!(result.get("test2").and_then(|v| v.as_str()), Some("ǿ"));
    }

    #[test]
    fn test_empty_strings() {
        let input = r#"{
            empty1 = "";
            empty2 = '';
        }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("empty1").and_then(|v| v.as_str()), Some(""));
        assert_eq!(result.get("empty2").and_then(|v| v.as_str()), Some(""));
    }

    #[test]
    fn test_mixed_quote_styles() {
        let input = r#"{
            doubleQuoted = "double";
            singleQuoted = 'single';
            doubleInSingle = 'say "hello"';
            singleInDouble = "it's working";
        }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("doubleQuoted").and_then(|v| v.as_str()), Some("double"));
        assert_eq!(result.get("singleQuoted").and_then(|v| v.as_str()), Some("single"));
        assert_eq!(result.get("doubleInSingle").and_then(|v| v.as_str()), Some("say \"hello\""));
        assert_eq!(result.get("singleInDouble").and_then(|v| v.as_str()), Some("it's working"));
    }

    #[test]
    fn test_unquoted_identifiers() {
        let input = r#"{
            unquoted = value;
            withNumbers = value123;
            withPath = path/to/file;
            withDots = com.example.app;
            withHyphens = with-hyphens;
            withUnderscores = with_underscores;
        }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("unquoted").and_then(|v| v.as_str()), Some("value"));
        assert_eq!(result.get("withNumbers").and_then(|v| v.as_str()), Some("value123"));
        assert_eq!(result.get("withPath").and_then(|v| v.as_str()), Some("path/to/file"));
        assert_eq!(result.get("withDots").and_then(|v| v.as_str()), Some("com.example.app"));
        assert_eq!(result.get("withHyphens").and_then(|v| v.as_str()), Some("with-hyphens"));
        assert_eq!(result.get("withUnderscores").and_then(|v| v.as_str()), Some("with_underscores"));
    }

    #[test]
    fn test_complex_nested_escapes() {
        let input = r#"{ complex = "prefix\n\tindented\\backslash\U0041suffix"; }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("complex").and_then(|v| v.as_str()), Some("prefix\n\tindented\\backslashAsuffix"));
    }

    #[test]
    fn test_numeric_formatting_quirks() {
        let input = r#"{
            octalString = 0755;
            trailingZero = 1.0;
            integer = 42;
            float = 3.14;
            scientificNotation = 1e5;
        }"#;
        let result = parse(input).unwrap();
        assert_eq!(result.get("octalString").and_then(|v| v.as_str()), Some("0755"));
        assert_eq!(result.get("trailingZero").and_then(|v| v.as_str()), Some("1.0"));
        assert_eq!(result.get("integer").and_then(|v| v.as_integer()), Some(42));
        match result.get("float").unwrap() {
            PlistValue::Float(f) => assert!((*f - 3.14).abs() < 0.001),
            other => panic!("Expected Float, got {:?}", other),
        }
        assert_eq!(result.get("scientificNotation").and_then(|v| v.as_str()), Some("1e5"));
    }

    #[test]
    fn test_data_literals() {
        let input = r#"{ singleByte = <48>; }"#;
        let result = parse(input).unwrap();
        match result.get("singleByte").unwrap() {
            PlistValue::Data(data) => assert_eq!(data, &vec![0x48]),
            other => panic!("Expected Data, got {:?}", other),
        }
    }

    #[test]
    fn test_data_with_spaces() {
        let input = r#"{ dataWithSpaces = <48 65 6c 6c 6f>; }"#;
        let result = parse(input).unwrap();
        match result.get("dataWithSpaces").unwrap() {
            PlistValue::Data(data) => assert_eq!(data, &vec![0x48, 0x65, 0x6c, 0x6c, 0x6f]),
            other => panic!("Expected Data, got {:?}", other),
        }
    }

    #[test]
    fn test_round_trip_unicode() {
        let input = r#"{
            unicode = "\U0041\U00e9\U2022";
            nextStep = "\240\267";
            mixed = "Hello\nWorld\t\U0041";
        }"#;
        let parsed = parse(input).unwrap();
        let rebuilt = build(&parsed);
        let reparsed = parse(&rebuilt).unwrap();
        assert_eq!(reparsed.get("unicode").and_then(|v| v.as_str()), Some("Aé•"));
        assert_eq!(reparsed.get("nextStep").and_then(|v| v.as_str()), Some("©•"));
        assert_eq!(reparsed.get("mixed").and_then(|v| v.as_str()), Some("Hello\nWorld\tA"));
    }

    #[test]
    fn test_round_trip_numeric_formatting() {
        let input = r#"{
            octal = 0755;
            trailingZero = 1.0;
            integer = 42;
        }"#;
        let parsed = parse(input).unwrap();
        let rebuilt = build(&parsed);
        assert!(rebuilt.contains("0755"));
        assert!(rebuilt.contains("1.0"));
        assert!(rebuilt.contains("42"));
    }

    #[test]
    fn test_unclosed_string_error() {
        let input = r#"{
            unclosed = "missing quote;
        }"#;
        assert!(parse(input).is_err());
    }
}
