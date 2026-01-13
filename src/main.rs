use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

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
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => init(),
    }
}

fn init() {
    let path = Path::new("CLAUDE.md");

    if path.exists() {
        eprintln!("CLAUDE.md already exists");
        std::process::exit(1);
    }

    let content = r#"# Shot

Package manager for Claude Code artifacts.

## Usage

```bash
shot init    # Initialize project (creates this file)
```
"#;

    match fs::write(path, content) {
        Ok(_) => println!("Created CLAUDE.md"),
        Err(e) => {
            eprintln!("Failed to create CLAUDE.md: {}", e);
            std::process::exit(1);
        }
    }
}
