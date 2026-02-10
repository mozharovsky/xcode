use crate::objects::PbxObject;

use super::xcode_project::XcodeProject;

/// Resolve the real filesystem path for a file reference or group.
///
/// Port of `getRealPath` from `paths.ts`.
pub fn get_real_path(project: &XcodeProject, object: &PbxObject) -> Option<String> {
    let source_tree_path = get_source_tree_real_path(project, object)?;
    let path = object.get_str("path").unwrap_or("");
    if source_tree_path.is_empty() && path.is_empty() {
        return None;
    }
    if source_tree_path.is_empty() {
        Some(path.to_string())
    } else if path.is_empty() {
        Some(source_tree_path)
    } else {
        Some(format!("{}/{}", source_tree_path, path))
    }
}

/// Resolve the source tree base path for an object.
///
/// Port of `getSourceTreeRealPath` from `paths.ts`.
pub fn get_source_tree_real_path(project: &XcodeProject, object: &PbxObject) -> Option<String> {
    let source_tree = object.get_str("sourceTree")?;

    match source_tree {
        "<group>" => {
            // Walk up to parent group
            let parent = get_parent(project, object)?;
            if parent.isa == "PBXProject" {
                // At the root — use project root + projectDirPath
                let project_root = project.get_project_root().unwrap_or_default();
                let project_dir = parent.get_str("projectDirPath").unwrap_or("");
                if project_dir.is_empty() {
                    Some(project_root)
                } else {
                    Some(format!("{}/{}", project_root, project_dir))
                }
            } else {
                get_real_path(project, &parent)
            }
        }
        "SOURCE_ROOT" => project.get_project_root(),
        "<absolute>" => Some(String::new()),
        // Other source trees like SDKROOT, BUILT_PRODUCTS_DIR, etc.
        other => Some(other.to_string()),
    }
}

/// Get the full (project-relative) path for an object.
///
/// Port of `getFullPath` from `paths.ts`.
pub fn get_full_path(project: &XcodeProject, object: &PbxObject) -> Option<String> {
    let root_path = get_resolved_root_path(project, object);
    let path = object.get_str("path").unwrap_or("");

    if path.is_empty() {
        root_path
    } else if let Some(root) = root_path {
        if root.is_empty() {
            Some(path.to_string())
        } else {
            Some(format!("{}/{}", root, path))
        }
    } else {
        Some(path.to_string())
    }
}

fn get_resolved_root_path(project: &XcodeProject, object: &PbxObject) -> Option<String> {
    let source_tree = object.get_str("sourceTree")?;

    match source_tree {
        "<group>" => {
            let parent = get_parent(project, object)?;
            if parent.isa == "PBXProject" {
                Some(String::new())
            } else {
                get_full_path(project, &parent)
            }
        }
        "SOURCE_ROOT" => Some(String::new()),
        "<absolute>" => Some("/".to_string()),
        other => Some(other.to_string()),
    }
}

/// Find the parent group/project for an object.
fn get_parent(project: &XcodeProject, object: &PbxObject) -> Option<PbxObject> {
    let referrers = project.get_referrers(&object.uuid);

    // Filter to groups and the project
    let groups: Vec<&&PbxObject> = referrers
        .iter()
        .filter(|r| r.isa == "PBXGroup" || r.isa == "PBXVariantGroup" || r.isa == "PBXProject")
        .collect();

    groups.first().map(|g| (**g).clone())
}

/// Get all parent groups up to the root.
pub fn get_parents(project: &XcodeProject, object: &PbxObject) -> Vec<PbxObject> {
    let main_group = project.main_group_uuid();
    if main_group.as_deref() == Some(&object.uuid) {
        return vec![];
    }

    if let Some(parent) = get_parent(project, object) {
        let mut parents = get_parents(project, &parent);
        parents.push(parent);
        parents
    } else {
        vec![]
    }
}
