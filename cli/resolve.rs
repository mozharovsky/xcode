use xcodekit::project::XcodeProject;

use crate::output::CliError;

fn looks_like_uuid(s: &str) -> bool {
    s.len() == 24 && s.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn resolve_target(project: &XcodeProject, query: &str) -> Result<String, CliError> {
    if looks_like_uuid(query) {
        if project.get_object(query).is_some() {
            return Ok(query.to_string());
        }
        return Err(CliError::target_not_found(query));
    }

    let matches: Vec<_> = project
        .native_targets()
        .iter()
        .filter(|t| t.get_str("name") == Some(query))
        .map(|t| t.uuid.clone())
        .collect();

    match matches.len() {
        0 => Err(CliError::target_not_found(query)),
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => Err(CliError::new(
            "AMBIGUOUS_REFERENCE",
            format!("Multiple targets matched '{}'", query),
        )),
    }
}

pub fn resolve_group(project: &XcodeProject, query: &str) -> Result<String, CliError> {
    if looks_like_uuid(query) {
        if project.get_object(query).is_some() {
            return Ok(query.to_string());
        }
        return Err(CliError::group_not_found(query));
    }

    let matches: Vec<_> = project
        .objects_by_isa("PBXGroup")
        .iter()
        .filter(|g| g.get_str("name") == Some(query) || g.get_str("path") == Some(query))
        .map(|g| g.uuid.clone())
        .collect();

    match matches.len() {
        0 => Err(CliError::group_not_found(query)),
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => Err(CliError::new(
            "AMBIGUOUS_REFERENCE",
            format!("Multiple groups matched '{}'", query),
        )),
    }
}
