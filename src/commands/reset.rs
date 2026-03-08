use std::env;
use git2::{Repository, ResetType};

pub fn reset_hard() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    // 1. Check if we are in the repository root
    if current_dir.join(".git").exists() && current_dir.join(".3g").exists() {
        return Err("Cannot run 'reset' from the repository root. Please run it from inside a branch directory.".into());
    }

    // 2. Open the repository
    let repo = Repository::discover(&current_dir)?;
    
    // 3. Find HEAD
    let head = repo.head()?;
    let target = head.peel_to_commit()?;
    
    // 4. Perform hard reset
    // This updates the index and working directory to match the target commit
    repo.reset(target.as_object(), ResetType::Hard, None)?;

    println!("HEAD is now at {} (Hard reset successful)", target.id());

    Ok(())
}
