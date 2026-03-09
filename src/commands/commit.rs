use std::env;
use std::fs;
use std::process::Command;
use git2::Repository;

pub fn commit_changes(amend: bool) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    // 1. Check if we are in the repository root
    if current_dir.join(".git").exists() && current_dir.join(".3g").exists() {
        return Err("Cannot run 'commit' from the repository root. Please run it from inside a branch directory.".into());
    }

    // 2. Open the repository
    let repo = Repository::discover(&current_dir)?;
    
    // Check if shallow
    if repo.is_shallow() {
        return Err("Repository history is incomplete (shallow clone). \
        The 3g-daemon is likely still fetching the full history. \
        Please wait for the background fetch to complete before committing.".into());
    }
    
    // 3. Get the index (the staging area)
    let mut index = repo.index()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    // 4. Get current commit if amending
    let mut existing_message = String::new();
    let mut parents = Vec::new();

    if let Ok(head) = repo.head() {
        let head_commit = head.peel_to_commit()?;
        if amend {
            existing_message = head_commit.message().unwrap_or("").to_string();
            // For amend, parents are the parents of the head commit
            for parent in head_commit.parents() {
                parents.push(parent);
            }
        } else {
            parents.push(head_commit);
        }
    } else if amend {
        return Err("Nothing to amend. This repository has no commits yet.".into());
    }

    // 5. Determine the editor to use
    // Priority: Git config (core.editor) -> EDITOR env var -> vi
    let config = repo.config()?;
    let editor = config.get_string("core.editor")
        .ok()
        .or_else(|| env::var("EDITOR").ok())
        .unwrap_or_else(|| "vi".to_string());
    
    println!("Using editor: {}", editor);
    
    // Create a unique temporary file for the commit message using a timestamp
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    
    let temp_file = env::temp_dir().join(format!("3G_COMMIT_EDITMSG_{}", timestamp));
    
    fs::write(&temp_file, &existing_message)?;

    // Use a shell to execute the editor command to handle complex strings
    let status = Command::new("sh")
        .arg("-c")
        .arg(format!("{} \"{}\"", editor, temp_file.display()))
        .status()?;

    if !status.success() {
        return Err(format!("Editor '{}' failed to open or exited with error.", editor).into());
    }

    // 6. Read the commit message
    let message = fs::read_to_string(&temp_file)?;
    fs::remove_file(&temp_file)?;

    if message.trim().is_empty() {
        return Err("Aborting commit due to empty commit message.".into());
    }

    // 7. Create the commit
    let signature = repo.signature()?;
    
    // Convert Vec<Commit> to Vec of references for the commit function
    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

    let commit_id = repo.commit(
        if amend { None } else { Some("HEAD") },
        &signature,
        &signature,
        &message,
        &tree,
        &parent_refs,
    )?;

    if amend {
        let mut head = repo.head()?;
        head.set_target(commit_id, "commit: amend")?;
        println!("Commit amended successfully.");
    } else {
        println!("Changes committed successfully.");
    }

    Ok(())
}
