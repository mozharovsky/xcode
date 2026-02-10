use std::collections::HashSet;

use indexmap::IndexMap;

use crate::types::plist::PlistValue;
use crate::types::isa::Isa;

/// A trait providing shared behavior for all PBX object types.
pub trait PbxObjectExt {
    /// The ISA type of this object.
    fn isa(&self) -> Isa;

    /// The UUID of this object.
    fn uuid(&self) -> &str;

    /// A human-readable display name.
    fn display_name(&self) -> Option<String>;

    /// Returns true if this object references the given UUID.
    fn is_referencing(&self, uuid: &str) -> bool;

    /// Remove all references to the given UUID from this object's properties.
    fn remove_reference(&mut self, uuid: &str);

    /// Get all UUID references contained in this object (for inflation).
    fn get_reference_uuids(&self) -> Vec<String>;
}

/// Generic PBX object — stores any pbxproj object as its raw plist properties.
///
/// Key design decision: Unlike the TypeScript version which inflates UUID references
/// into live object pointers, Rust stores all references as UUID strings.
/// Lookups go through the XcodeProject's objects map to avoid ownership complexity.
#[derive(Debug, Clone)]
pub struct PbxObject {
    pub uuid: String,
    pub isa: String,
    pub props: IndexMap<String, PlistValue>,
}

impl PbxObject {
    /// Create a new PbxObject from raw plist data.
    pub fn from_plist(uuid: String, props: &IndexMap<String, PlistValue>) -> Self {
        let isa = props
            .get("isa")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        PbxObject {
            uuid,
            isa,
            props: props.clone(),
        }
    }

    /// Convert back to plist representation.
    pub fn to_plist(&self) -> IndexMap<String, PlistValue> {
        self.props.clone()
    }

    /// Get a string property.
    pub fn get_str(&self, key: &str) -> Option<&str> {
        self.props.get(key).and_then(|v| v.as_str())
    }

