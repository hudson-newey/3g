use git2::{build::RepoBuilder, FetchOptions, RemoteCallbacks};
use std::path::PathBuf;
use std::fs;
use std::io::{self, Write};

pub fn clone_repo(url: &str, name: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Determine repository name
    let repo_name = match name {
        Some(n) => if n.ends_with(".git") { n } else { format!("{}.git", n) },
        None => extract_repo_name(url),
    };
    
    let target_dir = PathBuf::from(&repo_name);
    
    if target_dir.exists() {
        return Err(format!("Destination path '{}' already exists", repo_name).into());
    }

    println!("Cloning into '{}'...", repo_name);
    
    // 2. Create directory structure
    fs::create_dir_all(&target_dir)?;
    
    // 3. Create .3g directory
    let three_g_dir = target_dir.join(".3g");
    fs::create_dir_all(&three_g_dir)?;
    
    // 4. Clone bare repository into .git inside target_dir
    let git_dir = target_dir.join(".git");
    
    // Prepare callbacks for progress
    let mut cb = RemoteCallbacks::new();
    cb.transfer_progress(|stats| {
        if stats.received_objects() > 0 {
             print!("\rReceived {}/{} objects ({})   ", 
                stats.received_objects(), 
                stats.total_objects(),
                bytes_to_human(stats.received_bytes())
             );
             io::stdout().flush().unwrap();
        }
        true
    });

    let mut fo = FetchOptions::new();
    fo.remote_callbacks(cb);

    let mut builder = RepoBuilder::new();
    builder.bare(true);
    builder.fetch_options(fo);

    // Clone into the .git folder
    builder.clone(url, &git_dir)?;
    
    println!("\nRepository cloned successfully.");
    println!("- Root: {}", target_dir.display());
    println!("- Git Data: {}", git_dir.display());
    println!("- Metadata: {}", three_g_dir.display());
    println!("- No branches checked out (as requested).");

    Ok(())
}

fn extract_repo_name(url: &str) -> String {
    let url = url.trim_end_matches('/');
    let last_segment = url.split('/').last().unwrap_or("repository");
    if last_segment.ends_with(".git") {
        last_segment.to_string()
    } else {
        format!("{}.git", last_segment)
    }
}

fn bytes_to_human(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
