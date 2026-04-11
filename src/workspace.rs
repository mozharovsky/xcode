use quick_xml::de::from_str;
use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRef {
    #[serde(rename = "@location")]
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    #[serde(rename = "@location")]
    pub location: Option<String>,
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "$value", default)]
    pub children: Vec<WorkspaceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkspaceItem {
    FileRef(FileRef),
    Group(Group),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Workspace")]
pub struct Workspace {
    #[serde(rename = "@version")]
    pub version: Option<String>,
    #[serde(rename = "$value", default)]
    pub items: Vec<WorkspaceItem>,
}

impl Workspace {
    pub fn parse(xml: &str) -> Result<Workspace, String> {
        from_str(xml).map_err(|e| format!("Failed to parse workspace: {}", e))
    }

    pub fn build(&self) -> Result<String, String> {
        let xml = to_string(self).map_err(|e| format!("Failed to build workspace: {}", e))?;
        Ok(format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}\n", xml))
    }

    pub fn from_file(path: &str) -> Result<Workspace, String> {
        let data_path = if path.ends_with(".xcworkspacedata") {
            path.to_string()
        } else {
            format!("{}/contents.xcworkspacedata", path.trim_end_matches('/'))
        };
        let content =
            std::fs::read_to_string(&data_path).map_err(|e| format!("Failed to read {}: {}", data_path, e))?;
        Self::parse(&content)
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let data_path = if path.ends_with(".xcworkspacedata") {
            path.to_string()
        } else {
            format!("{}/contents.xcworkspacedata", path.trim_end_matches('/'))
        };
        let xml = self.build()?;
        if let Some(parent) = std::path::Path::new(&data_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        std::fs::write(&data_path, xml).map_err(|e| format!("Failed to write {}: {}", data_path, e))
    }

    pub fn get_project_paths(&self) -> Vec<String> {
        fn collect(items: &[WorkspaceItem], out: &mut Vec<String>) {
            for item in items {
                match item {
                    WorkspaceItem::FileRef(f) => out.push(f.location.clone()),
                    WorkspaceItem::Group(g) => collect(&g.children, out),
                }
            }
        }
        let mut paths = Vec::new();
        collect(&self.items, &mut paths);
        paths
    }

    pub fn add_project(&mut self, location: &str) {
        self.items.push(WorkspaceItem::FileRef(FileRef { location: location.to_string() }));
    }

    pub fn remove_project(&mut self, location: &str) -> bool {
        let before = self.items.len();
        self.items.retain(|item| match item {
            WorkspaceItem::FileRef(f) => f.location != location,
            _ => true,
        });
        self.items.len() != before
    }

    pub fn create_empty() -> Workspace {
        Workspace { version: Some("1.0".to_string()), items: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/workspaces");

    fn load(name: &str) -> Workspace {
        Workspace::parse(&std::fs::read_to_string(format!("{}/{}", FIXTURES, name)).unwrap()).unwrap()
    }

    #[test]
    fn parse_simple() {
        let ws = load("simple.xcworkspacedata");
        assert_eq!(ws.get_project_paths(), vec!["group:App.xcodeproj"]);
    }

    #[test]
    fn parse_cocoapods() {
        let ws = load("cocoapods.xcworkspacedata");
        let paths = ws.get_project_paths();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"group:App.xcodeproj".to_string()));
        assert!(paths.contains(&"group:Pods/Pods.xcodeproj".to_string()));
    }

    #[test]
    fn parse_all_location_types() {
        let ws = load("all-location-types.xcworkspacedata");
        assert!(ws.get_project_paths().len() >= 3);
    }

    #[test]
    fn parse_with_groups() {
        let ws = load("with-groups.xcworkspacedata");
        assert!(!ws.items.is_empty());
    }

    #[test]
    fn parse_self_reference() {
        let ws = load("self-reference.xcworkspacedata");
        let paths = ws.get_project_paths();
        assert!(paths.iter().any(|p| p.starts_with("self:")));
    }

    #[test]
    fn add_and_remove_project() {
        let mut ws = Workspace::create_empty();
        ws.add_project("group:New.xcodeproj");
        assert_eq!(ws.get_project_paths().len(), 1);
        assert!(ws.remove_project("group:New.xcodeproj"));
        assert_eq!(ws.get_project_paths().len(), 0);
    }

    #[test]
    fn remove_nonexistent_returns_false() {
        let mut ws = Workspace::create_empty();
        assert!(!ws.remove_project("group:DoesNotExist.xcodeproj"));
    }

    #[test]
    fn round_trip() {
        let original = load("cocoapods.xcworkspacedata");
        let xml = original.build().unwrap();
        let reparsed = Workspace::parse(&xml).unwrap();
        assert_eq!(original.get_project_paths(), reparsed.get_project_paths());
    }
}
