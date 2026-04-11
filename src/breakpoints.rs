use quick_xml::de::from_str;
use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointActionContent {
    #[serde(rename = "@consoleCommand")]
    pub console_command: Option<String>,
    #[serde(rename = "@message")]
    pub message: Option<String>,
    #[serde(rename = "@conveyanceType")]
    pub conveyance_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointActionProxy {
    #[serde(rename = "@ActionExtensionID")]
    pub action_extension_id: Option<String>,
    #[serde(rename = "ActionContent")]
    pub action_content: Option<BreakpointActionContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actions {
    #[serde(rename = "BreakpointActionProxy", default)]
    pub actions: Vec<BreakpointActionProxy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointContent {
    #[serde(rename = "@uuid")]
    pub uuid: Option<String>,
    #[serde(rename = "@shouldBeEnabled")]
    pub should_be_enabled: Option<String>,
    #[serde(rename = "@ignoreCount")]
    pub ignore_count: Option<String>,
    #[serde(rename = "@continueAfterRunningActions")]
    pub continue_after_running_actions: Option<String>,
    #[serde(rename = "@filePath")]
    pub file_path: Option<String>,
    #[serde(rename = "@startingColumnNumber")]
    pub starting_column_number: Option<String>,
    #[serde(rename = "@endingColumnNumber")]
    pub ending_column_number: Option<String>,
    #[serde(rename = "@startingLineNumber")]
    pub starting_line_number: Option<String>,
    #[serde(rename = "@endingLineNumber")]
    pub ending_line_number: Option<String>,
    #[serde(rename = "@landmarkName")]
    pub landmark_name: Option<String>,
    #[serde(rename = "@landmarkType")]
    pub landmark_type: Option<String>,
    #[serde(rename = "@condition")]
    pub condition: Option<String>,
    #[serde(rename = "@symbolName")]
    pub symbol_name: Option<String>,
    #[serde(rename = "@moduleName")]
    pub module_name: Option<String>,
    #[serde(rename = "@scope")]
    pub scope: Option<String>,
    #[serde(rename = "@stopOnStyle")]
    pub stop_on_style: Option<String>,
    #[serde(rename = "@exceptionType")]
    pub exception_type: Option<String>,
    #[serde(rename = "Actions")]
    pub actions: Option<Actions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointProxy {
    #[serde(rename = "@BreakpointExtensionID")]
    pub breakpoint_extension_id: Option<String>,
    #[serde(rename = "BreakpointContent")]
    pub breakpoint_content: Option<BreakpointContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoints {
    #[serde(rename = "BreakpointProxy", default)]
    pub breakpoint_proxies: Vec<BreakpointProxy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Bucket")]
pub struct BreakpointBucket {
    #[serde(rename = "@uuid")]
    pub uuid: Option<String>,
    #[serde(rename = "@type")]
    pub bucket_type: Option<String>,
    #[serde(rename = "@version")]
    pub version: Option<String>,
    #[serde(rename = "Breakpoints")]
    pub breakpoints: Option<Breakpoints>,
}

impl BreakpointBucket {
    pub fn parse(xml: &str) -> Result<BreakpointBucket, String> {
        from_str(xml).map_err(|e| format!("Failed to parse breakpoints: {}", e))
    }

    pub fn build(&self) -> Result<String, String> {
        let xml = to_string(self).map_err(|e| format!("Failed to build breakpoints: {}", e))?;
        Ok(format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}\n", xml))
    }

    pub fn from_file(path: &str) -> Result<BreakpointBucket, String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;
        Self::parse(&content)
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let xml = self.build()?;
        std::fs::write(path, xml).map_err(|e| format!("Failed to write {}: {}", path, e))
    }

    pub fn list_breakpoints(&self) -> Vec<&BreakpointContent> {
        self.breakpoints
            .as_ref()
            .map(|bp| bp.breakpoint_proxies.iter().filter_map(|p| p.breakpoint_content.as_ref()).collect())
            .unwrap_or_default()
    }

    pub fn add_file_breakpoint(&mut self, uuid: &str, file_path: &str, line: u32) {
        let content = BreakpointContent {
            uuid: Some(uuid.to_string()),
            should_be_enabled: Some("Yes".to_string()),
            ignore_count: Some("0".to_string()),
            continue_after_running_actions: Some("No".to_string()),
            file_path: Some(file_path.to_string()),
            starting_column_number: Some("9223372036854775807".to_string()),
            ending_column_number: Some("9223372036854775807".to_string()),
            starting_line_number: Some(line.to_string()),
            ending_line_number: Some(line.to_string()),
            landmark_name: None,
            landmark_type: None,
            condition: None,
            symbol_name: None,
            module_name: None,
            scope: None,
            stop_on_style: None,
            exception_type: None,
            actions: None,
        };
        let proxy = BreakpointProxy {
            breakpoint_extension_id: Some("Xcode.Breakpoint.FileBreakpoint".to_string()),
            breakpoint_content: Some(content),
        };
        let breakpoints = self.breakpoints.get_or_insert_with(|| Breakpoints { breakpoint_proxies: Vec::new() });
        breakpoints.breakpoint_proxies.push(proxy);
    }

    pub fn remove_breakpoint(&mut self, uuid: &str) -> bool {
        if let Some(ref mut bps) = self.breakpoints {
            let before = bps.breakpoint_proxies.len();
            bps.breakpoint_proxies
                .retain(|p| p.breakpoint_content.as_ref().and_then(|c| c.uuid.as_deref()) != Some(uuid));
            bps.breakpoint_proxies.len() != before
        } else {
            false
        }
    }

    /// Scan a .xcodeproj for breakpoint files in shared and user data.
    pub fn find_breakpoint_files(xcodeproj_path: &str) -> Vec<String> {
        let mut files = Vec::new();
        let shared = format!("{}/xcshareddata/xcdebugger/Breakpoints_v2.xcbkptlist", xcodeproj_path);
        if std::path::Path::new(&shared).exists() {
            files.push(shared);
        }
        let userdata_dir = format!("{}/xcuserdata", xcodeproj_path);
        if let Ok(users) = std::fs::read_dir(&userdata_dir) {
            for user_entry in users.flatten() {
                let bp_path = user_entry.path().join("xcdebugger/Breakpoints_v2.xcbkptlist");
                if bp_path.exists() {
                    files.push(bp_path.to_string_lossy().to_string());
                }
            }
        }
        files
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/breakpoints/Breakpoints_v2.xcbkptlist");

    #[test]
    fn parse_fixture() {
        let bucket = BreakpointBucket::from_file(FIXTURE).unwrap();
        assert_eq!(bucket.list_breakpoints().len(), 4);
    }

    #[test]
    fn file_breakpoint_fields() {
        let bucket = BreakpointBucket::from_file(FIXTURE).unwrap();
        let bps = bucket.list_breakpoints();
        let first = &bps[0];
        assert_eq!(first.file_path.as_deref(), Some("MyApp/ViewController.swift"));
        assert_eq!(first.starting_line_number.as_deref(), Some("42"));
        assert_eq!(first.should_be_enabled.as_deref(), Some("Yes"));
    }

    #[test]
    fn breakpoint_with_condition() {
        let bucket = BreakpointBucket::from_file(FIXTURE).unwrap();
        let bps = bucket.list_breakpoints();
        let conditioned = &bps[1];
        assert_eq!(conditioned.condition.as_deref(), Some("count > 10"));
        assert_eq!(conditioned.should_be_enabled.as_deref(), Some("No"));
    }

    #[test]
    fn breakpoint_with_actions() {
        let bucket = BreakpointBucket::from_file(FIXTURE).unwrap();
        let bps = bucket.list_breakpoints();
        let with_actions = &bps[1];
        let actions = &with_actions.actions.as_ref().unwrap().actions;
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].action_content.as_ref().unwrap().console_command.as_deref(), Some("po self"));
    }

    #[test]
    fn symbolic_breakpoint() {
        let bucket = BreakpointBucket::from_file(FIXTURE).unwrap();
        let bps = bucket.list_breakpoints();
        let symbolic = &bps[2];
        assert_eq!(symbolic.symbol_name.as_deref(), Some("objc_exception_throw"));
        assert!(symbolic.file_path.is_none());
    }

    #[test]
    fn exception_breakpoint() {
        let bucket = BreakpointBucket::from_file(FIXTURE).unwrap();
        let bps = bucket.list_breakpoints();
        let exception = &bps[3];
        assert_eq!(exception.scope.as_deref(), Some("0"));
        assert_eq!(exception.exception_type.as_deref(), Some("0"));
    }

    #[test]
    fn add_and_remove() {
        let mut bucket = BreakpointBucket::from_file(FIXTURE).unwrap();
        let before = bucket.list_breakpoints().len();
        bucket.add_file_breakpoint("TEST-UUID-1234", "Test.swift", 10);
        assert_eq!(bucket.list_breakpoints().len(), before + 1);
        assert!(bucket.remove_breakpoint("TEST-UUID-1234"));
        assert_eq!(bucket.list_breakpoints().len(), before);
    }

    #[test]
    fn remove_nonexistent_returns_false() {
        let mut bucket = BreakpointBucket::from_file(FIXTURE).unwrap();
        assert!(!bucket.remove_breakpoint("DOES-NOT-EXIST"));
    }

    #[test]
    fn round_trip() {
        let original = BreakpointBucket::from_file(FIXTURE).unwrap();
        let xml = original.build().unwrap();
        let reparsed = BreakpointBucket::parse(&xml).unwrap();
        assert_eq!(original.list_breakpoints().len(), reparsed.list_breakpoints().len());
    }
}
