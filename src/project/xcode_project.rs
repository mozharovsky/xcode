use std::collections::HashSet;
use std::path::Path;

use indexmap::IndexMap;

use crate::objects::{PbxObject, PbxObjectExt};
use crate::parser;
use crate::types::plist::PlistValue;
use crate::writer::serializer;

use super::uuid::generate_uuid;

/// An orphaned reference: an object UUID referenced from a property
/// (e.g. a build phase's `files` array) that doesn't exist in the `objects` map.
#[derive(Debug, Clone)]
pub struct OrphanedReference {
    pub referrer_uuid: String,
    pub referrer_isa: String,
    pub property: String,
    pub orphan_uuid: String,
}

/// The main container for an Xcode project.
///
/// Stores all objects as a flat map of UUID → PbxObject, plus project metadata.
/// Unlike the TypeScript version, this does NOT inflate UUID references into
/// object pointers. All references are UUID strings, lookups go through this map.
#[derive(Debug, Clone)]
pub struct XcodeProject {
    pub archive_version: i64,
    pub object_version: i64,
    pub classes: IndexMap<String, PlistValue>,
    pub root_object_uuid: String,
    objects: IndexMap<String, PbxObject>,
    file_path: Option<String>,
}

impl XcodeProject {
    /// Open and parse a .pbxproj file from disk.
    pub fn open(file_path: &str) -> Result<Self, String> {
        let contents =
            std::fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;
        let mut project = Self::from_plist(&contents)?;
        project.file_path = Some(file_path.to_string());
        Ok(project)
    }

    /// Parse a .pbxproj string into an XcodeProject.
    pub fn from_plist(text: &str) -> Result<Self, String> {
        let plist = parser::parse(text)?;
        Self::from_plist_value(&plist)
    }

    /// Create from an already-parsed PlistValue.
    pub fn from_plist_value(plist: &PlistValue) -> Result<Self, String> {
        let root = plist
            .as_object()
            .ok_or("Root must be an object")?;

        let archive_version = root
            .get("archiveVersion")
            .and_then(|v| v.as_integer())
            .unwrap_or(1);

        let object_version = root
            .get("objectVersion")
            .and_then(|v| v.as_integer())
            .unwrap_or(46);

        let classes = root
            .get("classes")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let root_object_uuid = root
            .get("rootObject")
            .and_then(|v| v.as_str())
            .ok_or("rootObject is required")?
            .to_string();

        let objects_map = root
            .get("objects")
            .and_then(|v| v.as_object())
            .ok_or("objects is required")?;

        // Inflate all objects
        let mut objects = IndexMap::new();
        for (uuid, obj_plist) in objects_map {
            if let Some(obj_map) = obj_plist.as_object() {
                let obj = PbxObject::from_plist(uuid.clone(), obj_map);
                objects.insert(uuid.clone(), obj);
            }
        }

        // Validate root object
        if let Some(root_obj) = objects.get(&root_object_uuid) {
            if root_obj.isa != "PBXProject" {
                return Err(format!(
                    "Root object \"{}\" is not a PBXProject (isa: {})",
                    root_object_uuid, root_obj.isa
                ));
            }
        } else {
            return Err(format!(
                "Root object \"{}\" not found in objects",
                root_object_uuid
            ));
        }

        Ok(XcodeProject {
            archive_version,
            object_version,
            classes,
            root_object_uuid,
            objects,
            file_path: None,
        })
    }

    /// Convert the project to a PlistValue for serialization.
    pub fn to_plist(&self) -> PlistValue {
        let mut root = IndexMap::new();
        root.insert(
            "archiveVersion".to_string(),
            PlistValue::Integer(self.archive_version),
        );
        root.insert(
            "classes".to_string(),
            PlistValue::Object(self.classes.clone()),
        );
        root.insert(
            "objectVersion".to_string(),
            PlistValue::Integer(self.object_version),
        );

        // Build objects map
        let mut objects = IndexMap::new();
        for (uuid, obj) in &self.objects {
            objects.insert(uuid.clone(), PlistValue::Object(obj.to_plist()));
        }
        root.insert("objects".to_string(), PlistValue::Object(objects));
        root.insert(
            "rootObject".to_string(),
            PlistValue::String(self.root_object_uuid.clone()),
        );

        PlistValue::Object(root)
    }

