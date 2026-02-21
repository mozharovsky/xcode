#[cfg(feature = "napi")]
#[macro_use]
extern crate napi_derive;

pub mod objects;
pub mod parser;
pub mod project;
pub mod types;
pub mod writer;

// ── WASM bindings ──────────────────────────────────────────────────

#[cfg(feature = "wasm")]
mod wasm_bindings {
    use serde::Serialize;
    use wasm_bindgen::prelude::*;

    fn serializer() -> serde_wasm_bindgen::Serializer {
        serde_wasm_bindgen::Serializer::new().serialize_maps_as_objects(true)
    }

    /// Parse a .pbxproj string into a JS object.
    #[wasm_bindgen]
    pub fn parse(text: &str) -> Result<JsValue, JsError> {
        let plist = crate::parser::parse(text).map_err(|e| JsError::new(&e))?;
        plist.serialize(&serializer()).map_err(|e| JsError::new(&e.to_string()))
    }

    /// Serialize a JS object back to .pbxproj format.
    #[wasm_bindgen]
    pub fn build(project: JsValue) -> Result<String, JsError> {
        let plist: crate::types::PlistValue =
            serde_wasm_bindgen::from_value(project).map_err(|e| JsError::new(&e.to_string()))?;
        Ok(crate::writer::serializer::build(&plist))
    }

    /// Parse and immediately re-serialize a .pbxproj string.
    #[wasm_bindgen(js_name = "parseAndBuild")]
    pub fn parse_and_build(text: &str) -> Result<String, JsError> {
        let plist = crate::parser::parse(text).map_err(|e| JsError::new(&e))?;
        Ok(crate::writer::serializer::build(&plist))
    }

    /// High-level project manipulation — stays in WASM memory.
    #[wasm_bindgen]
    pub struct XcodeProject {
        inner: crate::project::XcodeProject,
    }

    #[wasm_bindgen]
    impl XcodeProject {
        /// Parse a .pbxproj string into an XcodeProject.
        #[wasm_bindgen(constructor)]
        pub fn new(content: &str) -> Result<XcodeProject, JsError> {
            let inner = crate::project::XcodeProject::from_plist(content).map_err(|e| JsError::new(&e))?;
            Ok(XcodeProject { inner })
        }

        /// Serialize the project back to .pbxproj format.
        #[wasm_bindgen(js_name = "toBuild")]
        pub fn to_build(&self) -> String {
            self.inner.to_pbxproj()
        }

        /// Convert the project to a JS object.
        #[wasm_bindgen(js_name = "toJSON")]
        pub fn to_json(&self) -> Result<JsValue, JsError> {
            let plist = self.inner.to_plist();
            plist.serialize(&serializer()).map_err(|e| JsError::new(&e.to_string()))
        }

        // ── Properties ───────────────────────────────────────────

        #[wasm_bindgen(getter, js_name = "archiveVersion")]
        pub fn archive_version(&self) -> i64 {
            self.inner.archive_version
        }

        #[wasm_bindgen(getter, js_name = "objectVersion")]
        pub fn object_version(&self) -> i64 {
            self.inner.object_version
        }

        #[wasm_bindgen(getter, js_name = "mainGroupUuid")]
        pub fn main_group_uuid(&self) -> Option<String> {
            self.inner.main_group_uuid()
        }

        // ── Targets ──────────────────────────────────────────────

        #[wasm_bindgen(js_name = "getNativeTargets")]
        pub fn get_native_targets(&self) -> Vec<String> {
            self.inner.native_targets().iter().map(|t| t.uuid.clone()).collect()
        }

        #[wasm_bindgen(js_name = "findMainAppTarget")]
        pub fn find_main_app_target(&self, platform: Option<String>) -> Option<String> {
            let p = platform.as_deref().unwrap_or("ios");
            self.inner.find_main_app_target(p).map(|t| t.uuid.clone())
        }

        #[wasm_bindgen(js_name = "getTargetName")]
        pub fn get_target_name(&self, target_uuid: &str) -> Option<String> {
            self.inner.get_target_name(target_uuid)
        }

        #[wasm_bindgen(js_name = "getTargetProductType")]
        pub fn get_target_product_type(&self, target_uuid: &str) -> Option<String> {
            self.inner.get_target_product_type(target_uuid)
        }

