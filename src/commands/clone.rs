use git2::{build::RepoBuilder, FetchOptions, RemoteCallbacks};
use std::path::PathBuf;
use std::fs;
use std::io::{self, Write};
use std::os::unix::net::UnixStream;
use crate::ipc::{get_socket_path, FetchRequest};

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

    println!("Cloning into '{}' (depth 1)...", repo_name);
    
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
    fo.depth(1); // Shallow clone

    let mut builder = RepoBuilder::new();
    builder.bare(true);
    builder.fetch_options(fo);

    // Clone into the .git folder
    builder.clone(url, &git_dir)?;
    
    println!("\nRepository cloned successfully (shallow).");
    println!("- Root: {}", target_dir.display());
    println!("- Git Data: {}", git_dir.display());
    println!("- Metadata: {}", three_g_dir.display());
    println!("- No branches checked out (as requested).");

    // 5. Notify daemon to fetch full history
    notify_daemon(&target_dir);

    Ok(())
}

fn notify_daemon(repo_path: &PathBuf) {
    let socket_path = get_socket_path();
    let abs_path = fs::canonicalize(repo_path).unwrap_or(repo_path.clone());
    
    if !socket_path.exists() || UnixStream::connect(&socket_path).is_err() {
        println!("Warning: 3g-daemon is not running. Adding to fetch buffer for later processing.");
        let buffer_path = crate::ipc::get_buffer_path();
        if let Ok(mut file) = fs::OpenOptions::new().append(true).create(true).open(buffer_path) {
            let _ = writeln!(file, "{}", abs_path.display());
        }
        return;
    }

    match UnixStream::connect(&socket_path) {
        Ok(mut stream) => {
            let request = FetchRequest {
                repo_path: abs_path,
            };
            let json = serde_json::to_string(&request).unwrap();
            if let Err(e) = stream.write_all(json.as_bytes()) {
                 eprintln!("Failed to send fetch request to daemon: {}", e);
            } else {
                 println!("Notification sent to 3g-daemon to fetch full history.");
            }
        },
        Err(e) => {
            eprintln!("Failed to connect to 3g-daemon: {}", e);
        }
    }
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
