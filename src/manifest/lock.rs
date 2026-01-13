use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Lock file (shot.lock)
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LockFile {
    #[serde(default, rename = "package")]
    pub packages: Vec<LockedPackage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
    pub source: String,
}

impl LockFile {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize lock file: {}", e))?;

        fs::write(path, content)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
    }

    pub fn add_or_update(&mut self, name: &str, version: &str, source: &str) {
        // Remove existing entry if present
        self.packages.retain(|p| p.name != name);

        // Add new entry
        self.packages.push(LockedPackage {
            name: name.to_string(),
            version: version.to_string(),
            source: source.to_string(),
        });

        // Sort by name for consistent output
        self.packages.sort_by(|a, b| a.name.cmp(&b.name));
    }

    pub fn find(&self, name: &str) -> Option<&LockedPackage> {
        self.packages.iter().find(|p| p.name == name)
    }
}
