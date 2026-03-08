use std::io::{Read, BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::process::Command;
use three_g::ipc::{get_socket_path, get_buffer_path, FetchRequest};

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

    // Process buffered requests from when daemon was offline
    thread::spawn(|| {
        if let Err(e) = process_buffer_file() {
            eprintln!("Error processing buffer file: {}", e);
        }
    });

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

fn process_buffer_file() -> std::io::Result<()> {
    let buffer_path = get_buffer_path();
    if !buffer_path.exists() {
        return Ok(());
    }

    println!("Processing fetch buffer file: {}", buffer_path.display());

    let file = std::fs::File::open(&buffer_path)?;
    let reader = BufReader::new(file);
    let mut pending_paths: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();

    while !pending_paths.is_empty() {
        let path_str = pending_paths.remove(0);
        let path = PathBuf::from(&path_str);
        
        println!("Buffer processing: Fetching {}", path.display());
        match fetch_unshallow(&path) {
            Ok(_) => {
                println!("Buffer processing: Success for {}", path.display());
                // Successfully fetched, update the file with remaining paths
                std::fs::write(&buffer_path, pending_paths.join("\n") + if pending_paths.is_empty() { "" } else { "\n" })?;
            }
            Err(e) => {
                eprintln!("Buffer processing: Failed for {}: {}. It will remain in buffer.", path.display(), e);
                // Put it back at the beginning of the list to preserve it in the file
                pending_paths.insert(0, path_str);
                std::fs::write(&buffer_path, pending_paths.join("\n") + "\n")?;
                break; // Stop and retry next time the daemon starts
            }
        }
    }

    Ok(())
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