        #[wasm_bindgen(js_name = "setTargetName")]
        pub fn set_target_name(&mut self, target_uuid: &str, name: &str) -> bool {
            self.inner.set_target_name(target_uuid, name)
        }

        /// Rename a target and cascade through the project (group paths, product refs, proxies).
        #[wasm_bindgen(js_name = "renameTarget")]
        pub fn rename_target(&mut self, target_uuid: &str, old_name: &str, new_name: &str) -> bool {
            self.inner.rename_target(target_uuid, old_name, new_name)
        }

        #[wasm_bindgen(js_name = "createNativeTarget")]
        pub fn create_native_target(&mut self, name: &str, product_type: &str, bundle_id: &str) -> Option<String> {
            self.inner.create_native_target(name, product_type, bundle_id)
        }

        // ── Build settings ───────────────────────────────────────

        #[wasm_bindgen(js_name = "getBuildSetting")]
        pub fn get_build_setting(&self, target_uuid: &str, key: &str) -> Option<String> {
            self.inner
                .get_build_setting(target_uuid, key)
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        }

        #[wasm_bindgen(js_name = "setBuildSetting")]
        pub fn set_build_setting(&mut self, target_uuid: &str, key: &str, value: &str) -> bool {
            self.inner
                .set_build_setting(target_uuid, key, crate::types::PlistValue::String(value.to_string()))
        }

        #[wasm_bindgen(js_name = "removeBuildSetting")]
        pub fn remove_build_setting(&mut self, target_uuid: &str, key: &str) -> bool {
            self.inner.remove_build_setting(target_uuid, key)
        }

        // ── Files & groups ───────────────────────────────────────

        #[wasm_bindgen(js_name = "addFile")]
        pub fn add_file(&mut self, group_uuid: &str, path: &str) -> Option<String> {
            self.inner.add_file(group_uuid, path)
        }

        #[wasm_bindgen(js_name = "addGroup")]
        pub fn add_group(&mut self, parent_uuid: &str, name: &str) -> Option<String> {
            self.inner.add_group(parent_uuid, name)
        }

        #[wasm_bindgen(js_name = "getGroupChildren")]
        pub fn get_group_children(&self, group_uuid: &str) -> Vec<String> {
            self.inner.get_group_children(group_uuid)
        }

        // ── Build phases ─────────────────────────────────────────

        #[wasm_bindgen(js_name = "ensureBuildPhase")]
        pub fn ensure_build_phase(&mut self, target_uuid: &str, phase_isa: &str) -> Option<String> {
            self.inner.ensure_build_phase(target_uuid, phase_isa)
        }

        #[wasm_bindgen(js_name = "addBuildFile")]
        pub fn add_build_file(&mut self, phase_uuid: &str, file_ref_uuid: &str) -> Option<String> {
            self.inner.add_build_file(phase_uuid, file_ref_uuid)
        }

        #[wasm_bindgen(js_name = "addFramework")]
        pub fn add_framework(&mut self, target_uuid: &str, framework_name: &str) -> Option<String> {
            self.inner.add_framework(target_uuid, framework_name)
        }

        // ── Dependencies & embedding ─────────────────────────────

        #[wasm_bindgen(js_name = "addDependency")]
        pub fn add_dependency(&mut self, target_uuid: &str, depends_on: &str) -> Option<String> {
            self.inner.add_dependency(target_uuid, depends_on)
        }

        #[wasm_bindgen(js_name = "getEmbeddedTargets")]
        pub fn get_embedded_targets(&self, target_uuid: &str) -> Vec<String> {
            self.inner.get_embedded_targets(target_uuid)
        }

        #[wasm_bindgen(js_name = "embedExtension")]
        pub fn embed_extension(&mut self, host: &str, extension: &str) -> Option<String> {
            self.inner.embed_extension(host, extension)
        }

        #[wasm_bindgen(js_name = "addFileSystemSyncGroup")]
        pub fn add_file_system_sync_group(&mut self, target_uuid: &str, path: &str) -> Option<String> {
            self.inner.add_file_system_sync_group(target_uuid, path)
        }

