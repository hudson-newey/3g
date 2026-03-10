use std::env;
use git2::Repository;

use crate::commands::{add, commit};

pub fn revert_hash(reflog_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    // 1. Check if we are in a worktree (branch) and NOT the root
    if current_dir.join(".git").exists() && current_dir.join(".3g").exists() {
        return Err("Cannot run 'add' from the repository root. Please run it from inside a branch directory.".into());
    }

    // 2. Open the repository (git2 will find the .git/ directory automatically from a worktree)
    let repo = Repository::discover(&current_dir)?;

    // 3. Ensure we are indeed in a worktree or sub-directory of one
    if repo.is_bare() {
        return Err("Cannot run 'revert' in a bare repository.".into());
    }

    // 4. Find the commit using revparse_single (supports OIDs, HEAD, HEAD~n, etc.)
    let obj = repo.revparse_single(reflog_hash)?;
    let commit = obj.peel_to_commit()?;

    // 5. Revert the changes (these are not staged yet)
    repo.revert(&commit, None)?;

    // 6. Add all files and create a merge commit
    add::add_files(None)?;
    commit::commit_changes(false)?;

    Ok(())
}
