use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::process::Command;
use three_g::ipc::{get_socket_path, FetchRequest};

fn main() -> std::io::Result<()> {
    let socket_path = get_socket_path();
    
    // Cleanup old socket if it exists
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    // Create socket directory if needed
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    println!("3g-daemon listening on {}", socket_path.display());

    // Daemon loop
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(err) => {
                eprintln!("Error accepting connection: {}", err);
            }
        }
    }
    Ok(())
}

fn handle_client(mut stream: UnixStream) {
    let mut buffer = String::new();
    if let Err(e) = stream.read_to_string(&mut buffer) {
        eprintln!("Failed to read from client: {}", e);
        return;
    }

    let request: FetchRequest = match serde_json::from_str(&buffer) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("Invalid request: {}", e);
            return;
        }
    };

    println!("Received fetch request for: {}", request.repo_path.display());

    // Perform the unshallow fetch
    match fetch_unshallow(&request.repo_path) {
        Ok(_) => println!("Successfully unshallowed {}", request.repo_path.display()),
        Err(e) => eprintln!("Failed to unshallow {}: {}", request.repo_path.display(), e),
    }
}

fn fetch_unshallow(repo_path: &Path) -> std::io::Result<()> {
    // Check if repo exists
    if !repo_path.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Repo not found"));
    }

    let git_dir = repo_path.join(".git");
    if !git_dir.exists() {
         return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Not a valid 3g repo (missing .git)"));
    }

    println!("Starting fetch --unshallow for {}", git_dir.display());

    let status = Command::new("git")
        .arg("fetch")
        .arg("--unshallow")
        .current_dir(&git_dir) 
        .status()?;

    if status.success() {
        Ok(())
    } else {
        // Fallback for non-shallow or already complete repos
        let status_normal = Command::new("git")
            .arg("fetch")
            .current_dir(&git_dir)
            .status()?;
            
        if status_normal.success() {
             Ok(())
        } else {
             Err(std::io::Error::new(std::io::ErrorKind::Other, "Git fetch failed"))
        }
    }
}