        #[wasm_bindgen(js_name = "getTargetSyncGroupPaths")]
        pub fn get_target_sync_group_paths(&self, target_uuid: &str) -> Vec<String> {
            self.inner.get_target_sync_group_paths(target_uuid)
        }

        // ── Generic access ───────────────────────────────────────

        #[wasm_bindgen(js_name = "getObjectProperty")]
        pub fn get_object_property(&self, uuid: &str, key: &str) -> Option<String> {
            self.inner.get_object_property(uuid, key)
        }

        #[wasm_bindgen(js_name = "setObjectProperty")]
        pub fn set_object_property(&mut self, uuid: &str, key: &str, value: &str) -> bool {
            self.inner.set_object_property(uuid, key, value)
        }

        #[wasm_bindgen(js_name = "findObjectsByIsa")]
        pub fn find_objects_by_isa(&self, isa: &str) -> Vec<String> {
            self.inner.find_objects_by_isa(isa)
        }

        #[wasm_bindgen(js_name = "getUniqueId")]
        pub fn get_unique_id(&self, seed: &str) -> String {
            self.inner.get_unique_id(seed)
        }

        #[wasm_bindgen(js_name = "findOrphanedReferences")]
        pub fn find_orphaned_references(&self) -> String {
            let orphans = self.inner.find_orphaned_references();
            serde_json::to_string(
                &orphans
                    .iter()
                    .map(|o| {
                        serde_json::json!({
                            "referrerUuid": o.referrer_uuid,
                            "referrerIsa": o.referrer_isa,
                            "property": o.property,
                            "orphanUuid": o.orphan_uuid,
                        })
                    })
                    .collect::<Vec<_>>(),
            )
            .unwrap_or_else(|_| "[]".to_string())
        }
    }
}

#[cfg(feature = "napi")]
mod napi_bindings {
    use napi::bindgen_prelude::*;

