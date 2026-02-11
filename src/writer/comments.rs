use std::collections::HashMap;

use indexmap::IndexMap;

use crate::types::PlistValue;

/// Build a map of UUID → inline comment for serialization.
///
/// Replicates `createReferenceList` from comments.ts.
pub fn create_reference_list(project: &PlistValue) -> HashMap<String, String> {
    let mut cache: HashMap<String, String> = HashMap::new();

    let objects = match project
        .as_object()
        .and_then(|p| p.get("objects"))
        .and_then(|o| o.as_object())
    {
        Some(o) => o,
        None => return cache,
    };

    // Pre-build reverse index: build_file_uuid → (phase_isa, phase_name)
    // This eliminates the O(n²) scan in get_build_phase_name_containing_file
    let mut file_to_phase: HashMap<&str, (&str, Option<&str>)> = HashMap::new();
    for (_id, obj) in objects {
        if let Some(obj_map) = obj.as_object() {
            let isa = obj_map.get("isa").and_then(|v| v.as_str()).unwrap_or("");
            if isa.ends_with("BuildPhase") {
                let phase_name = obj_map.get("name").and_then(|v| v.as_str());
                if let Some(files) = obj_map.get("files").and_then(|f| f.as_array()) {
                    for f in files {
                        if let Some(file_uuid) = f.as_str() {
                            file_to_phase.insert(file_uuid, (isa, phase_name));
                        }
                    }
                }
            }
        }
    }

    // Process all objects to build comments
    for (id, object) in objects {
        get_comment_for_object(id, object, objects, &file_to_phase, &mut cache);
    }

    cache
}

fn get_comment_for_object<'a>(
    id: &str,
    object: &'a PlistValue,
    objects: &'a IndexMap<String, PlistValue>,
    file_to_phase: &HashMap<&str, (&str, Option<&str>)>,
    cache: &mut HashMap<String, String>,
) -> Option<String> {
    let obj = object.as_object()?;
    let isa = obj.get("isa").and_then(|v| v.as_str())?;

    if let Some(cached) = cache.get(id) {
        return Some(cached.clone());
    }

    let comment = if isa == "PBXBuildFile" {
        get_pbx_build_file_comment(id, obj, objects, file_to_phase, cache)
    } else if isa == "XCConfigurationList" {
        Some(get_xc_configuration_list_comment(id, objects))
    } else if isa == "XCRemoteSwiftPackageReference" {
        let repo_url = obj.get("repositoryURL").and_then(|v| v.as_str());
        if let Some(url) = repo_url {
            Some(format!("{} \"{}\"", isa, get_repo_name_from_url(url)))
        } else {
            Some(isa.to_string())
        }
    } else if isa == "XCLocalSwiftPackageReference" {
        let path = obj.get("relativePath").and_then(|v| v.as_str());
        if let Some(p) = path {
            Some(format!("{} \"{}\"", isa, p))
        } else {
            Some(isa.to_string())
        }
    } else if isa == "PBXProject" {
        Some("Project object".to_string())
    } else if isa.ends_with("BuildPhase") {
        Some(get_build_phase_name(obj, isa))
    } else if isa == "PBXGroup" {
        let has_name = obj.get("name").and_then(|v| v.as_str()).is_some();
        let has_path = obj.get("path").and_then(|v| v.as_str()).is_some();
        if !has_name && !has_path {
            Some(String::new())
        } else {
            get_default_name(obj, isa)
        }
    } else {
        get_default_name(obj, isa)
    };

    if let Some(ref c) = comment {
        cache.insert(id.to_string(), c.clone());
    }

    comment
}

fn get_default_name(obj: &IndexMap<String, PlistValue>, isa: &str) -> Option<String> {
    obj.get("name")
        .and_then(|v| v.as_str())
        .or_else(|| obj.get("productName").and_then(|v| v.as_str()))
        .or_else(|| obj.get("path").and_then(|v| v.as_str()))
        .map(|s| s.to_string())
        .or_else(|| Some(isa.to_string()))
}

fn get_pbx_build_file_comment(
    id: &str,
    build_file: &IndexMap<String, PlistValue>,
    objects: &IndexMap<String, PlistValue>,
    file_to_phase: &HashMap<&str, (&str, Option<&str>)>,
    cache: &mut HashMap<String, String>,
) -> Option<String> {
    // O(1) lookup instead of O(n) scan
    let build_phase_name = if let Some(&(isa, name)) = file_to_phase.get(id) {
        name.map(|n| n.to_string())
            .unwrap_or_else(|| get_default_build_phase_name(isa).unwrap_or_default())
    } else {
        "[missing build phase]".to_string()
    };

    let ref_id = build_file
        .get("fileRef")
        .or_else(|| build_file.get("productRef"))
        .and_then(|v| v.as_str());

    let name = if let Some(ref_id) = ref_id {
        if let Some(ref_obj) = objects.get(ref_id) {
            get_comment_for_object(ref_id, ref_obj, objects, file_to_phase, cache)
                .unwrap_or_else(|| "(null)".to_string())
        } else {
            "(null)".to_string()
        }
    } else {
        "(null)".to_string()
    };

    Some(format!("{} in {}", name, build_phase_name))
}

