use quick_xml::de::from_str;
use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};

// ── Core types ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildableReference {
    #[serde(rename = "@BuildableIdentifier")]
    pub buildable_identifier: Option<String>,
    #[serde(rename = "@BlueprintIdentifier")]
    pub blueprint_identifier: Option<String>,
    #[serde(rename = "@BuildableName")]
    pub buildable_name: Option<String>,
    #[serde(rename = "@BlueprintName")]
    pub blueprint_name: Option<String>,
    #[serde(rename = "@ReferencedContainer")]
    pub referenced_container: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentVariable {
    #[serde(rename = "@key")]
    pub key: String,
    #[serde(rename = "@value")]
    pub value: String,
    #[serde(rename = "@isEnabled")]
    pub is_enabled: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandLineArgument {
    #[serde(rename = "@argument")]
    pub argument: String,
    #[serde(rename = "@isEnabled")]
    pub is_enabled: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentVariables {
    #[serde(rename = "EnvironmentVariable", default)]
    pub variables: Vec<EnvironmentVariable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandLineArguments {
    #[serde(rename = "CommandLineArgument", default)]
    pub arguments: Vec<CommandLineArgument>,
}

// ── Execution actions (pre/post build scripts) ───────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionContent {
    #[serde(rename = "@title")]
    pub title: Option<String>,
    #[serde(rename = "@scriptText")]
    pub script_text: Option<String>,
    #[serde(rename = "@shellToInvoke")]
    pub shell_to_invoke: Option<String>,
    #[serde(rename = "EnvironmentBuildable")]
    pub environment_buildable: Option<EnvironmentBuildable>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentBuildable {
    #[serde(rename = "BuildableReference")]
    pub buildable_reference: Option<BuildableReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionAction {
    #[serde(rename = "@ActionType")]
    pub action_type: Option<String>,
    #[serde(rename = "ActionContent")]
    pub action_content: Option<ActionContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreActions {
    #[serde(rename = "ExecutionAction", default)]
    pub actions: Vec<ExecutionAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostActions {
    #[serde(rename = "ExecutionAction", default)]
    pub actions: Vec<ExecutionAction>,
}

// ── Build action ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildActionEntry {
    #[serde(rename = "@buildForTesting")]
    pub build_for_testing: Option<String>,
    #[serde(rename = "@buildForRunning")]
    pub build_for_running: Option<String>,
    #[serde(rename = "@buildForProfiling")]
    pub build_for_profiling: Option<String>,
    #[serde(rename = "@buildForArchiving")]
    pub build_for_archiving: Option<String>,
    #[serde(rename = "@buildForAnalyzing")]
    pub build_for_analyzing: Option<String>,
    #[serde(rename = "BuildableReference")]
    pub buildable_reference: Option<BuildableReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildActionEntries {
    #[serde(rename = "BuildActionEntry", default)]
    pub entries: Vec<BuildActionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildAction {
    #[serde(rename = "@parallelizeBuildables")]
    pub parallelize_buildables: Option<String>,
    #[serde(rename = "@buildImplicitDependencies")]
    pub build_implicit_dependencies: Option<String>,
    #[serde(rename = "@buildArchitectures")]
    pub build_architectures: Option<String>,
    #[serde(rename = "@runPostActionsOnFailure")]
    pub run_post_actions_on_failure: Option<String>,
    #[serde(rename = "PreActions")]
    pub pre_actions: Option<PreActions>,
    #[serde(rename = "PostActions")]
    pub post_actions: Option<PostActions>,
    #[serde(rename = "BuildActionEntries")]
    pub build_action_entries: Option<BuildActionEntries>,
}

// ── Test action ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestableReference {
    #[serde(rename = "@skipped")]
    pub skipped: Option<String>,
    #[serde(rename = "@useTestSelectionWhitelist")]
    pub use_test_selection_whitelist: Option<String>,
    #[serde(rename = "BuildableReference")]
    pub buildable_reference: Option<BuildableReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Testables {
    #[serde(rename = "TestableReference", default)]
    pub testable_references: Vec<TestableReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPlanReference {
    #[serde(rename = "@default")]
    pub default: Option<String>,
    #[serde(rename = "@reference")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPlans {
    #[serde(rename = "TestPlanReference", default)]
    pub test_plan_references: Vec<TestPlanReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroExpansion {
    #[serde(rename = "BuildableReference")]
    pub buildable_reference: Option<BuildableReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestAction {
    #[serde(rename = "@buildConfiguration")]
    pub build_configuration: Option<String>,
    #[serde(rename = "@selectedDebuggerIdentifier")]
    pub selected_debugger_identifier: Option<String>,
    #[serde(rename = "@selectedLauncherIdentifier")]
    pub selected_launcher_identifier: Option<String>,
    #[serde(rename = "@shouldUseLaunchSchemeArgsEnv")]
    pub should_use_launch_scheme_args_env: Option<String>,
    #[serde(rename = "@codeCoverageEnabled")]
    pub code_coverage_enabled: Option<String>,
    #[serde(rename = "@onlyGenerateCoverageForSpecifiedTargets")]
    pub only_generate_coverage_for_specified_targets: Option<String>,
    #[serde(rename = "@shouldAutocreateTestPlan")]
    pub should_autocreate_test_plan: Option<String>,
    #[serde(rename = "@preferredScreenCaptureFormat")]
    pub preferred_screen_capture_format: Option<String>,
    #[serde(rename = "PreActions")]
    pub pre_actions: Option<PreActions>,
    #[serde(rename = "PostActions")]
    pub post_actions: Option<PostActions>,
    #[serde(rename = "Testables")]
    pub testables: Option<Testables>,
    #[serde(rename = "MacroExpansion")]
    pub macro_expansion: Option<MacroExpansion>,
    #[serde(rename = "TestPlans")]
    pub test_plans: Option<TestPlans>,
    #[serde(rename = "CommandLineArguments")]
    pub command_line_arguments: Option<CommandLineArguments>,
    #[serde(rename = "EnvironmentVariables")]
    pub environment_variables: Option<EnvironmentVariables>,
}

// ── Launch action ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildableProductRunnable {
    #[serde(rename = "@runnableDebuggingMode")]
    pub runnable_debugging_mode: Option<String>,
    #[serde(rename = "BuildableReference")]
    pub buildable_reference: Option<BuildableReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteRunnable {
    #[serde(rename = "@runnableDebuggingMode")]
    pub runnable_debugging_mode: Option<String>,
    #[serde(rename = "@BundleIdentifier")]
    pub bundle_identifier: Option<String>,
    #[serde(rename = "@RemotePath")]
    pub remote_path: Option<String>,
    #[serde(rename = "BuildableReference")]
    pub buildable_reference: Option<BuildableReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationScenarioReference {
    #[serde(rename = "@identifier")]
    pub identifier: Option<String>,
    #[serde(rename = "@referenceType")]
    pub reference_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreKitConfigurationFileReference {
    #[serde(rename = "@identifier")]
    pub identifier: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchAction {
    #[serde(rename = "@buildConfiguration")]
    pub build_configuration: Option<String>,
    #[serde(rename = "@selectedDebuggerIdentifier")]
    pub selected_debugger_identifier: Option<String>,
    #[serde(rename = "@selectedLauncherIdentifier")]
    pub selected_launcher_identifier: Option<String>,
    #[serde(rename = "@launchStyle")]
    pub launch_style: Option<String>,
    #[serde(rename = "@useCustomWorkingDirectory")]
    pub use_custom_working_directory: Option<String>,
    #[serde(rename = "@customWorkingDirectory")]
    pub custom_working_directory: Option<String>,
    #[serde(rename = "@ignoresPersistentStateOnLaunch")]
    pub ignores_persistent_state_on_launch: Option<String>,
    #[serde(rename = "@debugDocumentVersioning")]
    pub debug_document_versioning: Option<String>,
    #[serde(rename = "@debugServiceExtension")]
    pub debug_service_extension: Option<String>,
    #[serde(rename = "@allowLocationSimulation")]
    pub allow_location_simulation: Option<String>,
    #[serde(rename = "@askForAppToLaunch")]
    pub ask_for_app_to_launch: Option<String>,
    #[serde(rename = "@launchAutomaticallySubstyle")]
    pub launch_automatically_substyle: Option<String>,
    #[serde(rename = "@customLaunchCommand")]
    pub custom_launch_command: Option<String>,
    #[serde(rename = "@appClipInvocationURLString")]
    pub app_clip_invocation_url_string: Option<String>,
    #[serde(rename = "PreActions")]
    pub pre_actions: Option<PreActions>,
    #[serde(rename = "PostActions")]
    pub post_actions: Option<PostActions>,
    #[serde(rename = "BuildableProductRunnable")]
    pub buildable_product_runnable: Option<BuildableProductRunnable>,
    #[serde(rename = "RemoteRunnable")]
    pub remote_runnable: Option<RemoteRunnable>,
    #[serde(rename = "MacroExpansion")]
    pub macro_expansion: Option<MacroExpansion>,
    #[serde(rename = "LocationScenarioReference")]
    pub location_scenario_reference: Option<LocationScenarioReference>,
    #[serde(rename = "CommandLineArguments")]
    pub command_line_arguments: Option<CommandLineArguments>,
    #[serde(rename = "EnvironmentVariables")]
    pub environment_variables: Option<EnvironmentVariables>,
    #[serde(rename = "StoreKitConfigurationFileReference")]
    pub storekit_configuration_file_reference: Option<StoreKitConfigurationFileReference>,
}

// ── Profile / Analyze / Archive actions ──────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileAction {
    #[serde(rename = "@buildConfiguration")]
    pub build_configuration: Option<String>,
    #[serde(rename = "@shouldUseLaunchSchemeArgsEnv")]
    pub should_use_launch_scheme_args_env: Option<String>,
    #[serde(rename = "@savedToolIdentifier")]
    pub saved_tool_identifier: Option<String>,
    #[serde(rename = "@useCustomWorkingDirectory")]
    pub use_custom_working_directory: Option<String>,
    #[serde(rename = "@debugDocumentVersioning")]
    pub debug_document_versioning: Option<String>,
    #[serde(rename = "@askForAppToLaunch")]
    pub ask_for_app_to_launch: Option<String>,
    #[serde(rename = "@launchAutomaticallySubstyle")]
    pub launch_automatically_substyle: Option<String>,
    #[serde(rename = "@appClipInvocationURLString")]
    pub app_clip_invocation_url_string: Option<String>,
    #[serde(rename = "PostActions")]
    pub post_actions: Option<PostActions>,
    #[serde(rename = "BuildableProductRunnable")]
    pub buildable_product_runnable: Option<BuildableProductRunnable>,
    #[serde(rename = "RemoteRunnable")]
    pub remote_runnable: Option<RemoteRunnable>,
    #[serde(rename = "MacroExpansion")]
    pub macro_expansion: Option<MacroExpansion>,
    #[serde(rename = "CommandLineArguments")]
    pub command_line_arguments: Option<CommandLineArguments>,
    #[serde(rename = "EnvironmentVariables")]
    pub environment_variables: Option<EnvironmentVariables>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeAction {
    #[serde(rename = "@buildConfiguration")]
    pub build_configuration: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveAction {
    #[serde(rename = "@buildConfiguration")]
    pub build_configuration: Option<String>,
    #[serde(rename = "@customArchiveName")]
    pub custom_archive_name: Option<String>,
    #[serde(rename = "@revealArchiveInOrganizer")]
    pub reveal_archive_in_organizer: Option<String>,
}

// ── Top-level Scheme ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Scheme")]
pub struct Scheme {
    #[serde(rename = "@LastUpgradeVersion")]
    pub last_upgrade_version: Option<String>,
    #[serde(rename = "@version")]
    pub version: Option<String>,
    #[serde(rename = "BuildAction")]
    pub build_action: Option<BuildAction>,
    #[serde(rename = "TestAction")]
    pub test_action: Option<TestAction>,
    #[serde(rename = "LaunchAction")]
    pub launch_action: Option<LaunchAction>,
    #[serde(rename = "ProfileAction")]
    pub profile_action: Option<ProfileAction>,
    #[serde(rename = "AnalyzeAction")]
    pub analyze_action: Option<AnalyzeAction>,
    #[serde(rename = "ArchiveAction")]
    pub archive_action: Option<ArchiveAction>,
}

// ── API ──────────────────────────────────────────────────────────

impl Scheme {
    pub fn parse(xml: &str) -> Result<Scheme, String> {
        from_str(xml).map_err(|e| format!("Failed to parse xcscheme: {}", e))
    }

    pub fn build(&self) -> Result<String, String> {
        let xml = to_string(self).map_err(|e| format!("Failed to build xcscheme: {}", e))?;
        Ok(format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}\n", xml))
    }

    pub fn from_file(path: &str) -> Result<Scheme, String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;
        Self::parse(&content)
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let xml = self.build()?;
        std::fs::write(path, xml).map_err(|e| format!("Failed to write {}: {}", path, e))
    }

    /// List scheme files inside a .xcodeproj directory.
    pub fn list_schemes(xcodeproj_path: &str) -> Vec<(String, String)> {
        let mut schemes = Vec::new();
        let shared_dir = format!("{}/xcshareddata/xcschemes", xcodeproj_path);
        if let Ok(entries) = std::fs::read_dir(&shared_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("xcscheme") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        schemes.push((name.to_string(), path.to_string_lossy().to_string()));
                    }
                }
            }
        }
        let userdata_dir = format!("{}/xcuserdata", xcodeproj_path);
        if let Ok(users) = std::fs::read_dir(&userdata_dir) {
            for user_entry in users.flatten() {
                let user_schemes = user_entry.path().join("xcschemes");
                if let Ok(entries) = std::fs::read_dir(&user_schemes) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|e| e.to_str()) == Some("xcscheme") {
                            if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                schemes.push((name.to_string(), path.to_string_lossy().to_string()));
                            }
                        }
                    }
                }
            }
        }
        schemes
    }

    pub fn get_build_targets(&self) -> Vec<&BuildableReference> {
        self.build_action
            .as_ref()
            .and_then(|ba| ba.build_action_entries.as_ref())
            .map(|entries| entries.entries.iter().filter_map(|e| e.buildable_reference.as_ref()).collect())
            .unwrap_or_default()
    }

    pub fn add_build_target(
        &mut self,
        blueprint_id: &str,
        buildable_name: &str,
        blueprint_name: &str,
        container: &str,
    ) {
        let build_ref = BuildableReference {
            buildable_identifier: Some("primary".to_string()),
            blueprint_identifier: Some(blueprint_id.to_string()),
            buildable_name: Some(buildable_name.to_string()),
            blueprint_name: Some(blueprint_name.to_string()),
            referenced_container: Some(container.to_string()),
        };
        let entry = BuildActionEntry {
            build_for_testing: Some("YES".to_string()),
            build_for_running: Some("YES".to_string()),
            build_for_profiling: Some("YES".to_string()),
            build_for_archiving: Some("YES".to_string()),
            build_for_analyzing: Some("YES".to_string()),
            buildable_reference: Some(build_ref),
        };
        let build_action = self.build_action.get_or_insert_with(|| BuildAction {
            parallelize_buildables: Some("YES".to_string()),
            build_implicit_dependencies: Some("YES".to_string()),
            build_architectures: None,
            run_post_actions_on_failure: None,
            pre_actions: None,
            post_actions: None,
            build_action_entries: None,
        });
        let entries =
            build_action.build_action_entries.get_or_insert_with(|| BuildActionEntries { entries: Vec::new() });
        entries.entries.push(entry);
    }

    pub fn set_env_var(&mut self, key: &str, value: &str, is_enabled: bool) {
        let env_var = EnvironmentVariable {
            key: key.to_string(),
            value: value.to_string(),
            is_enabled: Some(if is_enabled { "YES" } else { "NO" }.to_string()),
        };
        let launch = self.launch_action.get_or_insert_with(|| LaunchAction {
            build_configuration: Some("Debug".to_string()),
            selected_debugger_identifier: None,
            selected_launcher_identifier: None,
            launch_style: None,
            use_custom_working_directory: None,
            custom_working_directory: None,
            ignores_persistent_state_on_launch: None,
            debug_document_versioning: None,
            debug_service_extension: None,
            allow_location_simulation: None,
            ask_for_app_to_launch: None,
            launch_automatically_substyle: None,
            custom_launch_command: None,
            app_clip_invocation_url_string: None,
            pre_actions: None,
            post_actions: None,
            buildable_product_runnable: None,
            remote_runnable: None,
            macro_expansion: None,
            location_scenario_reference: None,
            command_line_arguments: None,
            environment_variables: None,
            storekit_configuration_file_reference: None,
        });
        let env_vars =
            launch.environment_variables.get_or_insert_with(|| EnvironmentVariables { variables: Vec::new() });
        if let Some(existing) = env_vars.variables.iter_mut().find(|v| v.key == key) {
            existing.value = value.to_string();
            existing.is_enabled = Some(if is_enabled { "YES" } else { "NO" }.to_string());
        } else {
            env_vars.variables.push(env_var);
        }
    }

    pub fn add_launch_arg(&mut self, arg: &str) {
        let cli_arg = CommandLineArgument { argument: arg.to_string(), is_enabled: Some("YES".to_string()) };
        let launch = self.launch_action.get_or_insert_with(|| LaunchAction {
            build_configuration: Some("Debug".to_string()),
            selected_debugger_identifier: None,
            selected_launcher_identifier: None,
            launch_style: None,
            use_custom_working_directory: None,
            custom_working_directory: None,
            ignores_persistent_state_on_launch: None,
            debug_document_versioning: None,
            debug_service_extension: None,
            allow_location_simulation: None,
            ask_for_app_to_launch: None,
            launch_automatically_substyle: None,
            custom_launch_command: None,
            app_clip_invocation_url_string: None,
            pre_actions: None,
            post_actions: None,
            buildable_product_runnable: None,
            remote_runnable: None,
            macro_expansion: None,
            location_scenario_reference: None,
            command_line_arguments: None,
            environment_variables: None,
            storekit_configuration_file_reference: None,
        });
        let args = launch.command_line_arguments.get_or_insert_with(|| CommandLineArguments { arguments: Vec::new() });
        args.arguments.push(cli_arg);
    }

    /// Create a minimal scheme for a target.
    pub fn create_for_target(name: &str, blueprint_id: &str, product_name: &str, container: &str) -> Scheme {
        let build_ref = BuildableReference {
            buildable_identifier: Some("primary".to_string()),
            blueprint_identifier: Some(blueprint_id.to_string()),
            buildable_name: Some(product_name.to_string()),
            blueprint_name: Some(name.to_string()),
            referenced_container: Some(container.to_string()),
        };
        let runnable = BuildableProductRunnable {
            runnable_debugging_mode: Some("0".to_string()),
            buildable_reference: Some(build_ref.clone()),
        };
        Scheme {
            last_upgrade_version: Some("1600".to_string()),
            version: Some("1.7".to_string()),
            build_action: Some(BuildAction {
                parallelize_buildables: Some("YES".to_string()),
                build_implicit_dependencies: Some("YES".to_string()),
                build_architectures: None,
                run_post_actions_on_failure: None,
                pre_actions: None,
                post_actions: None,
                build_action_entries: Some(BuildActionEntries {
                    entries: vec![BuildActionEntry {
                        build_for_testing: Some("YES".to_string()),
                        build_for_running: Some("YES".to_string()),
                        build_for_profiling: Some("YES".to_string()),
                        build_for_archiving: Some("YES".to_string()),
                        build_for_analyzing: Some("YES".to_string()),
                        buildable_reference: Some(build_ref.clone()),
                    }],
                }),
            }),
            test_action: Some(TestAction {
                build_configuration: Some("Debug".to_string()),
                selected_debugger_identifier: Some("Xcode.DebuggerFoundation.Debugger.LLDB".to_string()),
                selected_launcher_identifier: Some("Xcode.DebuggerFoundation.Launcher.LLDB".to_string()),
                should_use_launch_scheme_args_env: Some("YES".to_string()),
                code_coverage_enabled: None,
                only_generate_coverage_for_specified_targets: None,
                should_autocreate_test_plan: Some("YES".to_string()),
                preferred_screen_capture_format: None,
                pre_actions: None,
                post_actions: None,
                testables: None,
                macro_expansion: None,
                test_plans: None,
                command_line_arguments: None,
                environment_variables: None,
            }),
            launch_action: Some(LaunchAction {
                build_configuration: Some("Debug".to_string()),
                selected_debugger_identifier: Some("Xcode.DebuggerFoundation.Debugger.LLDB".to_string()),
                selected_launcher_identifier: Some("Xcode.DebuggerFoundation.Launcher.LLDB".to_string()),
                launch_style: Some("0".to_string()),
                use_custom_working_directory: Some("NO".to_string()),
                custom_working_directory: None,
                ignores_persistent_state_on_launch: Some("NO".to_string()),
                debug_document_versioning: Some("YES".to_string()),
                debug_service_extension: Some("internal".to_string()),
                allow_location_simulation: Some("YES".to_string()),
                ask_for_app_to_launch: None,
                launch_automatically_substyle: None,
                custom_launch_command: None,
                app_clip_invocation_url_string: None,
                pre_actions: None,
                post_actions: None,
                buildable_product_runnable: Some(runnable.clone()),
                remote_runnable: None,
                macro_expansion: None,
                location_scenario_reference: None,
                command_line_arguments: None,
                environment_variables: None,
                storekit_configuration_file_reference: None,
            }),
            profile_action: Some(ProfileAction {
                build_configuration: Some("Release".to_string()),
                should_use_launch_scheme_args_env: Some("YES".to_string()),
                saved_tool_identifier: Some(String::new()),
                use_custom_working_directory: Some("NO".to_string()),
                debug_document_versioning: Some("YES".to_string()),
                ask_for_app_to_launch: None,
                launch_automatically_substyle: None,
                app_clip_invocation_url_string: None,
                post_actions: None,
                buildable_product_runnable: Some(runnable),
                remote_runnable: None,
                macro_expansion: None,
                command_line_arguments: None,
                environment_variables: None,
            }),
            analyze_action: Some(AnalyzeAction { build_configuration: Some("Debug".to_string()) }),
            archive_action: Some(ArchiveAction {
                build_configuration: Some("Release".to_string()),
                custom_archive_name: None,
                reveal_archive_in_organizer: Some("YES".to_string()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/schemes");

    fn load(name: &str) -> Scheme {
        Scheme::from_file(&format!("{}/{}", FIXTURES, name)).unwrap()
    }

    #[test]
    fn parse_all_fixtures() {
        let files = [
            "iOS.xcscheme",
            "MinimalInformation.xcscheme",
            "AppClip.xcscheme",
            "WatchApp.xcscheme",
            "BuildArchitectures.xcscheme",
            "RunPostActionsOnFailure.xcscheme",
            "RunnableWithoutBuildableReference.xcscheme",
            "NoBlueprintID.xcscheme",
        ];
        for f in &files {
            let scheme = load(f);
            assert!(scheme.version.is_some(), "missing version in {}", f);
        }
    }

    #[test]
    fn ios_build_targets() {
        let scheme = load("iOS.xcscheme");
        let targets = scheme.get_build_targets();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].blueprint_name.as_deref(), Some("iOS"));
        assert_eq!(targets[0].buildable_name.as_deref(), Some("iOS.app"));
    }

    #[test]
    fn ios_launch_env_vars() {
        let scheme = load("iOS.xcscheme");
        let env = &scheme.launch_action.unwrap().environment_variables.unwrap().variables;
        assert_eq!(env.len(), 1);
        assert_eq!(env[0].key, "ENV_VAR");
        assert_eq!(env[0].value, "RUN");
    }

    #[test]
    fn ios_test_action_coverage() {
        let scheme = load("iOS.xcscheme");
        let test = scheme.test_action.unwrap();
        assert_eq!(test.code_coverage_enabled.as_deref(), Some("YES"));
    }

    #[test]
    fn ios_pre_post_actions() {
        let scheme = load("iOS.xcscheme");
        let build = scheme.build_action.unwrap();
        assert_eq!(build.pre_actions.unwrap().actions.len(), 1);
        assert_eq!(build.post_actions.unwrap().actions.len(), 1);
    }

    #[test]
    fn watch_app_multiple_targets() {
        let scheme = load("WatchApp.xcscheme");
        assert_eq!(scheme.get_build_targets().len(), 2);
    }

    #[test]
    fn watch_app_remote_runnable() {
        let scheme = load("WatchApp.xcscheme");
        let remote = scheme.launch_action.unwrap().remote_runnable.unwrap();
        assert_eq!(remote.bundle_identifier.as_deref(), Some("com.apple.Carousel"));
    }

    #[test]
    fn app_clip_invocation_url() {
        let scheme = load("AppClip.xcscheme");
        let url = scheme.launch_action.unwrap().app_clip_invocation_url_string;
        assert_eq!(url.as_deref(), Some("https://example.com/"));
    }

    #[test]
    fn no_blueprint_id() {
        let scheme = load("NoBlueprintID.xcscheme");
        let targets = scheme.get_build_targets();
        assert_eq!(targets.len(), 1);
        assert!(targets[0].blueprint_identifier.is_none());
    }

    #[test]
    fn minimal_scheme_has_env_var() {
        let scheme = load("MinimalInformation.xcscheme");
        let env = scheme.launch_action.unwrap().environment_variables.unwrap().variables;
        assert_eq!(env[0].key, "AI_TEST_MODE");
    }

    #[test]
    fn run_post_actions_on_failure() {
        let scheme = load("RunPostActionsOnFailure.xcscheme");
        assert_eq!(scheme.build_action.unwrap().run_post_actions_on_failure.as_deref(), Some("YES"));
    }

    #[test]
    fn build_architectures() {
        let scheme = load("BuildArchitectures.xcscheme");
        assert_eq!(scheme.build_action.unwrap().build_architectures.as_deref(), Some("Automatic"));
    }

    #[test]
    fn archive_custom_name() {
        let scheme = load("iOS.xcscheme");
        assert_eq!(scheme.archive_action.unwrap().custom_archive_name.as_deref(), Some("TestName"));
    }

    #[test]
    fn add_build_target_mutation() {
        let mut scheme = load("MinimalInformation.xcscheme");
        let before = scheme.get_build_targets().len();
        scheme.add_build_target("NEW123", "New.app", "NewTarget", "container:App.xcodeproj");
        assert_eq!(scheme.get_build_targets().len(), before + 1);
    }

    #[test]
    fn set_env_var_mutation() {
        let mut scheme = load("MinimalInformation.xcscheme");
        scheme.set_env_var("MY_KEY", "MY_VALUE", true);
        let env = &scheme.launch_action.unwrap().environment_variables.unwrap().variables;
        assert!(env.iter().any(|v| v.key == "MY_KEY" && v.value == "MY_VALUE"));
    }

    #[test]
    fn add_launch_arg_mutation() {
        let mut scheme = load("MinimalInformation.xcscheme");
        scheme.add_launch_arg("-verbose");
        let args = &scheme.launch_action.unwrap().command_line_arguments.unwrap().arguments;
        assert!(args.iter().any(|a| a.argument == "-verbose"));
    }

    #[test]
    fn create_for_target() {
        let scheme = Scheme::create_for_target("MyApp", "ABC123", "MyApp.app", "container:App.xcodeproj");
        assert_eq!(scheme.get_build_targets().len(), 1);
        assert_eq!(scheme.launch_action.as_ref().unwrap().build_configuration.as_deref(), Some("Debug"));
        assert_eq!(scheme.archive_action.as_ref().unwrap().build_configuration.as_deref(), Some("Release"));
    }

    #[test]
    fn round_trip_parse_build() {
        let original = load("iOS.xcscheme");
        let xml = original.build().unwrap();
        let reparsed = Scheme::parse(&xml).unwrap();
        assert_eq!(original.get_build_targets().len(), reparsed.get_build_targets().len());
        assert_eq!(
            original.launch_action.as_ref().unwrap().build_configuration,
            reparsed.launch_action.as_ref().unwrap().build_configuration
        );
    }
}
