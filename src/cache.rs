use std::fs;
use std::path::{Path, PathBuf};

/// Global cache at ~/.shot/cache/
pub struct Cache {
    root: PathBuf,
}

impl Cache {
    pub fn new() -> Result<Self, String> {
        let home = std::env::var("HOME").map_err(|_| "HOME not set")?;
        let root = PathBuf::from(home).join(".shot/cache");
        Ok(Self { root })
    }

    /// Get path to cached package: ~/.shot/cache/<name>/<version>/
    pub fn package_path(&self, name: &str, version: &str) -> PathBuf {
        self.root.join(name).join(version)
    }

    /// Check if package is cached
    pub fn is_cached(&self, name: &str, version: &str) -> bool {
        self.package_path(name, version).exists()
    }

    /// Cache a package from source path
    pub fn cache_package(&self, source: &Path, name: &str, version: &str) -> Result<PathBuf, String> {
        let target = self.package_path(name, version);

        // Remove old cache if exists
        if target.exists() {
            fs::remove_dir_all(&target)
                .map_err(|e| format!("Failed to remove old cache: {}", e))?;
        }

        // Create target directory
        fs::create_dir_all(&target)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;

        // Copy all files recursively
        copy_dir_recursive(source, &target)?;

        Ok(target)
    }
}

/// Copy directory recursively
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    if !dst.exists() {
        fs::create_dir_all(dst)
            .map_err(|e| format!("Failed to create {}: {}", dst.display(), e))?;
    }

    let entries = fs::read_dir(src)
        .map_err(|e| format!("Failed to read {}: {}", src.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        let file_name = path.file_name().unwrap();
        let target = dst.join(file_name);

        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            fs::copy(&path, &target)
                .map_err(|e| format!("Failed to copy {}: {}", path.display(), e))?;
        }
    }

    Ok(())
}
