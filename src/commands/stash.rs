use std::env;
use git2::{Repository, StashFlags};

pub fn handle_stash(arg: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    // 1. Check if we are in the repository root (container)
    if current_dir.join(".git").exists() && current_dir.join(".3g").exists() {
        return Err("Cannot run 'stash' from the repository root. Please run it from inside a branch directory.".into());
    }

    // 2. Open the repository
    let mut repo = Repository::discover(&current_dir)?;
    
    // 3. Handle 'pop' vs 'save'
    if let Some(arg_str) = arg {
        if arg_str == "pop" {
            pop_stash(&mut repo)?;
        } else {
            save_stash(&mut repo, arg_str)?;
        }
    } else {
        save_stash(&mut repo, "Stashed by 3g")?;
    }

    Ok(())
}

fn save_stash(repo: &mut Repository, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let signature = repo.signature()?;
    
    match repo.stash_save(&signature, message, Some(StashFlags::DEFAULT)) {
        Ok(_) => {
            println!("Changes stashed with message: '{}'", message);
            Ok(())
        }
        Err(e) if e.code() == git2::ErrorCode::NotFound => {
            Err("Nothing to stash.".into())
        }
        Err(e) => Err(e.into()),
    }
}

fn pop_stash(repo: &mut Repository) -> Result<(), Box<dyn std::error::Error>> {
    // Pop the most recent stash (index 0)
    match repo.stash_pop(0, None) {
        Ok(_) => {
            println!("Popped latest stash.");
            Ok(())
        }
        Err(e) if e.code() == git2::ErrorCode::NotFound => {
            Err("No stashes found to pop.".into())
        }
        Err(e) => Err(e.into()),
    }
}
