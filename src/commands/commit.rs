use std::env;
use std::fs;
use std::process::Command;
use git2::Repository;

pub fn commit_changes() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    // 1. Check if we are in the repository root
    if current_dir.join(".git").exists() && current_dir.join(".3g").exists() {
        return Err("Cannot run 'commit' from the repository root. Please run it from inside a branch directory.".into());
    }

    // 2. Open the repository
    let repo = Repository::discover(&current_dir)?;
    
    // 3. Get the index (the staging area)
    let mut index = repo.index()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // 4. Open default editor for the commit message
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    
    // Create a temporary file for the commit message
    let temp_file = env::temp_dir().join("3G_COMMIT_EDITMSG");
    if !temp_file.exists() {
        fs::write(&temp_file, "")?;
    }

    // Use a shell to execute the editor command to handle complex strings
    let status = Command::new("sh")
        .arg("-c")
        .arg(format!("{} \"{}\"", editor, temp_file.display()))
        .status()?;

    if !status.success() {
        return Err(format!("Editor '{}' failed to open or exited with error.", editor).into());
    }

    // 5. Read the commit message
    let message = fs::read_to_string(&temp_file)?;
    fs::remove_file(&temp_file)?;

    if message.trim().is_empty() {
        return Err("Aborting commit due to empty commit message.".into());
    }

    // 6. Create the commit
    let signature = repo.signature()?;
    
    // Get parents (if any)
    let mut parents = Vec::new();
    if let Ok(head) = repo.head() {
        parents.push(head.peel_to_commit()?);
    }
    
    // Convert Vec<&Commit> to Vec of references for the commit function
    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        &message,
        &tree,
        &parent_refs,
    )?;

    println!("Changes committed successfully.");

    Ok(())
}
