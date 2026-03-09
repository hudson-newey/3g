use std::env;
use git2::{Repository, StatusOptions, Status};

pub fn show_status() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    // 1. Check if we are in the repository root
    if current_dir.join(".git").exists() && current_dir.join(".3g").exists() {
        return Err("Cannot run 'status' from the repository root. Please run it from inside a branch directory.".into());
    }

    // 2. Open the repository
    let repo = Repository::discover(&current_dir)?;
    
    // 3. Get repository status
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    opts.recurse_untracked_dirs(true);

    let statuses = repo.statuses(Some(&mut opts))?;

    if statuses.is_empty() {
        println!("On branch {}", get_branch_name(&repo)?);
        println!("nothing to commit, working tree clean");
        return Ok(());
    }

    println!("On branch {}", get_branch_name(&repo)?);

    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();

    for entry in statuses.iter() {
        let status = entry.status();
        let path = entry.path().unwrap_or("unknown").to_string();

        if status.intersects(Status::INDEX_NEW | Status::INDEX_MODIFIED | Status::INDEX_DELETED | Status::INDEX_RENAMED | Status::INDEX_TYPECHANGE) {
            staged.push((status, path.clone()));
        }
        
        if status.intersects(Status::WT_MODIFIED | Status::WT_DELETED | Status::WT_TYPECHANGE | Status::WT_RENAMED) {
            unstaged.push((status, path.clone()));
        }

        if status.intersects(Status::WT_NEW) {
            untracked.push(path);
        }
    }

    if !staged.is_empty() {
        println!("\nChanges to be committed:");
        for (status, path) in staged {
            let label = if status.contains(Status::INDEX_NEW) { "new file:   " }
                        else if status.contains(Status::INDEX_MODIFIED) { "modified:   " }
                        else if status.contains(Status::INDEX_DELETED) { "deleted:    " }
                        else if status.contains(Status::INDEX_RENAMED) { "renamed:    " }
                        else { "typechange: " };
            println!("  \x1b[32m{}\x1b[0m{}", label, path);
        }
    }

    if !unstaged.is_empty() {
        println!("\nChanges not staged for commit:");
        for (status, path) in unstaged {
            let label = if status.contains(Status::WT_MODIFIED) { "modified:   " }
                        else if status.contains(Status::WT_DELETED) { "deleted:    " }
                        else if status.contains(Status::WT_RENAMED) { "renamed:    " }
                        else { "typechange: " };
            println!("  \x1b[31m{}\x1b[0m{}", label, path);
        }
    }

    if !untracked.is_empty() {
        println!("\nUntracked files:");
        for path in untracked {
            println!("  \x1b[31m{}\x1b[0m", path);
        }
    }

    Ok(())
}

fn get_branch_name(repo: &Repository) -> Result<String, Box<dyn std::error::Error>> {
    let head = repo.head()?;
    if head.is_branch() {
        Ok(head.shorthand().unwrap_or("unknown").to_string())
    } else {
        Ok(format!("DETACHED HEAD at {}", head.target().map(|t| t.to_string()).unwrap_or_default()))
    }
}