    /// Serialize to .pbxproj format.
    pub fn to_pbxproj(&self) -> String {
        serializer::build(&self.to_plist())
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<serde_json::Value, String> {
        let plist = self.to_plist();
        serde_json::to_value(&plist).map_err(|e| e.to_string())
    }

    /// Write the project to its original file.
    pub fn save(&self) -> Result<(), String> {
        let path = self
            .file_path
            .as_ref()
            .ok_or("No file path set")?;
        let output = self.to_pbxproj();
        std::fs::write(path, output).map_err(|e| e.to_string())
    }

    /// Get the file path this project was loaded from.
    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    /// Get the project root directory (parent of *.xcodeproj).
    pub fn get_project_root(&self) -> Option<String> {
        self.file_path.as_ref().map(|p| {
            Path::new(p)
                .parent() // project.pbxproj
                .and_then(|p| p.parent()) // *.xcodeproj
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default()
        })
    }

    // ── Object access ──────────────────────────────────────────────────

    /// Get a reference to an object by UUID.
    pub fn get_object(&self, uuid: &str) -> Option<&PbxObject> {
        self.objects.get(uuid)
    }

    /// Get a mutable reference to an object by UUID.
    pub fn get_object_mut(&mut self, uuid: &str) -> Option<&mut PbxObject> {
        self.objects.get_mut(uuid)
    }

    /// Get the root PBXProject object.
    pub fn root_object(&self) -> Option<&PbxObject> {
        self.objects.get(&self.root_object_uuid)
    }

    /// Get a mutable reference to the root PBXProject object.
    pub fn root_object_mut(&mut self) -> Option<&mut PbxObject> {
        self.objects.get_mut(&self.root_object_uuid)
    }

    /// Iterate over all objects.
    pub fn objects(&self) -> impl Iterator<Item = (&String, &PbxObject)> {
        self.objects.iter()
    }

    /// Iterate over all objects mutably.
    pub fn objects_mut(&mut self) -> impl Iterator<Item = (&String, &mut PbxObject)> {
        self.objects.iter_mut()
    }

    /// Get all objects with a specific ISA type.
    pub fn objects_by_isa(&self, isa: &str) -> Vec<&PbxObject> {
        self.objects
            .values()
            .filter(|obj| obj.isa == isa)
            .collect()
    }

    /// Get all native targets.
    pub fn native_targets(&self) -> Vec<&PbxObject> {
        self.objects_by_isa("PBXNativeTarget")
    }

    /// Find objects that reference a given UUID.
    pub fn get_referrers(&self, uuid: &str) -> Vec<&PbxObject> {
        self.objects
            .values()
            .filter(|obj| obj.is_referencing(uuid))
            .collect()
    }

    /// Generate a unique UUID for the project.
    pub fn get_unique_id(&self, seed: &str) -> String {
        let existing: HashSet<String> = self.objects.keys().cloned().collect();
        generate_uuid(seed, &existing)
    }

    /// Create a new object and add it to the project.
    pub fn create_object(&mut self, props: IndexMap<String, PlistValue>) -> String {
        let seed = serde_json::to_string(&props).unwrap_or_default();
        let uuid = self.get_unique_id(&seed);
        let obj = PbxObject::from_plist(uuid.clone(), &props);
        self.objects.insert(uuid.clone(), obj);
        uuid
    }

    /// Delete an object by UUID.
    pub fn delete_object(&mut self, uuid: &str) -> Option<PbxObject> {
        self.objects.shift_remove(uuid)
    }

    /// Remove an object and all references to it.
    pub fn remove_object(&mut self, uuid: &str) {
        self.delete_object(uuid);
        // Remove references from all other objects
        let keys: Vec<String> = self.objects.keys().cloned().collect();
        for key in keys {
            if let Some(obj) = self.objects.get_mut(&key) {
                obj.remove_reference(uuid);
            }
        }
    }

    // ── Validation ──────────────────────────────────────────────────────

    /// Find all orphaned references in the project.
    ///
    /// Returns a list of references where an object points to a UUID
    /// that doesn't exist in the objects map.
    pub fn find_orphaned_references(&self) -> Vec<OrphanedReference> {
        let mut orphans = Vec::new();

        for (uuid, obj) in &self.objects {
            for key in obj.reference_keys() {
                if let Some(value) = obj.props.get(key) {
                    match value {
                        PlistValue::String(ref_uuid) if !ref_uuid.is_empty() => {
                            if !self.objects.contains_key(ref_uuid) {
                                orphans.push(OrphanedReference {
                                    referrer_uuid: uuid.clone(),
                                    referrer_isa: obj.isa.clone(),
                                    property: key.to_string(),
                                    orphan_uuid: ref_uuid.clone(),
                                });
                            }
                        }
                        PlistValue::Array(items) => {
                            for item in items {
                                if let Some(ref_uuid) = item.as_str() {
                                    if !ref_uuid.is_empty()
                                        && !self.objects.contains_key(ref_uuid)
                                    {
                                        orphans.push(OrphanedReference {
                                            referrer_uuid: uuid.clone(),
                                            referrer_isa: obj.isa.clone(),
                                            property: key.to_string(),
                                            orphan_uuid: ref_uuid.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        orphans
    }

    // ── High-level helpers ─────────────────────────────────────────────

    /// Get the main group UUID from the root object.
    pub fn main_group_uuid(&self) -> Option<String> {
        self.root_object()
            .and_then(|root| root.get_str("mainGroup"))
            .map(|s| s.to_string())
    }

    /// Get the product ref group UUID from the root object.
    pub fn product_ref_group_uuid(&self) -> Option<String> {
        self.root_object()
            .and_then(|root| root.get_str("productRefGroup"))
            .map(|s| s.to_string())
    }

    /// Get the build configuration list UUID for the project.
    pub fn build_configuration_list_uuid(&self) -> Option<String> {
        self.root_object()
            .and_then(|root| root.get_str("buildConfigurationList"))
            .map(|s| s.to_string())
    }

    /// Get all target UUIDs from the root project.
    pub fn target_uuids(&self) -> Vec<String> {
        self.root_object()
            .and_then(|root| root.get_array("targets"))
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find a native target by product type.
    pub fn find_target_by_product_type(&self, product_type: &str) -> Option<&PbxObject> {
        for uuid in self.target_uuids() {
            if let Some(target) = self.get_object(&uuid) {
                if target.isa == "PBXNativeTarget"
                    && target.get_str("productType") == Some(product_type)
                {
                    return Some(target);
                }
            }
        }
        None
    }

    /// Find the main app target (heuristic based on deployment target).
    pub fn find_main_app_target(&self, platform: &str) -> Option<&PbxObject> {
        let deployment_key = match platform {
            "ios" => "IPHONEOS_DEPLOYMENT_TARGET",
            "macos" => "MACOSX_DEPLOYMENT_TARGET",
            "tvos" => "TVOS_DEPLOYMENT_TARGET",
            "watchos" => "WATCHOS_DEPLOYMENT_TARGET",
            "visionos" => "XROS_DEPLOYMENT_TARGET",
            _ => return None,
        };

        let app_targets: Vec<&PbxObject> = self
            .target_uuids()
            .iter()
            .filter_map(|uuid| self.get_object(uuid))
            .filter(|t| {
                t.isa == "PBXNativeTarget"
                    && t.get_str("productType") == Some("com.apple.product-type.application")
            })
            .collect();

        // Filter by deployment target build setting
        for target in &app_targets {
            if let Some(config_list_uuid) = target.get_str("buildConfigurationList") {
                if let Some(config_list) = self.get_object(config_list_uuid) {
                    if let Some(configs) = config_list.get_array("buildConfigurations") {
                        for config_val in configs {
                            if let Some(config_uuid) = config_val.as_str() {
                                if let Some(config) = self.get_object(config_uuid) {
                                    if let Some(build_settings) = config.get_object("buildSettings")
                                    {
                                        if build_settings.contains_key(deployment_key) {
                                            return Some(*target);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback: return the first app target
        app_targets.into_iter().next()
    }

    /// Find a build phase of a specific type for a target.
    pub fn find_build_phase(&self, target_uuid: &str, phase_isa: &str) -> Option<&PbxObject> {
        let target = self.get_object(target_uuid)?;
        let phases = target.get_array("buildPhases")?;
        for phase_val in phases {
            if let Some(phase_uuid) = phase_val.as_str() {
                if let Some(phase) = self.get_object(phase_uuid) {
                    if phase.isa == phase_isa {
                        return Some(phase);
                    }
                }
            }
        }
        None
    }

    /// Get the default build configuration for a configuration list.
    pub fn get_default_configuration(&self, config_list_uuid: &str) -> Option<&PbxObject> {
        let config_list = self.get_object(config_list_uuid)?;
        let default_name = config_list.get_str("defaultConfigurationName")?;
        let configs = config_list.get_array("buildConfigurations")?;

        for config_val in configs {
            if let Some(config_uuid) = config_val.as_str() {
                if let Some(config) = self.get_object(config_uuid) {
                    if config.get_str("name") == Some(default_name) {
                        return Some(config);
                    }
                }
            }
        }

        // Fallback: first configuration
        configs
            .first()
            .and_then(|v| v.as_str())
            .and_then(|uuid| self.get_object(uuid))
    }

    /// Get a build setting value from a target's default configuration.
    pub fn get_build_setting(&self, target_uuid: &str, key: &str) -> Option<PlistValue> {
        let target = self.get_object(target_uuid)?;
        let config_list_uuid = target.get_str("buildConfigurationList")?;
        let config = self.get_default_configuration(config_list_uuid)?;
        let build_settings = config.get_object("buildSettings")?;
        build_settings.get(key).cloned()
    }

    /// Set a build setting on all configurations for a target.
    pub fn set_build_setting(
        &mut self,
        target_uuid: &str,
        key: &str,
        value: PlistValue,
    ) -> bool {
        let target = match self.get_object(target_uuid) {
            Some(t) => t,
            None => return false,
        };
        let config_list_uuid = match target.get_str("buildConfigurationList") {
            Some(s) => s.to_string(),
            None => return false,
        };
        let config_list = match self.get_object(&config_list_uuid) {
            Some(c) => c,
            None => return false,
        };
        let config_uuids: Vec<String> = config_list
            .get_array("buildConfigurations")
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        for config_uuid in config_uuids {
            if let Some(config) = self.get_object_mut(&config_uuid) {
                if let Some(PlistValue::Object(ref mut settings)) = config.props.get_mut("buildSettings") {
                    settings.insert(key.to_string(), value.clone());
                }
            }
        }
        true
    }

    /// Remove a build setting from all configurations for a target.
    pub fn remove_build_setting(&mut self, target_uuid: &str, key: &str) -> bool {
        let target = match self.get_object(target_uuid) {
            Some(t) => t,
            None => return false,
        };
        let config_list_uuid = match target.get_str("buildConfigurationList") {
            Some(s) => s.to_string(),
            None => return false,
        };
        let config_list = match self.get_object(&config_list_uuid) {
            Some(c) => c,
            None => return false,
        };
        let config_uuids: Vec<String> = config_list
            .get_array("buildConfigurations")
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        for config_uuid in config_uuids {
            if let Some(config) = self.get_object_mut(&config_uuid) {
                if let Some(PlistValue::Object(ref mut settings)) = config.props.get_mut("buildSettings") {
                    settings.shift_remove(key);
                }
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    const FIXTURES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/__test__/fixtures");

    #[test]
    fn test_open_project() {
        let path = Path::new(FIXTURES_DIR).join("project.pbxproj");
        let content = fs::read_to_string(&path).unwrap();
        let project = XcodeProject::from_plist(&content).unwrap();

        assert_eq!(project.archive_version, 1);
        assert_eq!(project.object_version, 46);
        assert!(!project.root_object_uuid.is_empty());
        assert!(project.root_object().is_some());
    }

    #[test]
    fn test_objects_by_isa() {
        let path = Path::new(FIXTURES_DIR).join("project.pbxproj");
        let content = fs::read_to_string(&path).unwrap();
        let project = XcodeProject::from_plist(&content).unwrap();

        let targets = project.native_targets();
        assert!(!targets.is_empty());

        let groups = project.objects_by_isa("PBXGroup");
        assert!(!groups.is_empty());
    }

    #[test]
    fn test_get_referrers() {
        let path = Path::new(FIXTURES_DIR).join("project.pbxproj");
        let content = fs::read_to_string(&path).unwrap();
        let project = XcodeProject::from_plist(&content).unwrap();

        // The root object's mainGroup should be referenced by the root object
        if let Some(main_group_uuid) = project.main_group_uuid() {
            let referrers = project.get_referrers(&main_group_uuid);
            assert!(!referrers.is_empty());
        }
    }

    #[test]
    fn test_roundtrip_via_xcode_project() {
        let path = Path::new(FIXTURES_DIR).join("project.pbxproj");
        let original = fs::read_to_string(&path).unwrap();
        let project = XcodeProject::from_plist(&original).unwrap();
        let output = project.to_pbxproj();
        assert_eq!(output, original);
    }

    #[test]
    fn test_unique_id() {
        let path = Path::new(FIXTURES_DIR).join("project.pbxproj");
        let content = fs::read_to_string(&path).unwrap();
        let project = XcodeProject::from_plist(&content).unwrap();

        let id1 = project.get_unique_id("test-seed");
        let id2 = project.get_unique_id("test-seed");
        assert_eq!(id1, id2); // Same seed, same result
        assert_eq!(id1.len(), 24);

        let id3 = project.get_unique_id("different-seed");
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_find_target() {
        let path = Path::new(FIXTURES_DIR).join("project.pbxproj");
        let content = fs::read_to_string(&path).unwrap();
        let project = XcodeProject::from_plist(&content).unwrap();

        let target = project.find_target_by_product_type("com.apple.product-type.application");
        assert!(target.is_some());
    }

    #[test]
    fn test_clean_project_has_no_orphans() {
        let path = Path::new(FIXTURES_DIR).join("project.pbxproj");
        let content = fs::read_to_string(&path).unwrap();
        let project = XcodeProject::from_plist(&content).unwrap();

        let orphans = project.find_orphaned_references();
        assert!(
            orphans.is_empty(),
            "Clean project should have no orphans, found: {:?}",
            orphans
                .iter()
                .map(|o| format!(
                    "{} > {}.{} > {}",
                    o.referrer_uuid, o.referrer_isa, o.property, o.orphan_uuid
                ))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_malformed_project_detects_orphans() {
        let path = Path::new(FIXTURES_DIR).join("malformed.pbxproj");
        let content = fs::read_to_string(&path).unwrap();
        let project = XcodeProject::from_plist(&content).unwrap();

        let orphans = project.find_orphaned_references();
        assert!(
            !orphans.is_empty(),
            "Malformed project should have orphaned references"
        );

        // The known orphan: 3E1C2299F05049539341855D in PBXResourcesBuildPhase.files
        let known_orphan = orphans.iter().find(|o| o.orphan_uuid == "3E1C2299F05049539341855D");
        assert!(
            known_orphan.is_some(),
            "Should detect orphaned UUID 3E1C2299F05049539341855D"
        );
        let orphan = known_orphan.unwrap();
        assert_eq!(orphan.referrer_isa, "PBXResourcesBuildPhase");
        assert_eq!(orphan.property, "files");
    }

    #[test]
    fn test_malformed_project_still_parses() {
        // Malformed projects should parse and round-trip without crashing
        let path = Path::new(FIXTURES_DIR).join("malformed.pbxproj");
        let content = fs::read_to_string(&path).unwrap();
        let project = XcodeProject::from_plist(&content).unwrap();

        // Should still have valid structure
        assert!(project.root_object().is_some());
        assert!(!project.native_targets().is_empty());

        // Should be able to serialize without crashing
        let output = project.to_pbxproj();
        assert!(output.contains("PBXResourcesBuildPhase"));
    }
}
