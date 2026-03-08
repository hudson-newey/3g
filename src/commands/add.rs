use std::env;
use git2::Repository;

pub fn add_all() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;
    
    // 1. Check if we are in a worktree (branch) and NOT the root
    if current_dir.join(".git").exists() && current_dir.join(".3g").exists() {
        return Err("Cannot run 'add' from the repository root. Please run it from inside a branch directory.".into());
    }

    // 2. Open the repository (git2 will find the .git/ directory automatically from a worktree)
    let repo = Repository::discover(&current_dir)?;
    
    // 3. Ensure we are indeed in a worktree or sub-directory of one
    if repo.is_bare() {
        return Err("Cannot run 'add' in a bare repository.".into());
    }

    let mut index = repo.index()?;

    // 4. Add all files (equivalent to git add .)
    // We use an empty pathspec to match everything
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;

    println!("All changes added to the staging area.");
    
    Ok(())
}
