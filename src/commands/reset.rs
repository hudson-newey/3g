use git2::{Repository, ResetType};
use std::env;

use crate::commands::add::add_files;

pub fn reset_hard(
    paths: Option<Vec<String>>,
    soft: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    // 1. Check if we are in the repository root
    if current_dir.join(".git").exists() && current_dir.join(".3g").exists() {
        return Err("Cannot run 'reset' from the repository root. Please run it from inside a branch directory.".into());
    }

    // 2. Open the repository
    let repo = Repository::discover(&current_dir)?;

    // 3. Find HEAD
    let head = repo.head()?;
    let target = head.peel_to_commit()?;

    // If there are files specified in the command line arguments, we only want
    // to reset those files.
    // However, if there are no files in the arguments, we want to automatically
    // reset all files (including uncommitted files).
    if paths.is_none() {
        add_files(None)?;
    } else if let Some(p) = paths && p.is_empty() {
        add_files(None)?;
    }

    let reset_type = if soft {
        ResetType::Soft
    } else {
        ResetType::Hard
    };

    repo.reset(target.as_object(), reset_type, None)?;

    println!("HEAD is now at {} (Hard reset successful)", target.id());

    Ok(())
}
