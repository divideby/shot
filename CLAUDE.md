# Shot — Package Manager for Claude Code

## Project Overview

Shot is a package manager for Claude Code artifacts (commands, agents, rules, scripts). Written in Rust.

**Full specification:** `docs/SPEC.md`

## Quick Reference

### What Shot Manages

| Artifact | Location | Context |
|----------|----------|---------|
| commands | `.claude/commands/` | on-demand |
| agents | `.claude/agents/` | on-demand |
| rules | `CLAUDE.md` | always |
| scripts | `~/.local/bin/` | — |

### Key Commands (MVP)

```bash
shot init              # Create shot.toml
shot install <path>    # Install package from local path
shot install           # Install all from shot.toml
shot install -g <path> # Global install to ~/.claude/
shot list              # List installed packages
shot remove <pkg>      # Remove package
shot doctor            # Check consistency
shot repair            # Reinstall from cache
```

### Storage (Cargo-style)

```
~/.shot/cache/           # Global cache (all packages)
project/shot.toml        # Project manifest + aliases
project/shot.lock        # Locked versions
project/.claude/         # Installed commands/agents
```

## Architecture

### Core Types

```rust
// Package manifest (shot.toml in package)
struct PackageManifest {
    package: PackageInfo,
    install: InstallConfig,
    dependencies: HashMap<String, Dependency>,
}

// Project manifest (shot.toml in project)
struct ProjectManifest {
    project: ProjectInfo,
    dependencies: HashMap<String, DependencySpec>,
}

// Lock file
struct LockFile {
    packages: Vec<LockedPackage>,
}

// Source abstraction
trait Source {
    fn resolve(&self, pkg: &str) -> Result<PackageMeta>;
    fn fetch(&self, pkg: &str, dest: &Path) -> Result<()>;
}
```

### File Structure

```
src/
├── main.rs          # CLI entry point (clap)
├── cli/             # Command implementations
│   ├── mod.rs
│   ├── init.rs
│   ├── install.rs
│   ├── list.rs
│   ├── remove.rs
│   ├── doctor.rs
│   └── repair.rs
├── manifest/        # TOML parsing
│   ├── mod.rs
│   ├── package.rs   # PackageManifest
│   ├── project.rs   # ProjectManifest
│   └── lock.rs      # LockFile
├── source/          # Package sources
│   ├── mod.rs
│   ├── local.rs     # LocalSource (MVP)
│   └── github.rs    # GitHubSource (v0.2)
├── cache.rs         # ~/.shot/cache/ management
├── installer.rs     # Copy files to .claude/
└── error.rs         # Error types
```

## Development

### Build & Run

```bash
cargo build
cargo run -- init
cargo run -- install ~/test-package
```

### Dependencies (Cargo.toml)

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
walkdir = "2"
dialoguer = "0.11"
thiserror = "1"
dirs = "5"
```

### Testing

```bash
cargo test
```

Create test package in `tests/fixtures/reading/` for E2E tests.

## Implementation Order

1. **Каркас** — CLI skeleton with clap, basic types
2. **Кэш** — `~/.shot/cache/` structure, LocalSource
3. **Install** — Copy to cache, then to `.claude/`
4. **Lock** — Create/update `shot.lock`
5. **List/Remove** — Read lock, delete files
6. **Doctor/Repair** — Compare lock vs actual
7. **Conflicts** — Interactive prompts, aliases
8. **Global** — `-g` flag, `~/.claude/`

## Code Style

- Use `thiserror` for error types
- Prefer `?` over `.unwrap()`
- Keep functions small and focused
- Document public APIs with rustdoc

## Useful Links

- [clap docs](https://docs.rs/clap/latest/clap/)
- [serde docs](https://serde.rs/)
- [toml crate](https://docs.rs/toml/latest/toml/)
