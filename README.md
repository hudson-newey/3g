# 3g (Three-G)

A Git alternative written in Rust, focused on a unique worktree-based workflow.

## Features
- **Smart Cloning**: Creates a `.git` suffixed directory acting as a container for branches.
- **Worktree-First**: Clones a bare repository into a hidden `.git` folder, keeping the root clean for branch worktrees.
- **Metadata**: Includes a `.3g` directory for future metadata and caching.

## Usage

### Cloning a Repository
```bash
cargo run -- clone <repository-url>
```

Example:
```bash
cargo run -- clone https://github.com/rust-lang/rust.git
```

This creates:
```
rust.git/
├── .git/  (Bare repository data)
├── .3g/   (Metadata placeholder)
```
No branches are checked out initially.
