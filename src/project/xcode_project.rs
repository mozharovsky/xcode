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
        let contents = std::fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;
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
        let root = plist.as_object().ok_or("Root must be an object")?;

        let archive_version = root.get("archiveVersion").and_then(|v| v.as_integer()).unwrap_or(1);

        let object_version = root.get("objectVersion").and_then(|v| v.as_integer()).unwrap_or(46);

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
    pub fn to_plist(&self) -> PlistValue {
        let mut root = IndexMap::new();
        root.insert("archiveVersion".to_string(), PlistValue::Integer(self.archive_version));
        root.insert("classes".to_string(), PlistValue::Object(self.classes.clone()));
        root.insert("objectVersion".to_string(), PlistValue::Integer(self.object_version));

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
    pub fn set_build_setting(&mut self, target_uuid: &str, key: &str, value: PlistValue) -> bool {
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
                    settings.insert(key.to_string(), value.clone());
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

        let mut props = IndexMap::new();
        props.insert("isa".to_string(), PlistValue::String("PBXFileReference".to_string()));
        props.insert("fileEncoding".to_string(), PlistValue::Integer(4));
        props.insert(
            "lastKnownFileType".to_string(),
            PlistValue::String(file_type.to_string()),
        );
        if name != path {
            props.insert("name".to_string(), PlistValue::String(name.to_string()));
        }
        props.insert("path".to_string(), PlistValue::String(path.to_string()));
        props.insert("sourceTree".to_string(), PlistValue::String(source_tree.to_string()));

        let file_uuid = self.create_object(props);

        // Add to group's children
        if let Some(group) = self.get_object_mut(group_uuid) {
            if let Some(PlistValue::Array(ref mut children)) = group.props.get_mut("children") {
                children.push(PlistValue::String(file_uuid.clone()));
            }
        }

        Some(file_uuid)
    }

    /// Create a group and add it as a child of a parent group.
    /// Returns the UUID of the new PBXGroup.
    pub fn add_group(&mut self, parent_uuid: &str, name: &str) -> Option<String> {
        let mut props = IndexMap::new();
        props.insert("isa".to_string(), PlistValue::String("PBXGroup".to_string()));
        props.insert("children".to_string(), PlistValue::Array(vec![]));
        props.insert("name".to_string(), PlistValue::String(name.to_string()));
        props.insert("sourceTree".to_string(), PlistValue::String("<group>".to_string()));

        let group_uuid = self.create_object(props);

        if let Some(parent) = self.get_object_mut(parent_uuid) {
            if let Some(PlistValue::Array(ref mut children)) = parent.props.get_mut("children") {
                children.push(PlistValue::String(group_uuid.clone()));
            }
        }

        Some(group_uuid)
    }

    // ── Build phase operations ─────────────────────────────────────

    /// Add a build file to a build phase (e.g. adding a source file to the Sources phase).
    /// Returns the UUID of the new PBXBuildFile.
    pub fn add_build_file(&mut self, phase_uuid: &str, file_ref_uuid: &str) -> Option<String> {
        let mut props = IndexMap::new();
        props.insert("isa".to_string(), PlistValue::String("PBXBuildFile".to_string()));
        props.insert("fileRef".to_string(), PlistValue::String(file_ref_uuid.to_string()));

        let build_file_uuid = self.create_object(props);

        if let Some(phase) = self.get_object_mut(phase_uuid) {
            if let Some(PlistValue::Array(ref mut files)) = phase.props.get_mut("files") {
                files.push(PlistValue::String(build_file_uuid.clone()));
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
        let mut props = IndexMap::new();
        props.insert("isa".to_string(), PlistValue::String(phase_isa.to_string()));
        props.insert("buildActionMask".to_string(), PlistValue::Integer(2147483647));
        props.insert("files".to_string(), PlistValue::Array(vec![]));
        props.insert("runOnlyForDeploymentPostprocessing".to_string(), PlistValue::Integer(0));

        let phase_uuid = self.create_object(props);

        // Add to target's buildPhases
        if let Some(target) = self.get_object_mut(target_uuid) {
            if let Some(PlistValue::Array(ref mut phases)) = target.props.get_mut("buildPhases") {
                phases.push(PlistValue::String(phase_uuid.clone()));
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
        let mut file_props = IndexMap::new();
        file_props.insert("isa".to_string(), PlistValue::String("PBXFileReference".to_string()));
        file_props.insert(
            "lastKnownFileType".to_string(),
            PlistValue::String("wrapper.framework".to_string()),
        );
        file_props.insert("name".to_string(), PlistValue::String(name.clone()));
        file_props.insert("path".to_string(), PlistValue::String(path));
        file_props.insert("sourceTree".to_string(), PlistValue::String("SDKROOT".to_string()));

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
        let mut proxy_props = IndexMap::new();
        proxy_props.insert(
            "isa".to_string(),
            PlistValue::String("PBXContainerItemProxy".to_string()),
        );
        proxy_props.insert(
            "containerPortal".to_string(),
            PlistValue::String(self.root_object_uuid.clone()),
        );
        proxy_props.insert("proxyType".to_string(), PlistValue::Integer(1));
        proxy_props.insert(
            "remoteGlobalIDString".to_string(),
            PlistValue::String(depends_on_uuid.to_string()),
        );

        // Get name of the dependency target
        let remote_name = self
            .get_object(depends_on_uuid)
            .and_then(|t| t.get_str("name"))
            .unwrap_or("Unknown")
            .to_string();
        proxy_props.insert("remoteInfo".to_string(), PlistValue::String(remote_name));

        let proxy_uuid = self.create_object(proxy_props);

        // Create PBXTargetDependency
        let mut dep_props = IndexMap::new();
        dep_props.insert("isa".to_string(), PlistValue::String("PBXTargetDependency".to_string()));
        dep_props.insert("target".to_string(), PlistValue::String(depends_on_uuid.to_string()));
        dep_props.insert("targetProxy".to_string(), PlistValue::String(proxy_uuid));

        let dep_uuid = self.create_object(dep_props);

        // Add to target's dependencies
        if let Some(target) = self.get_object_mut(target_uuid) {
            if let Some(PlistValue::Array(ref mut deps)) = target.props.get_mut("dependencies") {
                deps.push(PlistValue::String(dep_uuid.clone()));
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
        let mut product_props = IndexMap::new();
        product_props.insert("isa".to_string(), PlistValue::String("PBXFileReference".to_string()));
        product_props.insert(
            "explicitFileType".to_string(),
            PlistValue::String(
                crate::types::constants::FILE_TYPES_BY_EXTENSION
                    .get(product_ext)
                    .copied()
                    .unwrap_or("wrapper.application")
                    .to_string(),
            ),
        );
        product_props.insert("includeInIndex".to_string(), PlistValue::Integer(0));
        product_props.insert("path".to_string(), PlistValue::String(product_name));
        product_props.insert(
            "sourceTree".to_string(),
            PlistValue::String("BUILT_PRODUCTS_DIR".to_string()),
        );
        let product_ref_uuid = self.create_object(product_props);

        // Add product to Products group
        if let Some(products_uuid) = self.product_ref_group_uuid() {
            if let Some(products) = self.get_object_mut(&products_uuid) {
                if let Some(PlistValue::Array(ref mut children)) = products.props.get_mut("children") {
                    children.push(PlistValue::String(product_ref_uuid.clone()));
                }
            }
        }

        // 2. Create Debug build configuration
        let mut debug_settings = IndexMap::new();
        debug_settings.insert(
            "PRODUCT_BUNDLE_IDENTIFIER".to_string(),
            PlistValue::String(bundle_id.to_string()),
        );
        debug_settings.insert("PRODUCT_NAME".to_string(), PlistValue::String(name.to_string()));
        debug_settings.insert("SWIFT_VERSION".to_string(), PlistValue::String("5.0".to_string()));

        let mut debug_props = IndexMap::new();
        debug_props.insert(
            "isa".to_string(),
            PlistValue::String("XCBuildConfiguration".to_string()),
        );
        debug_props.insert("buildSettings".to_string(), PlistValue::Object(debug_settings));
        debug_props.insert("name".to_string(), PlistValue::String("Debug".to_string()));
        let debug_uuid = self.create_object(debug_props);

        // 3. Create Release build configuration
        let mut release_settings = IndexMap::new();
        release_settings.insert(
            "PRODUCT_BUNDLE_IDENTIFIER".to_string(),
            PlistValue::String(bundle_id.to_string()),
        );
        release_settings.insert("PRODUCT_NAME".to_string(), PlistValue::String(name.to_string()));
        release_settings.insert("SWIFT_VERSION".to_string(), PlistValue::String("5.0".to_string()));

        let mut release_props = IndexMap::new();
        release_props.insert(
            "isa".to_string(),
            PlistValue::String("XCBuildConfiguration".to_string()),
        );
        release_props.insert("buildSettings".to_string(), PlistValue::Object(release_settings));
        release_props.insert("name".to_string(), PlistValue::String("Release".to_string()));
        let release_uuid = self.create_object(release_props);

        // 4. Create XCConfigurationList
        let mut config_list_props = IndexMap::new();
        config_list_props.insert("isa".to_string(), PlistValue::String("XCConfigurationList".to_string()));
        config_list_props.insert(
            "buildConfigurations".to_string(),
            PlistValue::Array(vec![PlistValue::String(debug_uuid), PlistValue::String(release_uuid)]),
        );
        config_list_props.insert("defaultConfigurationIsVisible".to_string(), PlistValue::Integer(0));
        config_list_props.insert(
            "defaultConfigurationName".to_string(),
            PlistValue::String("Release".to_string()),
        );
        let config_list_uuid = self.create_object(config_list_props);

        // 5. Create standard build phases
        let sources_uuid = {
            let mut p = IndexMap::new();
            p.insert(
                "isa".to_string(),
                PlistValue::String("PBXSourcesBuildPhase".to_string()),
            );
            p.insert("buildActionMask".to_string(), PlistValue::Integer(2147483647));
            p.insert("files".to_string(), PlistValue::Array(vec![]));
            p.insert("runOnlyForDeploymentPostprocessing".to_string(), PlistValue::Integer(0));
            self.create_object(p)
        };
        let frameworks_uuid = {
            let mut p = IndexMap::new();
            p.insert(
                "isa".to_string(),
                PlistValue::String("PBXFrameworksBuildPhase".to_string()),
            );
            p.insert("buildActionMask".to_string(), PlistValue::Integer(2147483647));
            p.insert("files".to_string(), PlistValue::Array(vec![]));
            p.insert("runOnlyForDeploymentPostprocessing".to_string(), PlistValue::Integer(0));
            self.create_object(p)
        };
        let resources_uuid = {
            let mut p = IndexMap::new();
            p.insert(
                "isa".to_string(),
                PlistValue::String("PBXResourcesBuildPhase".to_string()),
            );
            p.insert("buildActionMask".to_string(), PlistValue::Integer(2147483647));
            p.insert("files".to_string(), PlistValue::Array(vec![]));
            p.insert("runOnlyForDeploymentPostprocessing".to_string(), PlistValue::Integer(0));
            self.create_object(p)
        };

        // 6. Create PBXNativeTarget
        let mut target_props = IndexMap::new();
        target_props.insert("isa".to_string(), PlistValue::String("PBXNativeTarget".to_string()));
        target_props.insert(
            "buildConfigurationList".to_string(),
            PlistValue::String(config_list_uuid),
        );
        target_props.insert(
            "buildPhases".to_string(),
            PlistValue::Array(vec![
                PlistValue::String(sources_uuid),
                PlistValue::String(frameworks_uuid),
                PlistValue::String(resources_uuid),
            ]),
        );
        target_props.insert("buildRules".to_string(), PlistValue::Array(vec![]));
        target_props.insert("dependencies".to_string(), PlistValue::Array(vec![]));
        target_props.insert("name".to_string(), PlistValue::String(name.to_string()));
        target_props.insert("productName".to_string(), PlistValue::String(name.to_string()));
        target_props.insert("productReference".to_string(), PlistValue::String(product_ref_uuid));
        target_props.insert("productType".to_string(), PlistValue::String(product_type.to_string()));
        let target_uuid = self.create_object(target_props);

        // 7. Add target to PBXProject.targets
        let root_uuid = self.root_object_uuid.clone();
        if let Some(root) = self.get_object_mut(&root_uuid) {
            if let Some(PlistValue::Array(ref mut targets)) = root.props.get_mut("targets") {
                targets.push(PlistValue::String(target_uuid.clone()));
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
        let mut build_file_props = IndexMap::new();
        build_file_props.insert("isa".to_string(), PlistValue::String("PBXBuildFile".to_string()));
        build_file_props.insert("fileRef".to_string(), PlistValue::String(product_ref_uuid));
        let mut settings = IndexMap::new();
        settings.insert(
            "ATTRIBUTES".to_string(),
            PlistValue::Array(vec![PlistValue::String("RemoveHeadersOnCopy".to_string())]),
        );
        build_file_props.insert("settings".to_string(), PlistValue::Object(settings));
        let build_file_uuid = self.create_object(build_file_props);

        // Create PBXCopyFilesBuildPhase
        let mut phase_props = IndexMap::new();
        phase_props.insert(
            "isa".to_string(),
            PlistValue::String("PBXCopyFilesBuildPhase".to_string()),
        );
        phase_props.insert("buildActionMask".to_string(), PlistValue::Integer(2147483647));
        phase_props.insert("dstPath".to_string(), PlistValue::String(dst_path.to_string()));
        phase_props.insert("dstSubfolderSpec".to_string(), PlistValue::Integer(dst_subfolder_spec));
        phase_props.insert(
            "files".to_string(),
            PlistValue::Array(vec![PlistValue::String(build_file_uuid)]),
        );
        phase_props.insert("name".to_string(), PlistValue::String(phase_name.to_string()));
        phase_props.insert("runOnlyForDeploymentPostprocessing".to_string(), PlistValue::Integer(0));
        let phase_uuid = self.create_object(phase_props);

        // Add phase to host target's buildPhases
        if let Some(host) = self.get_object_mut(host_target_uuid) {
            if let Some(PlistValue::Array(ref mut phases)) = host.props.get_mut("buildPhases") {
                phases.push(PlistValue::String(phase_uuid.clone()));
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
        let mut props = IndexMap::new();
        props.insert(
            "isa".to_string(),
            PlistValue::String("PBXFileSystemSynchronizedRootGroup".to_string()),
        );
        props.insert("path".to_string(), PlistValue::String(path.to_string()));
        props.insert("sourceTree".to_string(), PlistValue::String("<group>".to_string()));
        let sync_group_uuid = self.create_object(props);

        // Add to target's fileSystemSynchronizedGroups
        if let Some(target) = self.get_object_mut(target_uuid) {
            match target.props.get_mut("fileSystemSynchronizedGroups") {
                Some(PlistValue::Array(ref mut groups)) => {
                    groups.push(PlistValue::String(sync_group_uuid.clone()));
                }
                _ => {
                    target.props.insert(
                        "fileSystemSynchronizedGroups".to_string(),
                        PlistValue::Array(vec![PlistValue::String(sync_group_uuid.clone())]),
                    );
                }
            }
        }

        // Add to main group's children
        let main_group = self.main_group_uuid();
        if let Some(mg_uuid) = main_group {
            if let Some(group) = self.get_object_mut(&mg_uuid) {
                if let Some(PlistValue::Array(ref mut children)) = group.props.get_mut("children") {
                    children.push(PlistValue::String(sync_group_uuid.clone()));
                }
            }
        }

        Some(sync_group_uuid)
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
