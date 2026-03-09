use std::env;
use git2::Repository;

pub fn add_files(paths: Option<Vec<String>>) -> Result<(), Box<dyn std::error::Error>> {
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

    // 4. Add files
    if let Some(p) = paths {
        if p.is_empty() {
            // Default to all if list is empty
            index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
            println!("All changes added to the staging area.");
        } else {
            for path in p {
                index.add_path(std::path::Path::new(&path))?;
            }
            println!("Specified files added to the staging area.");
        }
    } else {
        // Default to all if None
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        println!("All changes added to the staging area.");
    }
    
    index.write()?;
    
    Ok(())
}
