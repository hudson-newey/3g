use std::path::PathBuf;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct FetchRequest {
    pub repo_path: PathBuf,
}

pub fn get_socket_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "3g", "3g-daemon") {
        let dir = proj_dirs.runtime_dir()
            .unwrap_or_else(|| proj_dirs.cache_dir());
        // Ensure directory exists
        let _ = std::fs::create_dir_all(dir);
        dir.join("3g.sock")
    } else {
        PathBuf::from("/tmp/3g.sock")
    }
}
