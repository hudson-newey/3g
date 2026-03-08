use std::fs;

pub fn add_branch(branch_name: &str, base_branch: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
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

    // 3. Find or create the branch
    let branch = match repo.find_branch(branch_name, git2::BranchType::Local) {
        Ok(b) => {
            println!("Found existing local branch '{}'.", branch_name);
            b
        },
        Err(_) => {
            // If not found locally, try to find it as a remote branch and create a local one
            let remote_branch_name = format!("origin/{}", branch_name);
            match repo.find_branch(&remote_branch_name, git2::BranchType::Remote) {
                Ok(rb) => {
                    println!("Found remote branch '{}', tracking it.", remote_branch_name);
                    let commit = rb.get().peel_to_commit()?;
                    repo.branch(branch_name, &commit, false)?
                }
                Err(_) => {
                    // Create from base branch if provided, otherwise from HEAD
                    let commit = if let Some(base) = base_branch {
                        println!("Creating branch '{}' from base '{}'...", branch_name, base);
                        // Find base branch (local or remote)
                        match repo.find_branch(base, git2::BranchType::Local) {
                            Ok(b) => b.get().peel_to_commit()?,
                            Err(_) => {
                                let remote_base = format!("origin/{}", base);
                                repo.find_branch(&remote_base, git2::BranchType::Remote)
                                    .map_err(|_| format!("Base branch '{}' not found", base))?
                                    .get().peel_to_commit()?
                            }
                        }
                    } else {
                        println!("Branch '{}' not found upstream. Creating from HEAD...", branch_name);
                        repo.head()?.peel_to_commit()?
                    };
                    repo.branch(branch_name, &commit, false)?
                }
            }
        }
    };

    let reference = branch.into_reference();

    // 4. Create the worktree
    println!("Checking out branch '{}' into '{}'...", branch_name, branch_name);
    
    // We use a unique name for the worktree, replacing slashes with dashes for internal git naming
    let wt_name = branch_name.replace('/', "-");
    let mut opts = git2::WorktreeAddOptions::new();
    opts.reference(Some(&reference));

    repo.worktree(&wt_name, &target_path, Some(&opts))?;

    println!("Branch '{}' added successfully.", branch_name);
    
    Ok(())
}
