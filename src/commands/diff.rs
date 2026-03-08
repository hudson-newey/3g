use std::env;
use std::io::{self, Write};
use git2::{Repository, BranchType, DiffOptions, DiffFormat};

pub fn show_diff(target_branch: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    // 1. Check if we are in the repository root
    if current_dir.join(".git").exists() && current_dir.join(".3g").exists() {
        return Err("Cannot run 'diff' from the repository root. Please run it from inside a branch directory.".into());
    }

    // 2. Open the repository
    let repo = Repository::discover(&current_dir)?;
    
    // 3. Get current HEAD commit
    let head = repo.head()?;
    let head_commit = head.peel_to_commit()?;
    let head_tree = head_commit.tree()?;

    // 4. Determine the comparison side
    let other_commit = if let Some(name) = target_branch {
        // Find specific branch
        match repo.find_branch(name, BranchType::Local) {
            Ok(b) => b.get().peel_to_commit()?,
            Err(_) => {
                let remote_name = format!("origin/{}", name);
                repo.find_branch(&remote_name, BranchType::Remote)
                    .map_err(|_| format!("Branch '{}' not found locally or on origin.", name))?
                    .get().peel_to_commit()?
            }
        }
    } else {
        // Default to origin/<current_branch>
        let branch_name = head.shorthand().ok_or("Could not get current branch name")?;
        let remote_name = format!("origin/{}", branch_name);
        repo.find_branch(&remote_name, BranchType::Remote)
            .map_err(|_| format!("No upstream branch found for '{}' (origin/{}).", branch_name, branch_name))?
            .get().peel_to_commit()?
    };
    
    let other_tree = other_commit.tree()?;

    // 5. Generate and print diff
    let mut opts = DiffOptions::new();
    let diff = repo.diff_tree_to_tree(Some(&other_tree), Some(&head_tree), Some(&mut opts))?;

    println!("Diffing HEAD against {}:", target_branch.unwrap_or("origin"));
    println!("{}", "=".repeat(40));

    // Print diff in patch format
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        io::stdout().write_all(line.content()).unwrap();
        true
    })?;

    Ok(())
}