fn get_build_phase_name(obj: &IndexMap<String, PlistValue>, isa: &str) -> String {
    if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
        return name.to_string();
    }
    get_default_build_phase_name(isa).unwrap_or_default()
}

/// Extract the default name from a build phase ISA.
/// e.g., "PBXSourcesBuildPhase" → "Sources"
fn get_default_build_phase_name(isa: &str) -> Option<String> {
    if let Some(start) = isa.strip_prefix("PBX") {
        if let Some(name) = start.strip_suffix("BuildPhase") {
            return Some(name.to_string());
        }
    }
    None
}

fn get_xc_configuration_list_comment(id: &str, objects: &IndexMap<String, PlistValue>) -> String {
    for (inner_id, obj) in objects {
        if let Some(obj_map) = obj.as_object() {
            let config_list = obj_map.get("buildConfigurationList").and_then(|v| v.as_str());
            if config_list == Some(id) {
                let isa = obj_map.get("isa").and_then(|v| v.as_str()).unwrap_or("");

                let name = obj_map
                    .get("name")
                    .and_then(|v| v.as_str())
                    .or_else(|| obj_map.get("path").and_then(|v| v.as_str()))
                    .or_else(|| obj_map.get("productName").and_then(|v| v.as_str()));

                if let Some(name) = name {
                    return format!("Build configuration list for {} \"{}\"", isa, name);
                }

                if let Some(targets) = obj_map.get("targets").and_then(|v| v.as_array()) {
                    if let Some(first_target_id) = targets.first().and_then(|v| v.as_str()) {
                        if let Some(target_obj) = objects.get(first_target_id).and_then(|v| v.as_object()) {
                            let target_name = target_obj
                                .get("productName")
                                .or_else(|| target_obj.get("name"))
                                .and_then(|v| v.as_str());
                            if let Some(name) = target_name {
                                return format!("Build configuration list for {} \"{}\"", isa, name);
                            }
                        }
                    }
                }

                let proxy_name = objects.values().find_map(|v| {
                    let m = v.as_object()?;
                    if m.get("isa").and_then(|v| v.as_str()) == Some("PBXContainerItemProxy")
                        && m.get("containerPortal").and_then(|v| v.as_str()) == Some(inner_id)
                    {
                        m.get("remoteInfo").and_then(|v| v.as_str()).map(|s| s.to_string())
                    } else {
                        None
                    }
                });

                if let Some(name) = proxy_name {
                    return format!("Build configuration list for {} \"{}\"", isa, name);
                }

                return format!("Build configuration list for {}", isa);
            }
        }
    }
    "Build configuration list for [unknown]".to_string()
}

fn get_repo_name_from_url(repo_url: &str) -> String {
    if let Some(path) = repo_url.strip_prefix("https://github.com/") {
        if let Some(name) = path.split('/').last() {
            let name = name.strip_suffix(".git").unwrap_or(name);
            if !name.is_empty() {
                return name.to_string();
            }
        }
    }
    if let Some(path) = repo_url.strip_prefix("http://github.com/") {
        if let Some(name) = path.split('/').last() {
            let name = name.strip_suffix(".git").unwrap_or(name);
            if !name.is_empty() {
                return name.to_string();
            }
        }
    }
    repo_url.to_string()
}

/// Check if an object's ISA is PBXBuildFile.
pub fn is_pbx_build_file(isa: &str) -> bool {
    isa == "PBXBuildFile"
}

/// Check if an object's ISA is PBXFileReference.
pub fn is_pbx_file_reference(isa: &str) -> bool {
    isa == "PBXFileReference"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_build_phase_name() {
        assert_eq!(
            get_default_build_phase_name("PBXSourcesBuildPhase"),
            Some("Sources".to_string())
        );
        assert_eq!(
            get_default_build_phase_name("PBXFrameworksBuildPhase"),
            Some("Frameworks".to_string())
        );
        assert_eq!(
            get_default_build_phase_name("PBXResourcesBuildPhase"),
            Some("Resources".to_string())
        );
        assert_eq!(get_default_build_phase_name("PBXProject"), None);
    }

    #[test]
    fn test_repo_name_from_url() {
        assert_eq!(
            get_repo_name_from_url("https://github.com/expo/spm-package"),
            "spm-package"
        );
        assert_eq!(get_repo_name_from_url("https://github.com/user/repo.git"), "repo");
        assert_eq!(
            get_repo_name_from_url("https://example.com/custom"),
            "https://example.com/custom"
        );
    }
}
