use std::env;
use git2::Repository;

pub fn show_log() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    // 1. Check if we are in the repository root
    if current_dir.join(".git").exists() && current_dir.join(".3g").exists() {
        return Err("Cannot run 'log' from the repository root. Please run it from inside a branch directory.".into());
    }

    // 2. Open the repository
    let repo = Repository::discover(&current_dir)?;
    
    // 3. Setup revwalk to traverse commits
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?; // Start from current branch HEAD
    revwalk.set_sorting(git2::Sort::TIME)?;

    println!("Commit History:");
    println!("{}", "-".repeat(40));

    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        
        let author = commit.author();
        let message = commit.message().unwrap_or("(no message)");
        let time = commit.time();
        
        // Format basic commit info
        println!("\x1b[33mcommit {}\x1b[0m", oid); // Yellow for hash
        println!("Author: {} <{}>", author.name().unwrap_or("Unknown"), author.email().unwrap_or("unknown"));
        
        // Convert git time to human readable (using a simple formatting)
        println!("Date:   {}", format_time(time));
        println!("\n    {}", message.trim_end());
        println!("{}", "-".repeat(40));
    }

    Ok(())
}

fn format_time(time: git2::Time) -> String {
    // Basic formatting: git2 provides time as seconds from epoch
    // For a real app, we'd use chrono, but for now, we'll just show the raw timestamp
    // or a very simple conversion if we had a library.
    // Let's just use the offset to show it accurately.
    let seconds = time.seconds();
    let offset = time.offset_minutes();
    let sign = if offset >= 0 { '+' } else { '-' };
    let hours = (offset.abs() / 60) as i32;
    let minutes = (offset.abs() % 60) as i32;
    
    // Using a simple placeholder since we don't have chrono added yet
    // In a real environment, we'd add chrono to Cargo.toml
    format!("{} (UTC{}{:02}{:02})", seconds, sign, hours, minutes)
}
