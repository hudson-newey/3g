use std::env;
use git2::{Repository, FetchOptions, RemoteCallbacks, RebaseOptions};

pub fn pull_rebase(target_branch: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    // 1. Check if we are in the repository root
    if current_dir.join(".git").exists() && current_dir.join(".3g").exists() {
        return Err("Cannot run 'pull' from the repository root. Please run it from inside a branch directory.".into());
    }

    // 2. Open the repository
    let repo = Repository::discover(&current_dir)?;
    
    // 3. Get the current branch
    let head = repo.head()?;
    let branch_name = head.shorthand().ok_or("Could not get branch name")?;
    let target = target_branch.unwrap_or(branch_name);

    println!("Pulling from 'origin/{}' and rebasing...", target);

    // 4. Fetch from 'origin'
    let mut remote = repo.find_remote("origin")?;
    let mut callbacks = RemoteCallbacks::new();
    
    // Auth: Try SSH agent
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
    });

    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);

    remote.fetch(&[target], Some(&mut fetch_opts), None)?;

    // 5. Find the remote branch commit
    let remote_branch_ref = format!("refs/remotes/origin/{}", target);
    let remote_commit_obj = repo.find_reference(&remote_branch_ref)?
        .peel_to_commit()?;
    
    let upstream = repo.reference_to_annotated_commit(&repo.find_reference(&remote_branch_ref)?)?;

    // 6. Start the rebase
    let mut rebase = repo.rebase(None, Some(&upstream), None, Some(&mut RebaseOptions::new()))?;

    // 7. Iterate through rebase operations
    while let Some(op) = rebase.next() {
        let op = op?;
        let commit = repo.find_commit(op.id())?;
        println!("Rebasing: {}", commit.summary().unwrap_or(""));
        
        rebase.commit(None, &repo.signature()?, None)?;
    }

    rebase.finish(None)?;
    println!("Successfully pulled and rebased onto 'origin/{}'", target);

    Ok(())
}
