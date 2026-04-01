use std::borrow::Cow;
use std::collections::HashSet;
use std::path::Path;

use indexmap::IndexMap;

use crate::objects::{PbxObject, PbxObjectExt};
use crate::parser;
use crate::types::plist::{PlistMap, PlistObject, PlistValue};
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
    pub classes: PlistObject<'static>,
    pub root_object_uuid: String,
    objects: IndexMap<String, PbxObject>,
    file_path: Option<String>,
}

impl XcodeProject {
    /// Open and parse a .pbxproj file from disk.
    pub fn open(file_path: &str) -> Result<Self, String> {
        let contents = std::fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;
        let mut project = Self::from_plist(&contents)?;
        project.file_path = Some(file_path.to_string());
        Ok(project)
    }

    /// Parse a .pbxproj string into an XcodeProject.
    pub fn from_plist(text: &str) -> Result<Self, String> {
        let plist = parser::parse(text)?.into_owned();
        Self::from_plist_value(&plist)
    }

    /// Create from an already-parsed PlistValue.
    pub fn from_plist_value(plist: &PlistValue<'static>) -> Result<Self, String> {
        plist.as_object().ok_or("Root must be an object")?;

        let archive_version = plist.get("archiveVersion").and_then(|v| v.as_integer()).unwrap_or(1);

        let object_version = plist.get("objectVersion").and_then(|v| v.as_integer()).unwrap_or(46);

        let classes = plist
            .get("classes")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let root_object_uuid = plist
            .get("rootObject")
            .and_then(|v| v.as_str())
            .ok_or("rootObject is required")?
            .to_string();

        let objects_pairs = plist
            .get("objects")
            .and_then(|v| v.as_object())
            .ok_or("objects is required")?;

        // Inflate all objects
        let mut objects = IndexMap::new();
        for (uuid, obj_plist) in objects_pairs {
            if let Some(obj_pairs) = obj_plist.as_object() {
                let obj = PbxObject::from_plist(uuid.to_string(), obj_pairs);
                objects.insert(uuid.to_string(), obj);
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
            return Err(format!("Root object \"{}\" not found in objects", root_object_uuid));
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
    pub fn to_plist(&self) -> PlistValue<'static> {
        let mut objects_pairs: PlistObject<'static> = Vec::new();
        for (uuid, obj) in &self.objects {
            objects_pairs.push((Cow::Owned(uuid.clone()), PlistValue::Object(obj.to_plist())));
        }

        let root: PlistObject<'static> = vec![
            (Cow::Owned("archiveVersion".to_string()), PlistValue::Integer(self.archive_version)),
            (Cow::Owned("classes".to_string()), PlistValue::Object(self.classes.clone())),
            (Cow::Owned("objectVersion".to_string()), PlistValue::Integer(self.object_version)),
            (Cow::Owned("objects".to_string()), PlistValue::Object(objects_pairs)),
            (
                Cow::Owned("rootObject".to_string()),
                PlistValue::String(Cow::Owned(self.root_object_uuid.clone())),
            ),
        ];

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
        let path = self.file_path.as_ref().ok_or("No file path set")?;
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
        self.objects.values().filter(|obj| obj.isa == isa).collect()
    }

    /// Get all native targets.
    pub fn native_targets(&self) -> Vec<&PbxObject> {
        self.objects_by_isa("PBXNativeTarget")
    }

    /// Find objects that reference a given UUID.
    pub fn get_referrers(&self, uuid: &str) -> Vec<&PbxObject> {
        self.objects.values().filter(|obj| obj.is_referencing(uuid)).collect()
    }

    /// Generate a unique UUID for the project.
    pub fn get_unique_id(&self, seed: &str) -> String {
        let existing: HashSet<String> = self.objects.keys().cloned().collect();
        generate_uuid(seed, &existing)
    }

    /// Create a new object and add it to the project.
    pub fn create_object(&mut self, props: PlistMap<'static>) -> String {
        let seed = serde_json::to_string(&props).unwrap_or_default();
        let uuid = self.get_unique_id(&seed);
        let pairs: PlistObject<'static> = props.into_iter().collect();
        let obj = PbxObject::from_plist(uuid.clone(), &pairs);
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
                            if !self.objects.contains_key(&**ref_uuid) {
                                orphans.push(OrphanedReference {
                                    referrer_uuid: uuid.clone(),
                                    referrer_isa: obj.isa.clone(),
                                    property: key.to_string(),
                                    orphan_uuid: ref_uuid.to_string(),
                                });
                            }
                        }
                        PlistValue::Array(items) => {
                            for item in items {
                                if let Some(ref_uuid) = item.as_str() {
                                    if !ref_uuid.is_empty() && !self.objects.contains_key(ref_uuid) {
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
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default()
    }

    /// Find a native target by product type.
    pub fn find_target_by_product_type(&self, product_type: &str) -> Option<&PbxObject> {
        for uuid in self.target_uuids() {
            if let Some(target) = self.get_object(&uuid) {
                if target.isa == "PBXNativeTarget" && target.get_str("productType") == Some(product_type) {
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
                t.isa == "PBXNativeTarget" && t.get_str("productType") == Some("com.apple.product-type.application")
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
                                    if let Some(build_settings) = config.get_object("buildSettings") {
                                        if build_settings.iter().any(|(k, _)| k.as_ref() == deployment_key) {
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
    pub fn get_build_setting(&self, target_uuid: &str, key: &str) -> Option<PlistValue<'static>> {
        let target = self.get_object(target_uuid)?;
        let config_list_uuid = target.get_str("buildConfigurationList")?;
        let config = self.get_default_configuration(config_list_uuid)?;
        let build_settings = config.get_object("buildSettings")?;
        build_settings.iter().find(|(k, _)| k.as_ref() == key).map(|(_, v)| v.clone())
    }

    /// Set a build setting on all configurations for a target.
    pub fn set_build_setting(&mut self, target_uuid: &str, key: &str, value: PlistValue<'static>) -> bool {
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
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        for config_uuid in config_uuids {
            if let Some(config) = self.get_object_mut(&config_uuid) {
                if let Some(PlistValue::Object(ref mut settings)) = config.props.get_mut("buildSettings") {
                    if let Some(pos) = settings.iter().position(|(k, _)| k.as_ref() == key) {
                        settings[pos].1 = value.clone();
                    } else {
                        settings.push((Cow::Owned(key.to_string()), value.clone()));
                    }
                }
            }
        }
        true
    }

    // ── File & group operations ──────────────────────────────────────

    /// Get children UUIDs of a group.
    pub fn get_group_children(&self, group_uuid: &str) -> Vec<String> {
        self.get_object(group_uuid)
            .and_then(|obj| obj.get_array("children"))
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default()
    }

    /// Add a file reference to the project and a group.
    /// Returns the UUID of the new PBXFileReference.
    pub fn add_file(&mut self, group_uuid: &str, path: &str) -> Option<String> {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let file_type = crate::types::constants::FILE_TYPES_BY_EXTENSION
            .get(ext)
            .copied()
            .unwrap_or("file");

        let source_tree = crate::types::constants::SOURCETREE_BY_FILETYPE
            .get(file_type)
            .copied()
            .unwrap_or("<group>");

        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path);

        let mut props = PlistMap::default();
        props.insert(Cow::Owned("isa".to_string()), PlistValue::String(Cow::Owned("PBXFileReference".to_string())));
        props.insert(Cow::Owned("fileEncoding".to_string()), PlistValue::Integer(4));
        props.insert(
            Cow::Owned("lastKnownFileType".to_string()),
            PlistValue::String(Cow::Owned(file_type.to_string())),
        );
        if name != path {
            props.insert(Cow::Owned("name".to_string()), PlistValue::String(Cow::Owned(name.to_string())));
        }
        props.insert(Cow::Owned("path".to_string()), PlistValue::String(Cow::Owned(path.to_string())));
        props.insert(Cow::Owned("sourceTree".to_string()), PlistValue::String(Cow::Owned(source_tree.to_string())));

        let file_uuid = self.create_object(props);

        // Add to group's children
        if let Some(group) = self.get_object_mut(group_uuid) {
            if let Some(PlistValue::Array(ref mut children)) = group.props.get_mut("children") {
                children.push(PlistValue::String(Cow::Owned(file_uuid.clone())));
            }
        }

        Some(file_uuid)
    }

    /// Create a group and add it as a child of a parent group.
    /// Returns the UUID of the new PBXGroup.
    pub fn add_group(&mut self, parent_uuid: &str, name: &str) -> Option<String> {
        let mut props = PlistMap::default();
        props.insert(Cow::Owned("isa".to_string()), PlistValue::String(Cow::Owned("PBXGroup".to_string())));
        props.insert(Cow::Owned("children".to_string()), PlistValue::Array(vec![]));
        props.insert(Cow::Owned("name".to_string()), PlistValue::String(Cow::Owned(name.to_string())));
        props.insert(Cow::Owned("sourceTree".to_string()), PlistValue::String(Cow::Owned("<group>".to_string())));

        let group_uuid = self.create_object(props);

        if let Some(parent) = self.get_object_mut(parent_uuid) {
            if let Some(PlistValue::Array(ref mut children)) = parent.props.get_mut("children") {
                children.push(PlistValue::String(Cow::Owned(group_uuid.clone())));
            }
        }

        Some(group_uuid)
    }

    /// Remove a file reference from the project.
    /// Removes it from its parent group's children array and deletes the object.
    /// Also removes any PBXBuildFile that references it.
    pub fn remove_file(&mut self, file_ref_uuid: &str) -> bool {
        if self.get_object(file_ref_uuid).is_none() {
            return false;
        }

        // Remove from parent group children
        let group_uuids: Vec<String> = self.objects_by_isa("PBXGroup")
            .iter()
            .chain(self.objects_by_isa("PBXVariantGroup").iter())
            .filter(|g| {
                g.get_array("children")
                    .map(|c| c.iter().any(|v| v.as_str() == Some(file_ref_uuid)))
                    .unwrap_or(false)
            })
            .map(|g| g.uuid.clone())
            .collect();

        for group_uuid in &group_uuids {
            if let Some(group) = self.get_object_mut(group_uuid) {
                if let Some(PlistValue::Array(ref mut children)) = group.props.get_mut("children") {
                    children.retain(|v| v.as_str() != Some(file_ref_uuid));
                }
            }
        }

        // Remove any PBXBuildFile referencing this file
        let build_file_uuids: Vec<String> = self.objects_by_isa("PBXBuildFile")
            .iter()
            .filter(|bf| bf.get_str("fileRef") == Some(file_ref_uuid))
            .map(|bf| bf.uuid.clone())
            .collect();

        for bf_uuid in &build_file_uuids {
            self.remove_object(bf_uuid);
        }

        self.remove_object(file_ref_uuid);
        true
    }

    /// Remove a group from the project.
    /// Removes it from its parent group's children array and deletes the group object.
    /// Does NOT remove the group's children objects (files stay in the project).
    pub fn remove_group(&mut self, group_uuid: &str) -> bool {
        if self.get_object(group_uuid).is_none() {
            return false;
        }

        // Remove from parent group children
        let parent_uuids: Vec<String> = self.objects_by_isa("PBXGroup")
            .iter()
            .filter(|g| {
                g.uuid != group_uuid
                    && g.get_array("children")
                        .map(|c| c.iter().any(|v| v.as_str() == Some(group_uuid)))
                        .unwrap_or(false)
            })
            .map(|g| g.uuid.clone())
            .collect();

        for parent_uuid in &parent_uuids {
            if let Some(parent) = self.get_object_mut(parent_uuid) {
                if let Some(PlistValue::Array(ref mut children)) = parent.props.get_mut("children") {
                    children.retain(|v| v.as_str() != Some(group_uuid));
                }
            }
        }

        self.remove_object(group_uuid);
        true
    }

    // ── Swift Package Manager operations ─────────────────────────────

    /// Add a remote Swift package reference to the project.
    /// Returns the UUID of the new XCRemoteSwiftPackageReference.
    pub fn add_remote_swift_package(&mut self, url: &str, version: &str) -> Option<String> {
        let mut props = PlistMap::default();
        props.insert(Cow::Owned("isa".to_string()), PlistValue::String(Cow::Owned("XCRemoteSwiftPackageReference".to_string())));
        props.insert(Cow::Owned("repositoryURL".to_string()), PlistValue::String(Cow::Owned(url.to_string())));

        let requirement: PlistObject<'static> = vec![
            (Cow::Owned("kind".to_string()), PlistValue::String(Cow::Owned("upToNextMajorVersion".to_string()))),
            (Cow::Owned("minimumVersion".to_string()), PlistValue::String(Cow::Owned(version.to_string()))),
        ];
        props.insert(Cow::Owned("requirement".to_string()), PlistValue::Object(requirement));

        let package_uuid = self.create_object(props);

        let root_uuid = self.root_object_uuid.clone();
        if let Some(root) = self.get_object_mut(&root_uuid) {
            match root.props.get_mut("packageReferences") {
                Some(PlistValue::Array(ref mut refs)) => {
                    refs.push(PlistValue::String(Cow::Owned(package_uuid.clone())));
                }
                _ => {
                    root.props.insert(
                        Cow::Owned("packageReferences".to_string()),
                        PlistValue::Array(vec![PlistValue::String(Cow::Owned(package_uuid.clone()))]),
                    );
                }
            }
        }

        Some(package_uuid)
    }

    /// Add a local Swift package reference to the project.
    /// Returns the UUID of the new XCLocalSwiftPackageReference.
    pub fn add_local_swift_package(&mut self, relative_path: &str) -> Option<String> {
        let mut props = PlistMap::default();
        props.insert(Cow::Owned("isa".to_string()), PlistValue::String(Cow::Owned("XCLocalSwiftPackageReference".to_string())));
        props.insert(Cow::Owned("relativePath".to_string()), PlistValue::String(Cow::Owned(relative_path.to_string())));

        let package_uuid = self.create_object(props);

        let root_uuid = self.root_object_uuid.clone();
        if let Some(root) = self.get_object_mut(&root_uuid) {
            match root.props.get_mut("packageReferences") {
                Some(PlistValue::Array(ref mut refs)) => {
                    refs.push(PlistValue::String(Cow::Owned(package_uuid.clone())));
                }
                _ => {
                    root.props.insert(
                        Cow::Owned("packageReferences".to_string()),
                        PlistValue::Array(vec![PlistValue::String(Cow::Owned(package_uuid.clone()))]),
                    );
                }
            }
        }

        Some(package_uuid)
    }

    /// Add a Swift package product dependency to a target.
    /// Creates XCSwiftPackageProductDependency + PBXBuildFile, wires to target
    /// and Frameworks build phase.
    /// Returns the UUID of the XCSwiftPackageProductDependency.
    pub fn add_swift_package_product(
        &mut self,
        target_uuid: &str,
        product_name: &str,
        package_uuid: &str,
    ) -> Option<String> {
        let mut dep_props = PlistMap::default();
        dep_props.insert(Cow::Owned("isa".to_string()), PlistValue::String(Cow::Owned("XCSwiftPackageProductDependency".to_string())));
        dep_props.insert(Cow::Owned("package".to_string()), PlistValue::String(Cow::Owned(package_uuid.to_string())));
        dep_props.insert(Cow::Owned("productName".to_string()), PlistValue::String(Cow::Owned(product_name.to_string())));

        let dep_uuid = self.create_object(dep_props);

        if let Some(target) = self.get_object_mut(target_uuid) {
            match target.props.get_mut("packageProductDependencies") {
                Some(PlistValue::Array(ref mut deps)) => {
                    deps.push(PlistValue::String(Cow::Owned(dep_uuid.clone())));
                }
                _ => {
                    target.props.insert(
                        Cow::Owned("packageProductDependencies".to_string()),
                        PlistValue::Array(vec![PlistValue::String(Cow::Owned(dep_uuid.clone()))]),
                    );
                }
            }
        }

        let mut bf_props = PlistMap::default();
        bf_props.insert(Cow::Owned("isa".to_string()), PlistValue::String(Cow::Owned("PBXBuildFile".to_string())));
        bf_props.insert(Cow::Owned("productRef".to_string()), PlistValue::String(Cow::Owned(dep_uuid.clone())));
        let bf_uuid = self.create_object(bf_props);

        let phase_uuid = self.ensure_build_phase(target_uuid, "PBXFrameworksBuildPhase")?;
        if let Some(phase) = self.get_object_mut(&phase_uuid) {
            if let Some(PlistValue::Array(ref mut files)) = phase.props.get_mut("files") {
                files.push(PlistValue::String(Cow::Owned(bf_uuid)));
            }
        }

        Some(dep_uuid)
    }

    /// Remove a Swift package product dependency from a target.
    pub fn remove_swift_package_product(&mut self, target_uuid: &str, product_name: &str) -> bool {
        let dep_uuid = {
            let target = match self.get_object(target_uuid) {
                Some(t) => t,
                None => return false,
            };
            let deps = match target.props.get("packageProductDependencies") {
                Some(PlistValue::Array(arr)) => arr,
                _ => return false,
            };
            let mut found = None;
            for dep_val in deps {
                if let Some(uuid) = dep_val.as_str() {
                    if let Some(dep_obj) = self.get_object(uuid) {
                        if dep_obj.get_str("productName") == Some(product_name) {
                            found = Some(uuid.to_string());
                            break;
                        }
                    }
                }
            }
            match found {
                Some(u) => u,
                None => return false,
            }
        };

        if let Some(target) = self.get_object_mut(target_uuid) {
            if let Some(PlistValue::Array(ref mut deps)) = target.props.get_mut("packageProductDependencies") {
                deps.retain(|v| v.as_str() != Some(&dep_uuid));
            }
        }

        let bf_uuids: Vec<String> = self.objects_by_isa("PBXBuildFile")
            .iter()
            .filter(|bf| bf.get_str("productRef") == Some(&dep_uuid))
            .map(|bf| bf.uuid.clone())
            .collect();
        for bf in &bf_uuids {
            self.remove_object(bf);
        }

        self.remove_object(&dep_uuid);
        true
    }

    /// List all Swift package references in the project.
    pub fn list_swift_packages(&self) -> Vec<(String, String, String)> {
        let root = match self.root_object() {
            Some(r) => r,
            None => return vec![],
        };
        let refs = match root.get_array("packageReferences") {
            Some(r) => r,
            None => return vec![],
        };

        let mut result = Vec::new();
        for ref_val in refs {
            if let Some(uuid) = ref_val.as_str() {
                if let Some(obj) = self.get_object(uuid) {
                    let kind = obj.isa.clone();
                    let location = obj.get_str("repositoryURL")
                        .or_else(|| obj.get_str("relativePath"))
                        .unwrap_or("")
                        .to_string();
                    result.push((uuid.to_string(), kind, location));
                }
            }
        }
        result
    }

    // ── Build phase operations ─────────────────────────────────────

    /// Add a build file to a build phase (e.g. adding a source file to the Sources phase).
    /// Returns the UUID of the new PBXBuildFile.
    pub fn add_build_file(&mut self, phase_uuid: &str, file_ref_uuid: &str) -> Option<String> {
        let mut props = PlistMap::default();
        props.insert(Cow::Owned("isa".to_string()), PlistValue::String(Cow::Owned("PBXBuildFile".to_string())));
        props.insert(Cow::Owned("fileRef".to_string()), PlistValue::String(Cow::Owned(file_ref_uuid.to_string())));

        let build_file_uuid = self.create_object(props);

        if let Some(phase) = self.get_object_mut(phase_uuid) {
            if let Some(PlistValue::Array(ref mut files)) = phase.props.get_mut("files") {
                files.push(PlistValue::String(Cow::Owned(build_file_uuid.clone())));
            }
        }

        Some(build_file_uuid)
    }

    /// Find or create a build phase of a given type for a target.
    /// Returns the UUID of the build phase.
    pub fn ensure_build_phase(&mut self, target_uuid: &str, phase_isa: &str) -> Option<String> {
        // Check if it already exists
        if let Some(existing) = self.find_build_phase(target_uuid, phase_isa) {
            return Some(existing.uuid.clone());
        }

        // Create new phase
        let mut props = PlistMap::default();
        props.insert(Cow::Owned("isa".to_string()), PlistValue::String(Cow::Owned(phase_isa.to_string())));
        props.insert(Cow::Owned("buildActionMask".to_string()), PlistValue::Integer(2147483647));
        props.insert(Cow::Owned("files".to_string()), PlistValue::Array(vec![]));
        props.insert(Cow::Owned("runOnlyForDeploymentPostprocessing".to_string()), PlistValue::Integer(0));

        let phase_uuid = self.create_object(props);

        // Add to target's buildPhases
        if let Some(target) = self.get_object_mut(target_uuid) {
            if let Some(PlistValue::Array(ref mut phases)) = target.props.get_mut("buildPhases") {
                phases.push(PlistValue::String(Cow::Owned(phase_uuid.clone())));
            }
        }

        Some(phase_uuid)
    }

    /// Add a framework to a target (creates file reference + build file + adds to Frameworks phase).
    /// Returns the UUID of the PBXBuildFile.
    pub fn add_framework(&mut self, target_uuid: &str, framework_name: &str) -> Option<String> {
        let name = if framework_name.ends_with(".framework") {
            framework_name.to_string()
        } else {
            format!("{}.framework", framework_name)
        };

        let path = format!("System/Library/Frameworks/{}", name);

        // Create PBXFileReference for the framework
        let mut file_props = PlistMap::default();
        file_props.insert(Cow::Owned("isa".to_string()), PlistValue::String(Cow::Owned("PBXFileReference".to_string())));
        file_props.insert(
            Cow::Owned("lastKnownFileType".to_string()),
            PlistValue::String(Cow::Owned("wrapper.framework".to_string())),
        );
        file_props.insert(Cow::Owned("name".to_string()), PlistValue::String(Cow::Owned(name.clone())));
        file_props.insert(Cow::Owned("path".to_string()), PlistValue::String(Cow::Owned(path)));
        file_props.insert(Cow::Owned("sourceTree".to_string()), PlistValue::String(Cow::Owned("SDKROOT".to_string())));

        let file_ref_uuid = self.create_object(file_props);

        // Ensure Frameworks build phase exists
        let phase_uuid = self.ensure_build_phase(target_uuid, "PBXFrameworksBuildPhase")?;

        // Add build file
        self.add_build_file(&phase_uuid, &file_ref_uuid)
    }

    // ── Target operations ──────────────────────────────────────────

    /// Add a dependency from one target to another.
    /// Returns the UUID of the PBXTargetDependency.
    pub fn add_dependency(&mut self, target_uuid: &str, depends_on_uuid: &str) -> Option<String> {
        // Create PBXContainerItemProxy
        let mut proxy_props = PlistMap::default();
        proxy_props.insert(
            Cow::Owned("isa".to_string()),
            PlistValue::String(Cow::Owned("PBXContainerItemProxy".to_string())),
        );
        proxy_props.insert(
            Cow::Owned("containerPortal".to_string()),
            PlistValue::String(Cow::Owned(self.root_object_uuid.clone())),
        );
        proxy_props.insert(Cow::Owned("proxyType".to_string()), PlistValue::Integer(1));
        proxy_props.insert(
            Cow::Owned("remoteGlobalIDString".to_string()),
            PlistValue::String(Cow::Owned(depends_on_uuid.to_string())),
        );

        // Get name of the dependency target
        let remote_name = self
            .get_object(depends_on_uuid)
            .and_then(|t| t.get_str("name"))
            .unwrap_or("Unknown")
            .to_string();
        proxy_props.insert(Cow::Owned("remoteInfo".to_string()), PlistValue::String(Cow::Owned(remote_name)));

        let proxy_uuid = self.create_object(proxy_props);

        // Create PBXTargetDependency
        let mut dep_props = PlistMap::default();
        dep_props.insert(Cow::Owned("isa".to_string()), PlistValue::String(Cow::Owned("PBXTargetDependency".to_string())));
        dep_props.insert(Cow::Owned("target".to_string()), PlistValue::String(Cow::Owned(depends_on_uuid.to_string())));
        dep_props.insert(Cow::Owned("targetProxy".to_string()), PlistValue::String(Cow::Owned(proxy_uuid)));

        let dep_uuid = self.create_object(dep_props);

        // Add to target's dependencies
        if let Some(target) = self.get_object_mut(target_uuid) {
            if let Some(PlistValue::Array(ref mut deps)) = target.props.get_mut("dependencies") {
                deps.push(PlistValue::String(Cow::Owned(dep_uuid.clone())));
            }
        }

        Some(dep_uuid)
    }

    /// Create a native target with build configurations and standard build phases.
    /// Returns the UUID of the new PBXNativeTarget.
    ///
    /// This creates:
    /// - XCBuildConfiguration for Debug and Release
    /// - XCConfigurationList referencing those configurations
    /// - PBXSourcesBuildPhase, PBXFrameworksBuildPhase, PBXResourcesBuildPhase
    /// - PBXNativeTarget with all of the above
    /// - PBXFileReference for the product (e.g. MyApp.app)
    /// - Adds the product ref to the Products group
    /// - Adds the target to PBXProject.targets
    pub fn create_native_target(&mut self, name: &str, product_type: &str, bundle_id: &str) -> Option<String> {
        // Determine product extension from product type
        let product_ext = crate::types::constants::PRODUCT_UTI_EXTENSIONS
            .get(product_type)
            .copied()
            .unwrap_or("app");

        let product_name = if product_ext.is_empty() {
            name.to_string()
        } else {
            format!("{}.{}", name, product_ext)
        };

        // 1. Create product PBXFileReference
        let mut product_props = PlistMap::default();
        product_props.insert(Cow::Owned("isa".to_string()), PlistValue::String(Cow::Owned("PBXFileReference".to_string())));
        product_props.insert(
            Cow::Owned("explicitFileType".to_string()),
            PlistValue::String(Cow::Owned(
                crate::types::constants::FILE_TYPES_BY_EXTENSION
                    .get(product_ext)
                    .copied()
                    .unwrap_or("wrapper.application")
                    .to_string(),
            )),
        );
        product_props.insert(Cow::Owned("includeInIndex".to_string()), PlistValue::Integer(0));
        product_props.insert(Cow::Owned("path".to_string()), PlistValue::String(Cow::Owned(product_name)));
        product_props.insert(
            Cow::Owned("sourceTree".to_string()),
            PlistValue::String(Cow::Owned("BUILT_PRODUCTS_DIR".to_string())),
        );
        let product_ref_uuid = self.create_object(product_props);

        // Add product to Products group
        if let Some(products_uuid) = self.product_ref_group_uuid() {
            if let Some(products) = self.get_object_mut(&products_uuid) {
                if let Some(PlistValue::Array(ref mut children)) = products.props.get_mut("children") {
                    children.push(PlistValue::String(Cow::Owned(product_ref_uuid.clone())));
                }
            }
        }

        // 2. Create Debug build configuration
        let debug_settings: PlistObject<'static> = vec![
            (Cow::Owned("PRODUCT_BUNDLE_IDENTIFIER".to_string()), PlistValue::String(Cow::Owned(bundle_id.to_string()))),
            (Cow::Owned("PRODUCT_NAME".to_string()), PlistValue::String(Cow::Owned(name.to_string()))),
            (Cow::Owned("SWIFT_VERSION".to_string()), PlistValue::String(Cow::Owned("5.0".to_string()))),
        ];

        let mut debug_props = PlistMap::default();
        debug_props.insert(
            Cow::Owned("isa".to_string()),
            PlistValue::String(Cow::Owned("XCBuildConfiguration".to_string())),
        );
        debug_props.insert(Cow::Owned("buildSettings".to_string()), PlistValue::Object(debug_settings));
        debug_props.insert(Cow::Owned("name".to_string()), PlistValue::String(Cow::Owned("Debug".to_string())));
        let debug_uuid = self.create_object(debug_props);

        // 3. Create Release build configuration
        let release_settings: PlistObject<'static> = vec![
            (Cow::Owned("PRODUCT_BUNDLE_IDENTIFIER".to_string()), PlistValue::String(Cow::Owned(bundle_id.to_string()))),
            (Cow::Owned("PRODUCT_NAME".to_string()), PlistValue::String(Cow::Owned(name.to_string()))),
            (Cow::Owned("SWIFT_VERSION".to_string()), PlistValue::String(Cow::Owned("5.0".to_string()))),
        ];

        let mut release_props = PlistMap::default();
        release_props.insert(
            Cow::Owned("isa".to_string()),
            PlistValue::String(Cow::Owned("XCBuildConfiguration".to_string())),
        );
        release_props.insert(Cow::Owned("buildSettings".to_string()), PlistValue::Object(release_settings));
        release_props.insert(Cow::Owned("name".to_string()), PlistValue::String(Cow::Owned("Release".to_string())));
        let release_uuid = self.create_object(release_props);

        // 4. Create XCConfigurationList
        let mut config_list_props = PlistMap::default();
        config_list_props.insert(Cow::Owned("isa".to_string()), PlistValue::String(Cow::Owned("XCConfigurationList".to_string())));
        config_list_props.insert(
            Cow::Owned("buildConfigurations".to_string()),
            PlistValue::Array(vec![PlistValue::String(Cow::Owned(debug_uuid)), PlistValue::String(Cow::Owned(release_uuid))]),
        );
        config_list_props.insert(Cow::Owned("defaultConfigurationIsVisible".to_string()), PlistValue::Integer(0));
        config_list_props.insert(
            Cow::Owned("defaultConfigurationName".to_string()),
            PlistValue::String(Cow::Owned("Release".to_string())),
        );
        let config_list_uuid = self.create_object(config_list_props);

        // 5. Create standard build phases
        let sources_uuid = {
            let mut p = PlistMap::default();
            p.insert(
                Cow::Owned("isa".to_string()),
                PlistValue::String(Cow::Owned("PBXSourcesBuildPhase".to_string())),
            );
            p.insert(Cow::Owned("buildActionMask".to_string()), PlistValue::Integer(2147483647));
            p.insert(Cow::Owned("files".to_string()), PlistValue::Array(vec![]));
            p.insert(Cow::Owned("runOnlyForDeploymentPostprocessing".to_string()), PlistValue::Integer(0));
            self.create_object(p)
        };
        let frameworks_uuid = {
            let mut p = PlistMap::default();
            p.insert(
                Cow::Owned("isa".to_string()),
                PlistValue::String(Cow::Owned("PBXFrameworksBuildPhase".to_string())),
            );
            p.insert(Cow::Owned("buildActionMask".to_string()), PlistValue::Integer(2147483647));
            p.insert(Cow::Owned("files".to_string()), PlistValue::Array(vec![]));
            p.insert(Cow::Owned("runOnlyForDeploymentPostprocessing".to_string()), PlistValue::Integer(0));
            self.create_object(p)
        };
        let resources_uuid = {
            let mut p = PlistMap::default();
            p.insert(
                Cow::Owned("isa".to_string()),
                PlistValue::String(Cow::Owned("PBXResourcesBuildPhase".to_string())),
            );
            p.insert(Cow::Owned("buildActionMask".to_string()), PlistValue::Integer(2147483647));
            p.insert(Cow::Owned("files".to_string()), PlistValue::Array(vec![]));
            p.insert(Cow::Owned("runOnlyForDeploymentPostprocessing".to_string()), PlistValue::Integer(0));
            self.create_object(p)
        };

        // 6. Create PBXNativeTarget
        let mut target_props = PlistMap::default();
        target_props.insert(Cow::Owned("isa".to_string()), PlistValue::String(Cow::Owned("PBXNativeTarget".to_string())));
        target_props.insert(
            Cow::Owned("buildConfigurationList".to_string()),
            PlistValue::String(Cow::Owned(config_list_uuid)),
        );
        target_props.insert(
            Cow::Owned("buildPhases".to_string()),
            PlistValue::Array(vec![
                PlistValue::String(Cow::Owned(sources_uuid)),
                PlistValue::String(Cow::Owned(frameworks_uuid)),
                PlistValue::String(Cow::Owned(resources_uuid)),
            ]),
        );
        target_props.insert(Cow::Owned("buildRules".to_string()), PlistValue::Array(vec![]));
        target_props.insert(Cow::Owned("dependencies".to_string()), PlistValue::Array(vec![]));
        target_props.insert(Cow::Owned("name".to_string()), PlistValue::String(Cow::Owned(name.to_string())));
        target_props.insert(Cow::Owned("productName".to_string()), PlistValue::String(Cow::Owned(name.to_string())));
        target_props.insert(Cow::Owned("productReference".to_string()), PlistValue::String(Cow::Owned(product_ref_uuid)));
        target_props.insert(Cow::Owned("productType".to_string()), PlistValue::String(Cow::Owned(product_type.to_string())));
        let target_uuid = self.create_object(target_props);

        // 7. Add target to PBXProject.targets
        let root_uuid = self.root_object_uuid.clone();
        if let Some(root) = self.get_object_mut(&root_uuid) {
            if let Some(PlistValue::Array(ref mut targets)) = root.props.get_mut("targets") {
                targets.push(PlistValue::String(Cow::Owned(target_uuid.clone())));
            }
        }

        Some(target_uuid)
    }

    // ── Generic object property access ───────────────────────────────

    /// Get a string property from any object by UUID and key.
    pub fn get_object_property(&self, uuid: &str, key: &str) -> Option<String> {
        self.get_object(uuid)?.get_str(key).map(|s| s.to_string())
    }

    /// Set a string property on any object by UUID and key.
    pub fn set_object_property(&mut self, uuid: &str, key: &str, value: &str) -> bool {
        if let Some(obj) = self.get_object_mut(uuid) {
            obj.set_str(key, value);
            true
        } else {
            false
        }
    }

    /// Find all object UUIDs matching a given ISA type.
    pub fn find_objects_by_isa(&self, isa: &str) -> Vec<String> {
        self.objects
            .iter()
            .filter(|(_, obj)| obj.isa == isa)
            .map(|(uuid, _)| uuid.clone())
            .collect()
    }

    // ── Target name access ─────────────────────────────────────────

    /// Get the name of a target.
    pub fn get_target_name(&self, target_uuid: &str) -> Option<String> {
        self.get_object(target_uuid)?.get_str("name").map(|s| s.to_string())
    }

    /// Get the product type of a target (e.g. `com.apple.product-type.application`).
    pub fn get_target_product_type(&self, target_uuid: &str) -> Option<String> {
        self.get_object(target_uuid)?
            .get_str("productType")
            .map(|s| s.to_string())
    }

    /// Set the name and productName of a target.
    pub fn set_target_name(&mut self, target_uuid: &str, name: &str) -> bool {
        if let Some(target) = self.get_object_mut(target_uuid) {
            target.set_str("name", name);
            target.set_str("productName", name);
            true
        } else {
            false
        }
    }

    /// Rename a target and cascade the change through the project.
    ///
    /// Updates:
    /// - Target name and productName
    /// - Main group child with matching path (group path + name)
    /// - Product reference path (e.g. OldName.app → NewName.app)
    /// - PBXContainerItemProxy remoteInfo referencing the old name
    /// - XCConfigurationList display comment (via target name)
    ///
    /// Returns true if the target was found and renamed.
    pub fn rename_target(&mut self, target_uuid: &str, old_name: &str, new_name: &str) -> bool {
        // 1. Update target name + productName
        if !self.set_target_name(target_uuid, new_name) {
            return false;
        }

        // 2. Update product reference path (e.g. OldName.app → NewName.app)
        let product_ref_uuid = self
            .get_object(target_uuid)
            .and_then(|t| t.get_str("productReference"))
            .map(|s| s.to_string());

        if let Some(ref product_uuid) = product_ref_uuid {
            if let Some(product) = self.get_object_mut(product_uuid) {
                if let Some(old_path) = product.get_str("path").map(|s| s.to_string()) {
                    let new_path = old_path.replace(old_name, new_name);
                    product.set_str("path", &new_path);
                }
            }
        }

        // 3. Update main group children with matching path
        let main_group = self.main_group_uuid();
        if let Some(mg_uuid) = main_group {
            let children = self.get_group_children(&mg_uuid);
            for child_uuid in children {
                let matches = self
                    .get_object(&child_uuid)
                    .and_then(|c| c.get_str("path"))
                    .map(|p| p == old_name)
                    .unwrap_or(false);

                if matches {
                    if let Some(child) = self.get_object_mut(&child_uuid) {
                        child.set_str("path", new_name);
                        if child.get_str("name").is_some() {
                            child.set_str("name", new_name);
                        }
                    }
                }
            }
        }

        // 4. Update PBXContainerItemProxy remoteInfo
        let proxy_uuids = self.find_objects_by_isa("PBXContainerItemProxy");
        for proxy_uuid in proxy_uuids {
            let matches = self
                .get_object(&proxy_uuid)
                .and_then(|p| p.get_str("remoteInfo"))
                .map(|info| info == old_name)
                .unwrap_or(false);

            if matches {
                if let Some(proxy) = self.get_object_mut(&proxy_uuid) {
                    proxy.set_str("remoteInfo", new_name);
                }
            }
        }

        true
    }

    // ── Extension embedding ────────────────────────────────────────

    /// Returns UUIDs of targets whose products are embedded in the given target
    /// via PBXCopyFilesBuildPhase (e.g. "Embed Foundation Extensions", "Embed Frameworks").
    ///
    /// Walks: target.buildPhases -> PBXCopyFilesBuildPhase -> files -> PBXBuildFile.fileRef
    ///        -> matches against all targets' productReference to resolve target UUIDs.
    pub fn get_embedded_targets(&self, target_uuid: &str) -> Vec<String> {
        let target = match self.get_object(target_uuid) {
            Some(t) => t,
            None => return vec![],
        };
        let phases = match target.get_array("buildPhases") {
            Some(p) => p,
            None => return vec![],
        };

        let mut embedded_file_refs: Vec<&str> = Vec::new();
        for phase_val in phases {
            let phase_uuid = match phase_val.as_str() {
                Some(u) => u,
                None => continue,
            };
            let phase = match self.get_object(phase_uuid) {
                Some(p) if p.isa == "PBXCopyFilesBuildPhase" => p,
                _ => continue,
            };
            let files = match phase.get_array("files") {
                Some(f) => f,
                None => continue,
            };
            for file_val in files {
                if let Some(build_file_uuid) = file_val.as_str() {
                    if let Some(build_file) = self.get_object(build_file_uuid) {
                        if let Some(file_ref) = build_file.get_str("fileRef") {
                            embedded_file_refs.push(file_ref);
                        }
                    }
                }
            }
        }

        if embedded_file_refs.is_empty() {
            return vec![];
        }

        let mut result = Vec::new();
        for t in self.native_targets() {
            if let Some(product_ref) = t.get_str("productReference") {
                if embedded_file_refs.contains(&product_ref) {
                    result.push(t.uuid.clone());
                }
            }
        }
        result
    }

    /// Embed an extension target into a host app target.
    ///
    /// Creates a PBXCopyFilesBuildPhase with the correct dstSubfolderSpec
    /// based on the extension's product type, creates a PBXBuildFile
    /// referencing the extension's product, and wires everything to the
    /// host target.
    ///
    /// Returns the UUID of the PBXCopyFilesBuildPhase.
    pub fn embed_extension(&mut self, host_target_uuid: &str, extension_target_uuid: &str) -> Option<String> {
        // Get extension target's product type and product reference
        let ext_target = self.get_object(extension_target_uuid)?;
        let product_type = ext_target.get_str("productType")?.to_string();
        let product_ref_uuid = ext_target.get_str("productReference")?.to_string();

        // Determine dstSubfolderSpec and phase name from product type
        let (dst_subfolder_spec, dst_path, phase_name) = match product_type.as_str() {
            "com.apple.product-type.application.on-demand-install-capable" => {
                (16, "$(CONTENTS_FOLDER_PATH)/AppClips", "Embed App Clips")
            }
            "com.apple.product-type.application" => (16, "$(CONTENTS_FOLDER_PATH)/Watch", "Embed Watch Content"),
            "com.apple.product-type.extensionkit-extension" => {
                (16, "$(EXTENSIONS_FOLDER_PATH)", "Embed ExtensionKit Extensions")
            }
            _ => {
                // Default: PlugIns folder for app extensions
                (13, "", "Embed Foundation Extensions")
            }
        };

        // Create PBXBuildFile referencing the extension product
        let mut build_file_props = PlistMap::default();
        build_file_props.insert(Cow::Owned("isa".to_string()), PlistValue::String(Cow::Owned("PBXBuildFile".to_string())));
        build_file_props.insert(Cow::Owned("fileRef".to_string()), PlistValue::String(Cow::Owned(product_ref_uuid)));
        let settings: PlistObject<'static> = vec![(
            Cow::Owned("ATTRIBUTES".to_string()),
            PlistValue::Array(vec![PlistValue::String(Cow::Owned("RemoveHeadersOnCopy".to_string()))]),
        )];
        build_file_props.insert(Cow::Owned("settings".to_string()), PlistValue::Object(settings));
        let build_file_uuid = self.create_object(build_file_props);

        // Create PBXCopyFilesBuildPhase
        let mut phase_props = PlistMap::default();
        phase_props.insert(
            Cow::Owned("isa".to_string()),
            PlistValue::String(Cow::Owned("PBXCopyFilesBuildPhase".to_string())),
        );
        phase_props.insert(Cow::Owned("buildActionMask".to_string()), PlistValue::Integer(2147483647));
        phase_props.insert(Cow::Owned("dstPath".to_string()), PlistValue::String(Cow::Owned(dst_path.to_string())));
        phase_props.insert(Cow::Owned("dstSubfolderSpec".to_string()), PlistValue::Integer(dst_subfolder_spec));
        phase_props.insert(
            Cow::Owned("files".to_string()),
            PlistValue::Array(vec![PlistValue::String(Cow::Owned(build_file_uuid))]),
        );
        phase_props.insert(Cow::Owned("name".to_string()), PlistValue::String(Cow::Owned(phase_name.to_string())));
        phase_props.insert(Cow::Owned("runOnlyForDeploymentPostprocessing".to_string()), PlistValue::Integer(0));
        let phase_uuid = self.create_object(phase_props);

        // Add phase to host target's buildPhases
        if let Some(host) = self.get_object_mut(host_target_uuid) {
            if let Some(PlistValue::Array(ref mut phases)) = host.props.get_mut("buildPhases") {
                phases.push(PlistValue::String(Cow::Owned(phase_uuid.clone())));
            }
        }

        Some(phase_uuid)
    }

    // ── Xcode 16+ file system sync groups ──────────────────────────

    /// Add a PBXFileSystemSynchronizedRootGroup to a target.
    ///
    /// Creates the sync group, adds it to the target's
    /// fileSystemSynchronizedGroups array, and adds it as a child
    /// of the main group.
    ///
    /// Returns the UUID of the sync group.
    pub fn add_file_system_sync_group(&mut self, target_uuid: &str, path: &str) -> Option<String> {
        let mut props = PlistMap::default();
        props.insert(
            Cow::Owned("isa".to_string()),
            PlistValue::String(Cow::Owned("PBXFileSystemSynchronizedRootGroup".to_string())),
        );
        props.insert(Cow::Owned("path".to_string()), PlistValue::String(Cow::Owned(path.to_string())));
        props.insert(Cow::Owned("sourceTree".to_string()), PlistValue::String(Cow::Owned("<group>".to_string())));
        let sync_group_uuid = self.create_object(props);

        // Add to target's fileSystemSynchronizedGroups
        if let Some(target) = self.get_object_mut(target_uuid) {
            match target.props.get_mut("fileSystemSynchronizedGroups") {
                Some(PlistValue::Array(ref mut groups)) => {
                    groups.push(PlistValue::String(Cow::Owned(sync_group_uuid.clone())));
                }
                _ => {
                    target.props.insert(
                        Cow::Owned("fileSystemSynchronizedGroups".to_string()),
                        PlistValue::Array(vec![PlistValue::String(Cow::Owned(sync_group_uuid.clone()))]),
                    );
                }
            }
        }

        // Add to main group's children
        let main_group = self.main_group_uuid();
        if let Some(mg_uuid) = main_group {
            if let Some(group) = self.get_object_mut(&mg_uuid) {
                if let Some(PlistValue::Array(ref mut children)) = group.props.get_mut("children") {
                    children.push(PlistValue::String(Cow::Owned(sync_group_uuid.clone())));
                }
            }
        }

        Some(sync_group_uuid)
    }

    /// Get the `path` of each `PBXFileSystemSynchronizedRootGroup` linked to a
    /// target's `fileSystemSynchronizedGroups` array.
    /// Returns `[]` if the target has no sync groups (pre-Xcode 16 projects).
    pub fn get_target_sync_group_paths(&self, target_uuid: &str) -> Vec<String> {
        let target = match self.get_object(target_uuid) {
            Some(t) => t,
            None => return vec![],
        };
        let group_uuids = match target.props.get("fileSystemSynchronizedGroups") {
            Some(PlistValue::Array(arr)) => arr,
            _ => return vec![],
        };
        group_uuids
            .iter()
            .filter_map(|v| v.as_str())
            .filter_map(|uuid| self.get_object(uuid))
            .filter_map(|obj| obj.get_str("path").map(|s| s.to_string()))
            .collect()
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
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();

        for config_uuid in config_uuids {
            if let Some(config) = self.get_object_mut(&config_uuid) {
                if let Some(PlistValue::Object(ref mut settings)) = config.props.get_mut("buildSettings") {
                    settings.retain(|(k, _)| k.as_ref() != key);
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

    const FIXTURES_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

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
        assert!(!orphans.is_empty(), "Malformed project should have orphaned references");

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
    fn test_get_target_product_type() {
        let path = Path::new(FIXTURES_DIR).join("project.pbxproj");
        let content = fs::read_to_string(&path).unwrap();
        let project = XcodeProject::from_plist(&content).unwrap();

        let target = project
            .find_target_by_product_type("com.apple.product-type.application")
            .expect("should find app target");
        assert_eq!(
            project.get_target_product_type(&target.uuid),
            Some("com.apple.product-type.application".to_string())
        );

        assert_eq!(project.get_target_product_type("nonexistent-uuid"), None);
    }

    #[test]
    fn test_get_target_sync_group_paths() {
        let path = Path::new(FIXTURES_DIR).join("project.pbxproj");
        let content = fs::read_to_string(&path).unwrap();
        let mut project = XcodeProject::from_plist(&content).unwrap();

        let target_uuid = project.native_targets()[0].uuid.clone();

        // Before adding any sync groups, should return empty
        assert!(project.get_target_sync_group_paths(&target_uuid).is_empty());

        // Add sync groups and verify they're returned
        project.add_file_system_sync_group(&target_uuid, "MyApp");
        project.add_file_system_sync_group(&target_uuid, "MyAppTests");

        let paths = project.get_target_sync_group_paths(&target_uuid);
        assert_eq!(paths, vec!["MyApp".to_string(), "MyAppTests".to_string()]);

        // Nonexistent target returns empty
        assert!(project.get_target_sync_group_paths("nonexistent-uuid").is_empty());
    }

    #[test]
    fn test_get_embedded_targets() {
        let path = Path::new(FIXTURES_DIR).join("project.pbxproj");
        let content = fs::read_to_string(&path).unwrap();
        let mut project = XcodeProject::from_plist(&content).unwrap();

        let host_uuid = project.native_targets()[0].uuid.clone();

        // No embedded targets initially
        assert!(project.get_embedded_targets(&host_uuid).is_empty());

        // Create an extension target and embed it
        let ext_uuid = project
            .create_native_target(
                "WidgetExtension",
                "com.apple.product-type.app-extension",
                "com.test.widget",
            )
            .unwrap();
        project.embed_extension(&host_uuid, &ext_uuid);

        let embedded = project.get_embedded_targets(&host_uuid);
        assert_eq!(embedded, vec![ext_uuid.clone()]);

        // Embed a second extension
        let ext2_uuid = project
            .create_native_target(
                "IntentExtension",
                "com.apple.product-type.app-extension",
                "com.test.intent",
            )
            .unwrap();
        project.embed_extension(&host_uuid, &ext2_uuid);

        let embedded = project.get_embedded_targets(&host_uuid);
        assert_eq!(embedded.len(), 2);
        assert!(embedded.contains(&ext_uuid));
        assert!(embedded.contains(&ext2_uuid));

        // Nonexistent target returns empty
        assert!(project.get_embedded_targets("nonexistent-uuid").is_empty());
    }

    #[test]
    fn test_remove_file() {
        let path = Path::new(FIXTURES_DIR).join("project.pbxproj");
        let content = fs::read_to_string(&path).unwrap();
        let mut project = XcodeProject::from_plist(&content).unwrap();

        let initial_count = project.objects().count();

        let file_refs = project.find_objects_by_isa("PBXFileReference");
        assert!(!file_refs.is_empty());
        let file_uuid = file_refs[0].clone();

        assert!(project.remove_file(&file_uuid));
        assert!(project.get_object(&file_uuid).is_none());
        assert!(project.objects().count() < initial_count);

        // Removing again should return false
        assert!(!project.remove_file(&file_uuid));

        // Nonexistent UUID
        assert!(!project.remove_file("000000000000000000000000"));
    }

    #[test]
    fn test_remove_group() {
        let path = Path::new(FIXTURES_DIR).join("project.pbxproj");
        let content = fs::read_to_string(&path).unwrap();
        let mut project = XcodeProject::from_plist(&content).unwrap();

        // Create a group, then remove it
        let main_group = project.main_group_uuid().unwrap();
        let group_uuid = project.add_group(&main_group, "TempGroup").unwrap();

        assert!(project.get_object(&group_uuid).is_some());
        let children_before = project.get_group_children(&main_group);
        assert!(children_before.contains(&group_uuid));

        assert!(project.remove_group(&group_uuid));
        assert!(project.get_object(&group_uuid).is_none());

        let children_after = project.get_group_children(&main_group);
        assert!(!children_after.contains(&group_uuid));

        // Removing again returns false
        assert!(!project.remove_group(&group_uuid));
    }

    #[test]
    fn test_add_remote_swift_package() {
        let path = Path::new(FIXTURES_DIR).join("project.pbxproj");
        let content = fs::read_to_string(&path).unwrap();
        let mut project = XcodeProject::from_plist(&content).unwrap();

        assert!(project.list_swift_packages().is_empty());

        let uuid = project
            .add_remote_swift_package("https://github.com/apple/swift-collections", "1.0.0")
            .unwrap();

        let obj = project.get_object(&uuid).unwrap();
        assert_eq!(obj.isa, "XCRemoteSwiftPackageReference");
        assert_eq!(obj.get_str("repositoryURL"), Some("https://github.com/apple/swift-collections"));

        let packages = project.list_swift_packages();
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].0, uuid);
        assert!(packages[0].2.contains("swift-collections"));
    }

    #[test]
    fn test_add_local_swift_package() {
        let path = Path::new(FIXTURES_DIR).join("project.pbxproj");
        let content = fs::read_to_string(&path).unwrap();
        let mut project = XcodeProject::from_plist(&content).unwrap();

        let uuid = project.add_local_swift_package("../Packages/MyLib").unwrap();

        let obj = project.get_object(&uuid).unwrap();
        assert_eq!(obj.isa, "XCLocalSwiftPackageReference");
        assert_eq!(obj.get_str("relativePath"), Some("../Packages/MyLib"));

        let packages = project.list_swift_packages();
        assert_eq!(packages.len(), 1);
    }

    #[test]
    fn test_add_and_remove_swift_package_product() {
        let path = Path::new(FIXTURES_DIR).join("project.pbxproj");
        let content = fs::read_to_string(&path).unwrap();
        let mut project = XcodeProject::from_plist(&content).unwrap();

        let target_uuid = project.native_targets()[0].uuid.clone();
        let package_uuid = project
            .add_remote_swift_package("https://github.com/apple/swift-collections", "1.0.0")
            .unwrap();

        let dep_uuid = project
            .add_swift_package_product(&target_uuid, "Collections", &package_uuid)
            .unwrap();

        let dep_obj = project.get_object(&dep_uuid).unwrap();
        assert_eq!(dep_obj.isa, "XCSwiftPackageProductDependency");
        assert_eq!(dep_obj.get_str("productName"), Some("Collections"));

        // Verify it was added to target's packageProductDependencies
        let target = project.get_object(&target_uuid).unwrap();
        let deps = target.get_array("packageProductDependencies").unwrap();
        assert!(deps.iter().any(|v| v.as_str() == Some(&dep_uuid)));

        // Remove it
        assert!(project.remove_swift_package_product(&target_uuid, "Collections"));
        assert!(project.get_object(&dep_uuid).is_none());

        // Target should no longer have it
        let target = project.get_object(&target_uuid).unwrap();
        let deps = target.get_array("packageProductDependencies").unwrap();
        assert!(!deps.iter().any(|v| v.as_str() == Some(&dep_uuid)));

        // Removing again returns false
        assert!(!project.remove_swift_package_product(&target_uuid, "Collections"));
    }

    #[test]
    fn test_list_swift_packages_on_spm_fixture() {
        let path = Path::new(FIXTURES_DIR).join("006-spm.pbxproj");
        let content = fs::read_to_string(&path).unwrap();
        let project = XcodeProject::from_plist(&content).unwrap();

        let packages = project.list_swift_packages();
        assert!(!packages.is_empty());
        assert!(packages.iter().any(|(_, _, loc)| loc.contains("supabase")));
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