    /// Parse a .pbxproj string into a JSON-compatible object.
    #[napi]
    pub fn parse(text: String) -> Result<serde_json::Value> {
        let plist = crate::parser::parse(&text).map_err(|e| Error::from_reason(e))?;
        serde_json::to_value(&plist).map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Serialize a JSON object back to .pbxproj format.
    #[napi]
    pub fn build(project: serde_json::Value) -> Result<String> {
        let plist: crate::types::PlistValue =
            serde_json::from_value(project).map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(crate::writer::serializer::build(&plist))
    }

    /// Serialize a JSON string back to .pbxproj format.
    /// Faster than `build()` — accepts `JSON.stringify(project)` directly,
    /// avoiding napi's recursive JS→Rust object marshalling.
    #[napi(js_name = "buildFromJSON")]
    pub fn build_from_json(json: String) -> Result<String> {
        let plist: crate::types::PlistValue =
            serde_json::from_str(&json).map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(crate::writer::serializer::build(&plist))
    }

    /// Parse and immediately re-serialize a .pbxproj string.
    /// Fastest path — stays entirely in Rust, zero JS↔Rust marshalling.
    #[napi(js_name = "parseAndBuild")]
    pub fn parse_and_build(text: String) -> Result<String> {
        let plist = crate::parser::parse(&text).map_err(|e| Error::from_reason(e))?;
        Ok(crate::writer::serializer::build(&plist))
    }

    /// XcodeProject class for high-level API.
    #[napi]
    pub struct XcodeProject {
        inner: crate::project::XcodeProject,
    }

    #[napi]
    impl XcodeProject {
        /// Open and parse a .pbxproj file from disk.
        #[napi(factory)]
        pub fn open(file_path: String) -> Result<Self> {
            let inner = crate::project::XcodeProject::open(&file_path).map_err(|e| Error::from_reason(e))?;
            Ok(XcodeProject { inner })
        }

        /// Parse a .pbxproj string into an XcodeProject (no file on disk needed).
        #[napi(factory, js_name = "fromString")]
        pub fn from_string(content: String) -> Result<Self> {
            let inner = crate::project::XcodeProject::from_plist(&content).map_err(|e| Error::from_reason(e))?;
            Ok(XcodeProject { inner })
        }

        /// Convert the project to a JSON-compatible object.
        #[napi(js_name = "toJSON")]
        pub fn to_json(&self) -> Result<serde_json::Value> {
            self.inner.to_json().map_err(|e| Error::from_reason(e))
        }

        /// Serialize the project back to .pbxproj format.
        #[napi(js_name = "toBuild")]
        pub fn to_build(&self) -> String {
            self.inner.to_pbxproj()
        }

        /// Write the project back to its original file.
        #[napi]
        pub fn save(&self) -> Result<()> {
            self.inner.save().map_err(|e| Error::from_reason(e))
        }

        /// Get the file path this project was loaded from.
        #[napi(getter)]
        pub fn file_path(&self) -> Option<String> {
            self.inner.file_path().map(|s| s.to_string())
        }

        /// Get the archive version.
        #[napi(getter)]
        pub fn archive_version(&self) -> i64 {
            self.inner.archive_version
        }

        /// Get the object version.
        #[napi(getter)]
        pub fn object_version(&self) -> i64 {
            self.inner.object_version
        }

        /// Get all native target UUIDs.
        #[napi]
        pub fn get_native_targets(&self) -> Vec<String> {
            self.inner.native_targets().iter().map(|t| t.uuid.clone()).collect()
        }

        /// Get a build setting value from a target.
        #[napi]
        pub fn get_build_setting(&self, target_uuid: String, key: String) -> Result<serde_json::Value> {
            match self.inner.get_build_setting(&target_uuid, &key) {
                Some(val) => serde_json::to_value(&val).map_err(|e| Error::from_reason(e.to_string())),
                None => Ok(serde_json::Value::Null),
            }
        }

        /// Set a build setting on all configurations for a target.
        #[napi]
        pub fn set_build_setting(&mut self, target_uuid: String, key: String, value: String) -> bool {
            self.inner
                .set_build_setting(&target_uuid, &key, crate::types::PlistValue::String(value))
        }

        /// Remove a build setting from all configurations for a target.
        #[napi]
        pub fn remove_build_setting(&mut self, target_uuid: String, key: String) -> bool {
            self.inner.remove_build_setting(&target_uuid, &key)
        }

        /// Find orphaned references (UUIDs referenced but not present in objects).
        /// Returns array of { referrerUuid, referrerIsa, property, orphanUuid }.
        #[napi(js_name = "findOrphanedReferences")]
        pub fn find_orphaned_references(&self) -> Vec<serde_json::Value> {
            self.inner
                .find_orphaned_references()
                .into_iter()
                .map(|o| {
                    serde_json::json!({
                        "referrerUuid": o.referrer_uuid,
                        "referrerIsa": o.referrer_isa,
                        "property": o.property,
                        "orphanUuid": o.orphan_uuid,
                    })
                })
                .collect()
        }

        /// Find the main app target UUID.
        #[napi]
        pub fn find_main_app_target(&self, platform: Option<String>) -> Option<String> {
            let platform = platform.as_deref().unwrap_or("ios");
            self.inner.find_main_app_target(platform).map(|t| t.uuid.clone())
        }

        /// Generate a unique UUID.
        #[napi]
        pub fn get_unique_id(&self, seed: String) -> String {
            self.inner.get_unique_id(&seed)
        }

        // ── File & group operations ──────────────────────────────

        /// Get the main group UUID.
        #[napi(getter, js_name = "mainGroupUuid")]
        pub fn main_group_uuid(&self) -> Option<String> {
            self.inner.main_group_uuid()
        }

        /// Get children UUIDs of a group.
        #[napi]
        pub fn get_group_children(&self, group_uuid: String) -> Vec<String> {
            self.inner.get_group_children(&group_uuid)
        }

        /// Add a file reference to the project and a group.
        /// Returns the UUID of the new PBXFileReference.
        #[napi]
        pub fn add_file(&mut self, group_uuid: String, path: String) -> Option<String> {
            self.inner.add_file(&group_uuid, &path)
        }

        /// Create a group and add it as a child of a parent group.
        /// Returns the UUID of the new PBXGroup.
        #[napi]
        pub fn add_group(&mut self, parent_uuid: String, name: String) -> Option<String> {
            self.inner.add_group(&parent_uuid, &name)
        }

        // ── Build phase operations ───────────────────────────────

        /// Add a build file to a build phase.
        /// Returns the UUID of the new PBXBuildFile.
        #[napi]
        pub fn add_build_file(&mut self, phase_uuid: String, file_ref_uuid: String) -> Option<String> {
            self.inner.add_build_file(&phase_uuid, &file_ref_uuid)
        }

        /// Find or create a build phase for a target.
        /// Returns the UUID of the build phase.
        #[napi]
        pub fn ensure_build_phase(&mut self, target_uuid: String, phase_isa: String) -> Option<String> {
            self.inner.ensure_build_phase(&target_uuid, &phase_isa)
        }

        /// Add a framework to a target.
        /// Returns the UUID of the PBXBuildFile.
        #[napi]
        pub fn add_framework(&mut self, target_uuid: String, framework_name: String) -> Option<String> {
            self.inner.add_framework(&target_uuid, &framework_name)
        }

        // ── Target operations ────────────────────────────────────

        /// Create a native target with Debug/Release configs, standard build phases, and product ref.
        /// Returns the UUID of the new PBXNativeTarget.
        #[napi]
        pub fn create_native_target(
            &mut self,
            name: String,
            product_type: String,
            bundle_id: String,
        ) -> Option<String> {
            self.inner.create_native_target(&name, &product_type, &bundle_id)
        }

        /// Add a dependency from one target to another.
        /// Returns the UUID of the PBXTargetDependency.
        #[napi]
        pub fn add_dependency(&mut self, target_uuid: String, depends_on_uuid: String) -> Option<String> {
            self.inner.add_dependency(&target_uuid, &depends_on_uuid)
        }

        /// Get UUIDs of targets embedded in the given target via PBXCopyFilesBuildPhase.
        #[napi]
        pub fn get_embedded_targets(&self, target_uuid: String) -> Vec<String> {
            self.inner.get_embedded_targets(&target_uuid)
        }

        /// Embed an extension target into a host app target.
        /// Creates PBXCopyFilesBuildPhase with correct dstSubfolderSpec.
        /// Returns the UUID of the copy files build phase.
        #[napi]
        pub fn embed_extension(&mut self, host_target_uuid: String, extension_target_uuid: String) -> Option<String> {
            self.inner.embed_extension(&host_target_uuid, &extension_target_uuid)
        }

        /// Add a PBXFileSystemSynchronizedRootGroup to a target (Xcode 16+).
        /// Returns the UUID of the sync group.
        #[napi]
        pub fn add_file_system_sync_group(&mut self, target_uuid: String, path: String) -> Option<String> {
            self.inner.add_file_system_sync_group(&target_uuid, &path)
        }

        /// Get the on-disk paths of a target's file system sync groups.
        #[napi]
        pub fn get_target_sync_group_paths(&self, target_uuid: String) -> Vec<String> {
            self.inner.get_target_sync_group_paths(&target_uuid)
        }

        // ── Generic property access ──────────────────────────────

        /// Get a string property from any object.
        #[napi]
        pub fn get_object_property(&self, uuid: String, key: String) -> Option<String> {
            self.inner.get_object_property(&uuid, &key)
        }

        /// Set a string property on any object.
        #[napi]
        pub fn set_object_property(&mut self, uuid: String, key: String, value: String) -> bool {
            self.inner.set_object_property(&uuid, &key, &value)
        }

        /// Find all object UUIDs matching a given ISA type.
        #[napi]
        pub fn find_objects_by_isa(&self, isa: String) -> Vec<String> {
            self.inner.find_objects_by_isa(&isa)
        }

        /// Get the name of a target.
        #[napi]
        pub fn get_target_name(&self, target_uuid: String) -> Option<String> {
            self.inner.get_target_name(&target_uuid)
        }

        /// Get the product type of a target.
        #[napi]
        pub fn get_target_product_type(&self, target_uuid: String) -> Option<String> {
            self.inner.get_target_product_type(&target_uuid)
        }

        /// Set the name and productName of a target.
        #[napi]
        pub fn set_target_name(&mut self, target_uuid: String, name: String) -> bool {
            self.inner.set_target_name(&target_uuid, &name)
        }

        /// Rename a target and cascade through the project (group paths, product refs, proxies).
        #[napi]
        pub fn rename_target(&mut self, target_uuid: String, old_name: String, new_name: String) -> bool {
            self.inner.rename_target(&target_uuid, &old_name, &new_name)
        }
    }
}
