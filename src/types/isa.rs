use std::fmt;
use std::str::FromStr;

/// All known ISA types in Xcode project files.
///
/// ISA stands for "is a" — a reference to Objective-C's isa pointer.
/// Each object in a .pbxproj file has an `isa` field indicating its type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Isa {
    PBXBuildFile,
    PBXAppleScriptBuildPhase,
    PBXCopyFilesBuildPhase,
    PBXFrameworksBuildPhase,
    PBXHeadersBuildPhase,
    PBXResourcesBuildPhase,
    PBXShellScriptBuildPhase,
    PBXSourcesBuildPhase,
    PBXRezBuildPhase,
    PBXContainerItemProxy,
    PBXFileReference,
    PBXGroup,
    PBXVariantGroup,
    XCVersionGroup,
    PBXFileSystemSynchronizedRootGroup,
    PBXFileSystemSynchronizedBuildFileExceptionSet,
    PBXFileSystemSynchronizedGroupBuildPhaseMembershipExceptionSet,
    PBXNativeTarget,
    PBXAggregateTarget,
    PBXLegacyTarget,
    PBXProject,
    PBXTargetDependency,
    XCBuildConfiguration,
    XCConfigurationList,
    PBXBuildRule,
    PBXReferenceProxy,
    XCSwiftPackageProductDependency,
    XCRemoteSwiftPackageReference,
    XCLocalSwiftPackageReference,
}

impl fmt::Display for Isa {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Isa::PBXBuildFile => "PBXBuildFile",
            Isa::PBXAppleScriptBuildPhase => "PBXAppleScriptBuildPhase",
            Isa::PBXCopyFilesBuildPhase => "PBXCopyFilesBuildPhase",
            Isa::PBXFrameworksBuildPhase => "PBXFrameworksBuildPhase",
            Isa::PBXHeadersBuildPhase => "PBXHeadersBuildPhase",
            Isa::PBXResourcesBuildPhase => "PBXResourcesBuildPhase",
            Isa::PBXShellScriptBuildPhase => "PBXShellScriptBuildPhase",
            Isa::PBXSourcesBuildPhase => "PBXSourcesBuildPhase",
            Isa::PBXRezBuildPhase => "PBXRezBuildPhase",
            Isa::PBXContainerItemProxy => "PBXContainerItemProxy",
            Isa::PBXFileReference => "PBXFileReference",
            Isa::PBXGroup => "PBXGroup",
            Isa::PBXVariantGroup => "PBXVariantGroup",
            Isa::XCVersionGroup => "XCVersionGroup",
            Isa::PBXFileSystemSynchronizedRootGroup => "PBXFileSystemSynchronizedRootGroup",
            Isa::PBXFileSystemSynchronizedBuildFileExceptionSet => "PBXFileSystemSynchronizedBuildFileExceptionSet",
            Isa::PBXFileSystemSynchronizedGroupBuildPhaseMembershipExceptionSet => {
                "PBXFileSystemSynchronizedGroupBuildPhaseMembershipExceptionSet"
            }
            Isa::PBXNativeTarget => "PBXNativeTarget",
            Isa::PBXAggregateTarget => "PBXAggregateTarget",
            Isa::PBXLegacyTarget => "PBXLegacyTarget",
            Isa::PBXProject => "PBXProject",
            Isa::PBXTargetDependency => "PBXTargetDependency",
            Isa::XCBuildConfiguration => "XCBuildConfiguration",
            Isa::XCConfigurationList => "XCConfigurationList",
            Isa::PBXBuildRule => "PBXBuildRule",
            Isa::PBXReferenceProxy => "PBXReferenceProxy",
            Isa::XCSwiftPackageProductDependency => "XCSwiftPackageProductDependency",
            Isa::XCRemoteSwiftPackageReference => "XCRemoteSwiftPackageReference",
            Isa::XCLocalSwiftPackageReference => "XCLocalSwiftPackageReference",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for Isa {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PBXBuildFile" => Ok(Isa::PBXBuildFile),
            "PBXAppleScriptBuildPhase" => Ok(Isa::PBXAppleScriptBuildPhase),
            "PBXCopyFilesBuildPhase" => Ok(Isa::PBXCopyFilesBuildPhase),
            "PBXFrameworksBuildPhase" => Ok(Isa::PBXFrameworksBuildPhase),
            "PBXHeadersBuildPhase" => Ok(Isa::PBXHeadersBuildPhase),
            "PBXResourcesBuildPhase" => Ok(Isa::PBXResourcesBuildPhase),
            "PBXShellScriptBuildPhase" => Ok(Isa::PBXShellScriptBuildPhase),
            "PBXSourcesBuildPhase" => Ok(Isa::PBXSourcesBuildPhase),
            "PBXRezBuildPhase" => Ok(Isa::PBXRezBuildPhase),
            "PBXContainerItemProxy" => Ok(Isa::PBXContainerItemProxy),
            "PBXFileReference" => Ok(Isa::PBXFileReference),
            "PBXGroup" => Ok(Isa::PBXGroup),
            "PBXVariantGroup" => Ok(Isa::PBXVariantGroup),
            "XCVersionGroup" => Ok(Isa::XCVersionGroup),
            "PBXFileSystemSynchronizedRootGroup" => Ok(Isa::PBXFileSystemSynchronizedRootGroup),
            "PBXFileSystemSynchronizedBuildFileExceptionSet" => Ok(Isa::PBXFileSystemSynchronizedBuildFileExceptionSet),
            "PBXFileSystemSynchronizedGroupBuildPhaseMembershipExceptionSet" => {
                Ok(Isa::PBXFileSystemSynchronizedGroupBuildPhaseMembershipExceptionSet)
            }
            "PBXNativeTarget" => Ok(Isa::PBXNativeTarget),
            "PBXAggregateTarget" => Ok(Isa::PBXAggregateTarget),
            "PBXLegacyTarget" => Ok(Isa::PBXLegacyTarget),
            "PBXProject" => Ok(Isa::PBXProject),
            "PBXTargetDependency" => Ok(Isa::PBXTargetDependency),
            "XCBuildConfiguration" => Ok(Isa::XCBuildConfiguration),
            "XCConfigurationList" => Ok(Isa::XCConfigurationList),
            "PBXBuildRule" => Ok(Isa::PBXBuildRule),
            "PBXReferenceProxy" => Ok(Isa::PBXReferenceProxy),
            "XCSwiftPackageProductDependency" => Ok(Isa::XCSwiftPackageProductDependency),
            "XCRemoteSwiftPackageReference" => Ok(Isa::XCRemoteSwiftPackageReference),
            "XCLocalSwiftPackageReference" => Ok(Isa::XCLocalSwiftPackageReference),
            _ => Err(format!("Unknown ISA: {}", s)),
        }
    }
}

