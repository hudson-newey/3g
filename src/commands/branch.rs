use std::fs;

pub fn add_branch(branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Find the .git directory (the bare repository)
    let git_dir = std::env::current_dir()?.join(".git");
    if !git_dir.exists() {
        return Err("Not a 3g repository (could not find .git directory)".into());
    }

    let repo = git2::Repository::open(&git_dir)?;
    
    // 2. Prepare target directory (the branch name as a path)
    let target_path = std::env::current_dir()?.join(branch_name);
    
    if target_path.exists() {
        return Err(format!("Branch directory '{}' already exists", branch_name).into());
    }

    // Ensure parent directories exist if branch name has slashes (e.g., feature/login)
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }

    println!("Adding branch '{}' at '{}'...", branch_name, branch_name);

    // 3. Find the branch (try local first, then remote)
    let branch = match repo.find_branch(branch_name, git2::BranchType::Local) {
        Ok(b) => b,
        Err(_) => {
            // If not found locally, try to find it as a remote branch and create a local one
            let remote_branch_name = format!("origin/{}", branch_name);
            match repo.find_branch(&remote_branch_name, git2::BranchType::Remote) {
                Ok(rb) => {
                    let commit = rb.get().peel_to_commit()?;
                    repo.branch(branch_name, &commit, false)?
                }
                Err(_) => {
                    // If still not found, try to create it from HEAD
                    let head = repo.head()?;
                    let commit = head.peel_to_commit()?;
                    repo.branch(branch_name, &commit, false)?
                }
            }
        }
    };

    let reference = branch.into_reference();

    // 4. Create the worktree
    // We use a unique name for the worktree, replacing slashes with dashes for internal git naming
    let wt_name = branch_name.replace('/', "-");
    let mut opts = git2::WorktreeAddOptions::new();
    opts.reference(Some(&reference));

    repo.worktree(&wt_name, &target_path, Some(&opts))?;

    println!("Branch '{}' added successfully.", branch_name);
    
    Ok(())
}
