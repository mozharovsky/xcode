use std::io::Cursor;

/// Parse a plist string into a serde_json::Value.
///
/// Auto-detects XML vs binary format. Handles `.entitlements`, `Info.plist`,
/// and any other Apple plist file.
pub fn parse_plist(content: &str) -> Result<serde_json::Value, String> {
    let cursor = Cursor::new(content.as_bytes());
    plist::from_reader(cursor).map_err(|e| format!("Failed to parse plist: {}", e))
}

/// Serialize a serde_json::Value to an XML plist string.
pub fn build_plist(value: &serde_json::Value) -> Result<String, String> {
    let mut buf = Vec::new();
    plist::to_writer_xml(&mut buf, value).map_err(|e| format!("Failed to serialize plist: {}", e))?;
    String::from_utf8(buf).map_err(|e| format!("Plist output is not valid UTF-8: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENTITLEMENTS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>aps-environment</key>
	<string>development</string>
	<key>com.apple.developer.applesignin</key>
	<array>
		<string>Default</string>
	</array>
	<key>com.apple.developer.associated-domains</key>
	<array>
		<string>applinks:example.com</string>
	</array>
</dict>
</plist>"#;

    const INFO_PLIST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDisplayName</key>
	<string>MyApp</string>
	<key>CFBundleIdentifier</key>
	<string>$(PRODUCT_BUNDLE_IDENTIFIER)</string>
	<key>CFBundleShortVersionString</key>
	<string>1.0</string>
	<key>CFBundleVersion</key>
	<string>1</string>
	<key>ITSAppUsesNonExemptEncryption</key>
	<false/>
	<key>UILaunchStoryboardName</key>
	<string>LaunchScreen</string>
</dict>
</plist>"#;

    #[test]
    fn test_parse_entitlements() {
        let value = parse_plist(ENTITLEMENTS).unwrap();
        let obj = value.as_object().unwrap();

        assert_eq!(obj["aps-environment"], "development");

        let signin = obj["com.apple.developer.applesignin"].as_array().unwrap();
        assert_eq!(signin.len(), 1);
        assert_eq!(signin[0], "Default");

        let domains = obj["com.apple.developer.associated-domains"].as_array().unwrap();
        assert_eq!(domains[0], "applinks:example.com");
    }

    #[test]
    fn test_parse_info_plist() {
        let value = parse_plist(INFO_PLIST).unwrap();
        let obj = value.as_object().unwrap();

        assert_eq!(obj["CFBundleDisplayName"], "MyApp");
        assert_eq!(obj["CFBundleShortVersionString"], "1.0");
        assert_eq!(obj["CFBundleVersion"], "1");
        assert_eq!(obj["ITSAppUsesNonExemptEncryption"], false);
    }

    #[test]
    fn test_roundtrip_entitlements() {
        let parsed = parse_plist(ENTITLEMENTS).unwrap();
        let xml = build_plist(&parsed).unwrap();
        let reparsed = parse_plist(&xml).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn test_roundtrip_info_plist() {
        let parsed = parse_plist(INFO_PLIST).unwrap();
        let xml = build_plist(&parsed).unwrap();
        let reparsed = parse_plist(&xml).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn test_build_produces_valid_xml() {
        let parsed = parse_plist(INFO_PLIST).unwrap();
        let xml = build_plist(&parsed).unwrap();
        assert!(xml.contains("<?xml version=\"1.0\""));
        assert!(xml.contains("<plist version=\"1.0\">"));
        assert!(xml.contains("<key>CFBundleDisplayName</key>"));
    }

    #[test]
    fn test_modify_and_rebuild() {
        let mut value = parse_plist(INFO_PLIST).unwrap();
        let obj = value.as_object_mut().unwrap();
        obj.insert(
            "CFBundleShortVersionString".to_string(),
            serde_json::Value::String("2.0".to_string()),
        );
        obj.insert(
            "CFBundleVersion".to_string(),
            serde_json::Value::String("42".to_string()),
        );

        let xml = build_plist(&value).unwrap();
        let reparsed = parse_plist(&xml).unwrap();
        assert_eq!(reparsed["CFBundleShortVersionString"], "2.0");
        assert_eq!(reparsed["CFBundleVersion"], "42");
    }

    #[test]
    fn test_parse_invalid_xml() {
        let result = parse_plist("not xml at all");
        assert!(result.is_err());
    }
}