impl Isa {
    /// Returns true if this ISA represents a build phase.
    pub fn is_build_phase(&self) -> bool {
        matches!(
            self,
            Isa::PBXAppleScriptBuildPhase
                | Isa::PBXCopyFilesBuildPhase
                | Isa::PBXFrameworksBuildPhase
                | Isa::PBXHeadersBuildPhase
                | Isa::PBXResourcesBuildPhase
                | Isa::PBXShellScriptBuildPhase
                | Isa::PBXSourcesBuildPhase
                | Isa::PBXRezBuildPhase
        )
    }

    /// Returns true if this ISA represents a target.
    pub fn is_target(&self) -> bool {
        matches!(
            self,
            Isa::PBXNativeTarget | Isa::PBXAggregateTarget | Isa::PBXLegacyTarget
        )
    }

    /// Returns true if this ISA represents a group-like object.
    pub fn is_group(&self) -> bool {
        matches!(
            self,
            Isa::PBXGroup | Isa::PBXVariantGroup | Isa::XCVersionGroup | Isa::PBXFileSystemSynchronizedRootGroup
        )
    }

    /// Extract the default build phase name from the ISA.
    /// e.g., PBXSourcesBuildPhase -> "Sources"
    pub fn default_build_phase_name(&self) -> Option<&'static str> {
        match self {
            Isa::PBXSourcesBuildPhase => Some("Sources"),
            Isa::PBXFrameworksBuildPhase => Some("Frameworks"),
            Isa::PBXResourcesBuildPhase => Some("Resources"),
            Isa::PBXCopyFilesBuildPhase => Some("CopyFiles"),
            Isa::PBXHeadersBuildPhase => Some("Headers"),
            Isa::PBXShellScriptBuildPhase => Some("ShellScript"),
            Isa::PBXAppleScriptBuildPhase => Some("AppleScript"),
            Isa::PBXRezBuildPhase => Some("Rez"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isa_roundtrip() {
        let all = [
            Isa::PBXBuildFile,
            Isa::PBXSourcesBuildPhase,
            Isa::PBXNativeTarget,
            Isa::PBXProject,
            Isa::XCBuildConfiguration,
            Isa::XCRemoteSwiftPackageReference,
            Isa::PBXFileSystemSynchronizedRootGroup,
        ];
        for isa in &all {
            let s = isa.to_string();
            let parsed: Isa = s.parse().unwrap();
            assert_eq!(*isa, parsed);
        }
    }

    #[test]
    fn test_build_phase_name() {
        assert_eq!(Isa::PBXSourcesBuildPhase.default_build_phase_name(), Some("Sources"));
        assert_eq!(Isa::PBXProject.default_build_phase_name(), None);
    }
}
