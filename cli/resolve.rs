use std::fmt;

use clap::ValueEnum;
use serde::Deserialize;
use xcodekit::project::XcodeProject;

use crate::output::{CliError, ErrorCode};

/// Build phase types. Works with both clap (CLI args) and serde (batch JSON).
#[derive(Debug, Clone, ValueEnum, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PhaseType {
    Sources,
    Frameworks,
    Resources,
    Headers,
    #[serde(alias = "copy-files")]
    #[value(alias = "copy-files")]
    CopyFiles,
    #[serde(alias = "shell-script")]
    #[value(alias = "shell-script")]
    ShellScript,
}

impl fmt::Display for PhaseType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_isa())
    }
}

impl PhaseType {
    pub fn as_isa(&self) -> &'static str {
        match self {
            Self::Sources => "PBXSourcesBuildPhase",
            Self::Frameworks => "PBXFrameworksBuildPhase",
            Self::Resources => "PBXResourcesBuildPhase",
            Self::Headers => "PBXHeadersBuildPhase",
            Self::CopyFiles => "PBXCopyFilesBuildPhase",
            Self::ShellScript => "PBXShellScriptBuildPhase",
        }
    }
}

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

    let matches: Vec<_> =
        project.native_targets().iter().filter(|t| t.get_str("name") == Some(query)).map(|t| t.uuid.clone()).collect();

    match matches.len() {
        0 => Err(CliError::target_not_found(query)),
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => Err(CliError::new(ErrorCode::AmbiguousReference, format!("Multiple targets matched '{}'", query))),
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
        _ => Err(CliError::new(ErrorCode::AmbiguousReference, format!("Multiple groups matched '{}'", query))),
    }
}
