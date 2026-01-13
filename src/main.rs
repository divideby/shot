mod cache;
mod manifest;

use cache::Cache;
use clap::{Parser, Subcommand};
use manifest::{LockFile, PackageManifest, ProjectManifest};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "shot", about = "Package manager for Claude Code")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project with CLAUDE.md
    Init,
    /// Install a package from local path or GitHub
    Install {
        /// Package source (path or github:user/repo)
        source: String,
    },
    /// List installed packages
    List,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => init(),
        Commands::Install { source } => install(&source),
        Commands::List => list(),
    }
}

fn init() {
    // Create shot.toml
    let shot_toml = Path::new("shot.toml");
    if shot_toml.exists() {
        eprintln!("shot.toml already exists");
        std::process::exit(1);
    }

    let project_name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "my-project".to_string());

    let manifest = ProjectManifest::new(&project_name);
    if let Err(e) = manifest.save(shot_toml) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
    println!("Created shot.toml");

    // Update CLAUDE.md
    let claude_md = Path::new("CLAUDE.md");
    let shot_section = r#"
## Shot

This project is managed by [shot](https://github.com/divideby/shot) — package manager for Claude Code artifacts.

### Commands

```bash
shot install <path>    # Install package from local path
shot install           # Install all from shot.toml
shot list              # List installed packages
shot remove <pkg>      # Remove package
```
"#;

    let existed = claude_md.exists();
    let content = if existed {
        let existing = fs::read_to_string(claude_md).unwrap_or_default();
        if existing.contains("## Shot") {
            // Already has shot section, skip
            return;
        }
        format!("{}\n{}", existing.trim_end(), shot_section)
    } else {
        format!("# {}{}", project_name, shot_section)
    };

    match fs::write(claude_md, content) {
        Ok(_) => {
            if existed {
                println!("Updated CLAUDE.md");
            } else {
                println!("Created CLAUDE.md");
            }
        }
        Err(e) => {
            eprintln!("Failed to write CLAUDE.md: {}", e);
            std::process::exit(1);
        }
    }
}

/// Resolve source to a local path
/// Returns (path_to_package, source_string_for_lock)
fn resolve_source(source: &str) -> (PathBuf, String) {
    if source.starts_with("github:") {
        // GitHub source: github:user/repo or github:user/repo/path
        let repo = source.strip_prefix("github:").unwrap();
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() < 2 {
            eprintln!("Invalid GitHub source: {}. Use github:user/repo or github:user/repo/path", source);
            std::process::exit(1);
        }
        let (user, repo_name) = (parts[0], parts[1]);
        let subpath: Option<PathBuf> = if parts.len() > 2 {
            Some(PathBuf::from(parts[2..].join("/")))
        } else {
            None
        };

        println!("Fetching from GitHub: {}/{}...", user, repo_name);

        // Download to temp directory
        let temp_dir = std::env::temp_dir().join(format!("shot-{}-{}", user, repo_name));
        if temp_dir.exists() {
            let _ = fs::remove_dir_all(&temp_dir);
        }

        // Use git clone (simpler than tarball)
        let status = std::process::Command::new("git")
            .args(["clone", "--depth", "1", &format!("https://github.com/{}/{}.git", user, repo_name), &temp_dir.to_string_lossy()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        match status {
            Ok(s) if s.success() => {}
            _ => {
                eprintln!("Failed to clone repository: {}/{}", user, repo_name);
                std::process::exit(1);
            }
        }

        // If subpath specified, use it
        let package_path = match &subpath {
            Some(p) => temp_dir.join(p),
            None => temp_dir,
        };

        if !package_path.exists() {
            eprintln!("Path not found in repository: {}", package_path.display());
            std::process::exit(1);
        }

        (package_path, source.to_string())
    } else {
        // Local path
        let path = Path::new(source);
        let package_path = if path.starts_with("~") {
            let home = std::env::var("HOME").expect("HOME not set");
            PathBuf::from(home).join(path.strip_prefix("~").unwrap())
        } else {
            fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
        };

        if !package_path.exists() {
            eprintln!("Package not found: {}", package_path.display());
            std::process::exit(1);
        }

        let source_str = format!("path:{}", package_path.display());
        (package_path, source_str)
    }
}

fn install(source: &str) {
    let (package_path, source_type) = resolve_source(source);

    // Load package manifest
    let pkg_manifest_path = package_path.join("shot.toml");
    let pkg_manifest = match PackageManifest::load(&pkg_manifest_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let pkg_name = &pkg_manifest.package.name;
    let pkg_version = &pkg_manifest.package.version;

    println!("Installing {} v{}...", pkg_name, pkg_version);

    // Initialize cache
    let cache = match Cache::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to initialize cache: {}", e);
            std::process::exit(1);
        }
    };

    // Cache the package
    let cached_path = match cache.cache_package(&package_path, pkg_name, pkg_version) {
        Ok(p) => {
            println!("  Cached to {}", p.display());
            p
        }
        Err(e) => {
            eprintln!("Failed to cache package: {}", e);
            std::process::exit(1);
        }
    };

    // Load or check project manifest
    let project_toml = Path::new("shot.toml");
    let mut project_manifest = if project_toml.exists() {
        match ProjectManifest::load(project_toml) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("No shot.toml found. Run 'shot init' first.");
        std::process::exit(1);
    };

    // Load or create lock file
    let lock_path = Path::new("shot.lock");
    let mut lock_file = match LockFile::load(lock_path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // Find and copy commands from cache
    let commands_dir = cached_path.join("commands");
    let target_dir = Path::new(".claude/commands");

    let mut count = 0;
    if commands_dir.exists() {
        if let Err(e) = fs::create_dir_all(target_dir) {
            eprintln!("Failed to create {}: {}", target_dir.display(), e);
            std::process::exit(1);
        }

        let entries = match fs::read_dir(&commands_dir) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!("Failed to read commands directory: {}", e);
                std::process::exit(1);
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "md") {
                let file_name = path.file_name().unwrap();
                let target_path = target_dir.join(file_name);

                if let Err(e) = fs::copy(&path, &target_path) {
                    eprintln!("Failed to copy {}: {}", path.display(), e);
                    std::process::exit(1);
                }

                println!("  + {}", target_path.display());
                count += 1;
            }
        }
    }

    // Update project manifest
    project_manifest.add_dependency(pkg_name, &package_path.display().to_string());
    if let Err(e) = project_manifest.save(project_toml) {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    // Update lock file
    lock_file.add_or_update(pkg_name, pkg_version, &source_type);
    if let Err(e) = lock_file.save(lock_path) {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    if count == 0 {
        println!("Installed {} v{} (no commands)", pkg_name, pkg_version);
    } else {
        println!("Installed {} v{} ({} command(s))", pkg_name, pkg_version, count);
    }
}

fn list() {
    let lock_path = Path::new("shot.lock");

    if !lock_path.exists() {
        println!("No packages installed");
        return;
    }

    let lock_file = match LockFile::load(lock_path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    if lock_file.packages.is_empty() {
        println!("No packages installed");
        return;
    }

    for pkg in &lock_file.packages {
        println!("{}  v{}  ({})", pkg.name, pkg.version, pkg.source);
    }
}
