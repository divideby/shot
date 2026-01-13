use clap::{Parser, Subcommand};
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
    /// Install a package from local path
    Install {
        /// Path to the package directory
        path: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => init(),
        Commands::Install { path } => install(&path),
    }
}

fn init() {
    let path = Path::new("CLAUDE.md");

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

    let existed = path.exists();
    let content = if existed {
        let existing = fs::read_to_string(path).unwrap_or_default();
        if existing.contains("## Shot") {
            eprintln!("Shot section already exists in CLAUDE.md");
            std::process::exit(1);
        }
        format!("{}\n{}", existing.trim_end(), shot_section)
    } else {
        format!("# Project{}", shot_section)
    };

    match fs::write(path, content) {
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

fn install(package_path: &Path) {
    // Expand ~ to home directory
    let package_path = if package_path.starts_with("~") {
        let home = std::env::var("HOME").expect("HOME not set");
        PathBuf::from(home).join(package_path.strip_prefix("~").unwrap())
    } else {
        package_path.to_path_buf()
    };

    // Check package exists
    if !package_path.exists() {
        eprintln!("Package not found: {}", package_path.display());
        std::process::exit(1);
    }

    // Check shot.toml exists
    let manifest_path = package_path.join("shot.toml");
    if !manifest_path.exists() {
        eprintln!("Not a shot package: missing shot.toml");
        std::process::exit(1);
    }

    // Find commands
    let commands_dir = package_path.join("commands");
    if !commands_dir.exists() {
        eprintln!("No commands directory in package");
        std::process::exit(1);
    }

    // Create .claude/commands/ in current directory
    let target_dir = Path::new(".claude/commands");
    if let Err(e) = fs::create_dir_all(target_dir) {
        eprintln!("Failed to create {}: {}", target_dir.display(), e);
        std::process::exit(1);
    }

    // Copy command files
    let entries = match fs::read_dir(&commands_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read commands directory: {}", e);
            std::process::exit(1);
        }
    };

    let mut count = 0;
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

    if count == 0 {
        println!("No commands found in package");
    } else {
        println!("Installed {} command(s)", count);
    }
}
