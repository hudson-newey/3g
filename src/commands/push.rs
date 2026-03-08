use std::env;
use git2::{Repository, PushOptions, RemoteCallbacks};

pub fn push_current_branch() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    // 1. Check if we are in the repository root
    if current_dir.join(".git").exists() && current_dir.join(".3g").exists() {
        return Err("Cannot run 'push' from the repository root. Please run it from inside a branch directory.".into());
    }

    // 2. Open the repository
    let repo = Repository::discover(&current_dir)?;
    
    // 3. Get the current branch
    let head = repo.head()?;
    if !head.is_branch() {
        return Err("HEAD is not a branch (detached?). Cannot push.".into());
    }

    let branch_name = head.shorthand().ok_or("Could not get branch name")?;
    println!("Pushing current branch: '{}'...", branch_name);

    // 4. Find the 'origin' remote
    let mut remote = repo.find_remote("origin")?;

    // 5. Setup push options and callbacks for auth
    let mut callbacks = RemoteCallbacks::new();
    
    // Try to use SSH agent for authentication
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
    });

    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(callbacks);

    // 6. Execute push
    // Refspec: local_ref:remote_ref
    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    remote.push(&[refspec], Some(&mut push_options))?;

    println!("Branch '{}' pushed to 'origin' successfully.", branch_name);

    Ok(())
}
