use std::env;
use git2::{Repository, PushOptions, RemoteCallbacks, Error};

pub fn push_current_branch(force: bool) -> Result<(), Box<dyn std::error::Error>> {
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
    
    // 4. Find the 'origin' remote
    let mut remote = repo.find_remote("origin")?;

    // 5. Setup push options and callbacks
    let mut callbacks = RemoteCallbacks::new();
    
    // Auth: Try SSH agent
    callbacks.credentials(|_url, username_from_url, _allowed_types| {
        git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
    });

    if force {
        println!("Force pushing current branch: '{}' (with lease)...", branch_name);
        
        // Implement force-with-lease behavior
        // Find the remote-tracking branch to get our "expected" OID
        let remote_ref_name = format!("refs/remotes/origin/{}", branch_name);
        let expected_oid = match repo.find_reference(&remote_ref_name) {
            Ok(ref_obj) => ref_obj.peel_to_commit()?.id(),
            Err(_) => return Err(format!("Could not find remote-tracking branch '{}'. Have you fetched?", remote_ref_name).into()),
        };

        callbacks.push_negotiation(move |updates| {
            for update in updates {
                if let Some(dst) = update.dst_refname() {
                    if dst.ends_with(branch_name) {
                        if update.src() != expected_oid {
                            return Err(Error::from_str("Remote ref has changed since last fetch (lease failed). Please pull first."));
                        }
                    }
                }
            }
            Ok(())
        });
    } else {
        println!("Pushing current branch: '{}'...", branch_name);
    }

    let mut push_options = PushOptions::new();
    push_options.remote_callbacks(callbacks);

    // 6. Execute push
    // Use '+' prefix for force push if force is true
    let prefix = if force { "+" } else { "" };
    let refspec = format!("{}refs/heads/{}:refs/heads/{}", prefix, branch_name, branch_name);
    
    remote.push(&[refspec], Some(&mut push_options))?;

    println!("Branch '{}' pushed to 'origin' successfully.", branch_name);

    Ok(())
}
