use std::env;
use std::io::{self, Write};
use git2::{Repository, DiffFormat};

pub fn show_commit(hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    // 1. Check if we are in the repository root
    if current_dir.join(".git").exists() && current_dir.join(".3g").exists() {
        return Err("Cannot run 'show' from the repository root. Please run it from inside a branch directory.".into());
    }

    // 2. Open the repository
    let repo = Repository::discover(&current_dir)?;
    
    // 3. Find the commit using revparse_single (supports OIDs, HEAD, HEAD~n, etc.)
    let obj = repo.revparse_single(hash)?;
    let commit = obj.peel_to_commit()?;
    let oid = commit.id();
    
    // 4. Print commit details
    let author = commit.author();
    let message = commit.message().unwrap_or("(no message)");
    let time = commit.time();
    
    println!("\x1b[33mcommit {}\x1b[0m", oid);
    println!("Author: {} <{}>", author.name().unwrap_or("Unknown"), author.email().unwrap_or("unknown"));
    println!("Date:   {}", format_time(time));
    println!("\n    {}", message.trim_end());
    println!("{}", "=".repeat(40));

    // 5. Generate and print diff against parent
    let commit_tree = commit.tree()?;
    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&commit_tree), None)?;
    
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        io::stdout().write_all(line.content()).unwrap();
        true
    })?;

    Ok(())
}

fn format_time(time: git2::Time) -> String {
    let seconds = time.seconds();
    let offset = time.offset_minutes();
    let sign = if offset >= 0 { '+' } else { '-' };
    let hours = (offset.abs() / 60) as i32;
    let minutes = (offset.abs() % 60) as i32;
    format!("{} (UTC{}{:02}{:02})", seconds, sign, hours, minutes)
}
