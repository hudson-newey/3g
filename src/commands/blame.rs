use std::env;
use std::fs;
use std::path::Path;
use git2::Repository;

pub fn show_blame(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    // 1. Check if we are in the repository root
    if current_dir.join(".git").exists() && current_dir.join(".3g").exists() {
        return Err("Cannot run 'blame' from the repository root. Please run it from inside a branch directory.".into());
    }

    // 2. Open the repository
    let repo = Repository::discover(&current_dir)?;
    
    // 3. Perform the blame operation
    let path = Path::new(file_path);
    let blame = repo.blame_file(path, None)?;
    
    // 4. Read the file content to match with blame hunks
    // Note: We're assuming the file exists on disk and matches the current version.
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let line_number = i + 1;
        if let Some(hunk) = blame.get_line(line_number) {
            let oid = hunk.final_commit_id();
            let commit = repo.find_commit(oid)?;
            let author = commit.author();
            let author_name = author.name().unwrap_or("Unknown");
            
            // Format: short_oid author (line_number) content
            println!("\x1b[33m{:.8}\x1b[0m \x1b[32m{:>12}\x1b[0m ({:>3}) {}", oid, author_name, line_number, line);
        } else {
            println!("        {:>12} ({:>3}) {}", "unknown", line_number, line);
        }
    }

    Ok(())
}
