use std::collections::HashMap;

use crate::types::PlistValue;

/// Build a map of UUID → inline comment for serialization.
///
/// Replicates `createReferenceList` from comments.ts.
pub fn create_reference_list(project: &PlistValue<'_>) -> HashMap<String, String> {
    let mut cache: HashMap<String, String> = HashMap::new();

    let objects = match project.get("objects").and_then(|o| o.as_object()) {
        Some(o) => o,
        None => return cache,
    };

    // Build O(1) lookup index for the objects dict (~4000 entries)
    let index: HashMap<&str, &PlistValue<'_>> =
        objects.iter().map(|(k, v)| (k.as_ref(), v)).collect();

    // Pre-build reverse index: build_file_uuid → (phase_isa, phase_name)
    // This eliminates the O(n²) scan in get_build_phase_name_containing_file
    let mut file_to_phase: HashMap<&str, (&str, Option<&str>)> = HashMap::new();
    for (_id, obj) in objects {
        let isa = obj.get("isa").and_then(|v| v.as_str()).unwrap_or("");
        if isa.ends_with("BuildPhase") {
            let phase_name = obj.get("name").and_then(|v| v.as_str());
            if let Some(files) = obj.get("files").and_then(|f| f.as_array()) {
                for f in files {
                    if let Some(file_uuid) = f.as_str() {
                        file_to_phase.insert(file_uuid, (isa, phase_name));
                    }
                }
            }
        }
    }

    // Process all objects to build comments
    for (id, object) in objects {
        get_comment_for_object(id, object, &index, &file_to_phase, &mut cache);
    }

    cache
}

fn get_comment_for_object<'a>(
    id: &str,
    object: &PlistValue<'a>,
    objects: &HashMap<&str, &PlistValue<'a>>,
    file_to_phase: &HashMap<&str, (&str, Option<&str>)>,
    cache: &mut HashMap<String, String>,
) -> Option<String> {
    object.as_object()?;
    let isa = object.get("isa").and_then(|v| v.as_str())?;

    if let Some(cached) = cache.get(id) {
        return Some(cached.clone());
    }

    let comment = if isa == "PBXBuildFile" {
        get_pbx_build_file_comment(id, object, objects, file_to_phase, cache)
    } else if isa == "XCConfigurationList" {
        Some(get_xc_configuration_list_comment(id, objects))
    } else if isa == "XCRemoteSwiftPackageReference" {
        let repo_url = object.get("repositoryURL").and_then(|v| v.as_str());
        if let Some(url) = repo_url {
            Some(format!("{} \"{}\"", isa, get_repo_name_from_url(url)))
        } else {
            Some(isa.to_string())
        }
    } else if isa == "XCLocalSwiftPackageReference" {
        let path = object.get("relativePath").and_then(|v| v.as_str());
        if let Some(p) = path {
            Some(format!("{} \"{}\"", isa, p))
        } else {
            Some(isa.to_string())
        }
    } else if isa == "PBXProject" {
        Some("Project object".to_string())
    } else if isa.ends_with("BuildPhase") {
        Some(get_build_phase_name(object, isa))
    } else if isa == "PBXGroup" {
        let has_name = object.get("name").and_then(|v| v.as_str()).is_some();
        let has_path = object.get("path").and_then(|v| v.as_str()).is_some();
        if !has_name && !has_path {
            Some(String::new())
        } else {
            get_default_name(object, isa)
        }
    } else {
        get_default_name(object, isa)
    };

    if let Some(ref c) = comment {
        cache.insert(id.to_string(), c.clone());
    }

    comment
}

fn get_default_name(obj: &PlistValue<'_>, isa: &str) -> Option<String> {
    obj.get("name")
        .and_then(|v| v.as_str())
        .or_else(|| obj.get("productName").and_then(|v| v.as_str()))
        .or_else(|| obj.get("path").and_then(|v| v.as_str()))
        .map(|s| s.to_string())
        .or_else(|| Some(isa.to_string()))
}

fn get_pbx_build_file_comment<'a>(
    id: &str,
    build_file: &PlistValue<'a>,
    objects: &HashMap<&str, &PlistValue<'a>>,
    file_to_phase: &HashMap<&str, (&str, Option<&str>)>,
    cache: &mut HashMap<String, String>,
) -> Option<String> {
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

fn get_build_phase_name(obj: &PlistValue<'_>, isa: &str) -> String {
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

fn get_xc_configuration_list_comment<'a>(
    id: &str,
    objects: &HashMap<&str, &PlistValue<'a>>,
) -> String {
    for (&inner_id, obj) in objects {
        if obj.as_object().is_some() {
            let config_list = obj.get("buildConfigurationList").and_then(|v| v.as_str());
            if config_list == Some(id) {
                let isa = obj.get("isa").and_then(|v| v.as_str()).unwrap_or("");

                let name = obj
                    .get("name")
                    .and_then(|v| v.as_str())
                    .or_else(|| obj.get("path").and_then(|v| v.as_str()))
                    .or_else(|| obj.get("productName").and_then(|v| v.as_str()));

                if let Some(name) = name {
                    return format!("Build configuration list for {} \"{}\"", isa, name);
                }

                if let Some(targets) = obj.get("targets").and_then(|v| v.as_array()) {
                    if let Some(first_target_id) = targets.first().and_then(|v| v.as_str()) {
                        if let Some(target_val) = objects.get(first_target_id) {
                            let target_name = target_val
                                .get("productName")
                                .or_else(|| target_val.get("name"))
                                .and_then(|v| v.as_str());
                            if let Some(name) = target_name {
                                return format!("Build configuration list for {} \"{}\"", isa, name);
                            }
                        }
                    }
                }

                let proxy_name = objects.values().find_map(|val| {
                    val.as_object()?;
                    if val.get("isa").and_then(|v| v.as_str()) == Some("PBXContainerItemProxy")
                        && val.get("containerPortal").and_then(|v| v.as_str()) == Some(inner_id)
                    {
                        val.get("remoteInfo").and_then(|v| v.as_str()).map(|s| s.to_string())
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
