#[cfg(feature = "napi")]
#[macro_use]
extern crate napi_derive;

pub mod objects;
pub mod parser;
pub mod project;
pub mod types;
pub mod writer;

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
    }
}
