# GEMINI.md - 3g Project Context

## Project Overview
**3g** (or `three-g`) is a fast, developer-friendly Git alternative written in Rust. Its primary goal is to improve the developer experience (DX) by making common Git operations faster and more intuitive.

### Key Features
- **Background Fetching:** Uses a background daemon (`3g-daemon`) to perform long-running operations like `git fetch --unshallow` in the background. This allows users to start working on a shallow clone immediately while the full history downloads silently.
- **Multiple Branch Checkout:** Simplified workflow for managing multiple branches (similar to `git worktree` but with improved DX).
- **Opinionated Defaults:** Shorter, more sensible defaults for common commands (e.g., `3g add` stages all changes, `3g push` defaults to the current upstream).
- **Amend Command:** `3g amend` and `3g commit --amend` allow for quick modifications to the last commit, with the editor pre-filled with the existing commit message.

## Architecture & Technology Stack
- **Language:** Rust (2024 edition).
- **CLI Framework:** `clap` (v4).
- **Git Integration:** `git2` (libgit2 bindings) for most operations, with occasional fallbacks to the `git` system command for complex tasks like unshallowing.
- **IPC (Inter-Process Communication):** Unix domain sockets for communication between the CLI and the daemon.
- **Data Serialization:** `serde` and `serde_json`.
- **Path Management:** `directories` crate for platform-agnostic config, cache, and runtime directory discovery.

### Project Structure
- `src/main.rs`: Entry point for the `3g` CLI tool.
- `src/bin/daemon.rs`: Entry point for the `3g-daemon` background process.
- `src/lib.rs`: Shared library containing commands and IPC logic.
- `src/ipc.rs`: IPC message definitions and socket path management.
- `src/commands/`: Individual module for each CLI command (e.g., `clone.rs`, `add.rs`, `commit.rs`).

## Building and Running

### Prerequisites
- Rust and Cargo (latest stable)
- `libgit2` (usually handled by the `git2` crate build script)
- `git` CLI (required by the daemon for certain operations)

### Key Commands
- **Build everything:** `cargo build`
- **Run the CLI:** `cargo run -- <command>`
- **Run the Daemon:** `cargo run --bin 3g-daemon`
- **Start the Daemon via CLI:** `cargo run -- daemon start`
- **Run Tests:** `cargo test`

## Development Conventions
- **Commands:** Each new command should be added as a module in `src/commands/` and registered in `src/commands/mod.rs` and `src/main.rs`.
- **IPC:** Any communication between the CLI and daemon must use the `FetchRequest` struct defined in `src/ipc.rs`.
- **Shallow First:** The `clone` command should always perform a shallow clone (depth 1) and then notify the daemon to unshallow it.
- **Error Handling:** Use `Box<dyn std::error::Error>` for CLI commands and proper `std::io::Result` for daemon/IPC logic.
- **Maintenance:** Whenever a new feature or significant change is introduced, update this `GEMINI.md` file to ensure it remains an accurate source of context for future interactions.

## Daemon & IPC Details
- **Socket Location:** Typically found in the system's runtime directory (e.g., `/run/user/UID/3g-daemon/3g.sock` on Linux).
- **Fetch Buffer:** If the daemon is not running when a clone occurs, the request is written to a buffer file (`daemon-fetch-buffer.txt`) and processed automatically when the daemon next starts.