    /// Get an integer property.
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.props.get(key).and_then(|v| v.as_integer())
    }

    /// Get an array property.
    pub fn get_array(&self, key: &str) -> Option<&Vec<PlistValue>> {
        self.props.get(key).and_then(|v| v.as_array())
    }

    /// Get an object (map) property.
    pub fn get_object(&self, key: &str) -> Option<&IndexMap<String, PlistValue>> {
        self.props.get(key).and_then(|v| v.as_object())
    }

    /// Set a string property.
    pub fn set_str(&mut self, key: &str, value: &str) {
        self.props
            .insert(key.to_string(), PlistValue::String(value.to_string()));
    }

    /// Set an integer property.
    pub fn set_int(&mut self, key: &str, value: i64) {
        self.props
            .insert(key.to_string(), PlistValue::Integer(value));
    }

    /// Set a property.
    pub fn set(&mut self, key: &str, value: PlistValue) {
        self.props.insert(key.to_string(), value);
    }

    /// Remove a property.
    pub fn remove(&mut self, key: &str) -> Option<PlistValue> {
        self.props.shift_remove(key)
    }

    /// Get properties that are known to contain UUID references, based on ISA type.
    pub fn reference_keys(&self) -> Vec<&str> {
        match self.isa.as_str() {
            "PBXProject" => vec![
                "buildConfigurationList",
                "mainGroup",
                "productRefGroup",
                "targets",
                "packageReferences",
            ],
            "PBXNativeTarget" | "PBXAggregateTarget" | "PBXLegacyTarget" => vec![
                "buildConfigurationList",
                "dependencies",
                "buildPhases",
                "buildRules",
                "productReference",
                "packageProductDependencies",
                "fileSystemSynchronizedGroups",
            ],
            "PBXGroup" | "PBXVariantGroup" | "XCVersionGroup" => vec!["children"],
            "XCConfigurationList" => vec!["buildConfigurations"],
            "XCBuildConfiguration" => vec!["baseConfigurationReference"],
            "PBXBuildFile" => vec!["fileRef", "productRef"],
            "PBXTargetDependency" => vec!["target", "targetProxy"],
            "PBXContainerItemProxy" => vec!["containerPortal"],
            "PBXReferenceProxy" => vec!["remoteRef"],
            "XCSwiftPackageProductDependency" => vec!["package"],
            "PBXFileSystemSynchronizedRootGroup" => vec!["exceptions"],
            // Build phases
            _ if self.isa.ends_with("BuildPhase") => vec!["files"],
            // File references, build rules, swift package refs, etc. have no UUID references
            _ => vec![],
        }
    }

    /// Collect all UUID strings referenced by this object.
    pub fn collect_references(&self) -> HashSet<String> {
        let mut refs = HashSet::new();
        for key in self.reference_keys() {
            if let Some(value) = self.props.get(key) {
                match value {
                    PlistValue::String(s) if looks_like_uuid(s) => {
                        refs.insert(s.clone());
                    }
                    PlistValue::Array(items) => {
                        for item in items {
                            if let Some(s) = item.as_str() {
                                if looks_like_uuid(s) {
                                    refs.insert(s.to_string());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        refs
    }
}

impl PbxObjectExt for PbxObject {
    fn isa(&self) -> Isa {
        self.isa.parse().unwrap_or(Isa::PBXBuildFile)
    }

    fn uuid(&self) -> &str {
        &self.uuid
    }

    fn display_name(&self) -> Option<String> {
        self.get_str("name")
            .or_else(|| self.get_str("productName"))
            .or_else(|| self.get_str("path"))
            .map(|s| s.to_string())
    }

    fn is_referencing(&self, uuid: &str) -> bool {
        for key in self.reference_keys() {
            if let Some(value) = self.props.get(key) {
                match value {
                    PlistValue::String(s) if s == uuid => return true,
                    PlistValue::Array(items) => {
                        if items.iter().any(|item| item.as_str() == Some(uuid)) {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    fn remove_reference(&mut self, uuid: &str) {
        let keys: Vec<String> = self.reference_keys().iter().map(|k| k.to_string()).collect();
        for key in keys {
            if let Some(value) = self.props.get_mut(&key) {
                match value {
                    PlistValue::String(s) if s == uuid => {
                        *value = PlistValue::String(String::new());
                    }
                    PlistValue::Array(items) => {
                        items.retain(|item| item.as_str() != Some(uuid));
                    }
                    _ => {}
                }
            }
        }
    }

    fn get_reference_uuids(&self) -> Vec<String> {
        self.collect_references().into_iter().collect()
    }
}

/// Heuristic: a 24-char hex string is likely a UUID.
fn looks_like_uuid(s: &str) -> bool {
    s.len() == 24 && s.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbx_object_basics() {
        let mut props = IndexMap::new();
        props.insert("isa".to_string(), PlistValue::String("PBXGroup".to_string()));
        props.insert("name".to_string(), PlistValue::String("Sources".to_string()));
        props.insert(
            "children".to_string(),
            PlistValue::Array(vec![PlistValue::String(
                "13B07F961A680F5B00A75B9A".to_string(),
            )]),
        );

        let obj = PbxObject::from_plist("AABB00112233445566778899".to_string(), &props);
        assert_eq!(obj.isa, "PBXGroup");
        assert_eq!(obj.get_str("name"), Some("Sources"));
        assert!(obj.is_referencing("13B07F961A680F5B00A75B9A"));
        assert!(!obj.is_referencing("0000000000000000000000FF"));
    }

    #[test]
    fn test_remove_reference() {
        let mut props = IndexMap::new();
        props.insert("isa".to_string(), PlistValue::String("PBXGroup".to_string()));
        props.insert(
            "children".to_string(),
            PlistValue::Array(vec![
                PlistValue::String("AAAA00000000000000000001".to_string()),
                PlistValue::String("BBBB00000000000000000002".to_string()),
            ]),
        );

        let mut obj = PbxObject::from_plist("ROOT0000000000000000000".to_string(), &props);
        obj.remove_reference("AAAA00000000000000000001");
        let children = obj.get_array("children").unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].as_str(), Some("BBBB00000000000000000002"));
    }
}
