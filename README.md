# 3g (Three-G)

A Git alternative written in Rust, focused on a unique branch-as-directory (worktree-based) workflow.

## Core Philosophy
In `3g`, each branch lives in its own directory within the repository container. This allows you to work on multiple branches simultaneously without expensive checkouts or switching costs.

## Features
- **Smart Containers**: Every repository is a `.git` suffixed directory containing your metadata and branch directories.
- **Branch-as-Directory**: Branches are checked out into subdirectories, keeping your work organized.
- **Shared Stashes**: Stashes are managed at the root level, making them accessible from any branch.
- **Safety First**: Destructive or branch-specific commands (`add`, `commit`, `push`, etc.) are locked down so they can only be run from within a branch directory.

## Usage

### 1. Initialize a Repository
Clone a repository into a new `3g` container.
```bash
3g clone <repository-url> [--name custom-name]
```

### 2. Manage Branches
Check out or create a branch into a subdirectory.
```bash
# In the repository root:
3g branch <name> [base_branch]
```

### 3. Development Workflow
Navigate into a branch directory to use standard development commands:

- **Stage changes**: `3g add` (Adds all changes in the current branch)
- **Commit**: `3g commit` (Opens your default `$EDITOR` for the message)
- **Push**: `3g push` (Pushes the current branch to `origin`)
- **Pull**: `3g pull [branch]` (Fetches and rebases from origin)
- **History**: `3g log` (Shows history for the current branch)
- **Show**: `3g show <hash>` (Shows a specific commit)
- **Diff**: `3g diff [branch]` (Diffs against origin or another branch)
- **Discard changes**: `3g reset` (Hard reset to `HEAD`)

### 4. Stashing
Stashes are shared across all branches in the same container.
```bash
# In any branch:
3g stash "Work in progress"
cd ../another-branch
3g stash pop
```

## Requirements
- Rust (Cargo)
- libgit2
- A default text editor set in `$EDITOR`
