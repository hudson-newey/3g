use std::env;
use git2::Repository;

pub fn handle_tag(tag_name: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    // 1. Check if we are in the repository root
    if current_dir.join(".git").exists() && current_dir.join(".3g").exists() {
        return Err("Cannot run 'tag' from the repository root. Please run it from inside a branch directory.".into());
    }

    // 2. Open the repository
    let repo = Repository::discover(&current_dir)?;
    
    match tag_name {
        Some(name) => {
            // 3. Create a lightweight tag at HEAD
            let head = repo.head()?;
            let target = head.peel_to_commit()?;
            
            repo.tag_lightweight(name, target.as_object(), false)?;
            println!("Created tag '{}' at {}", name, target.id());
        }
        None => {
            // 4. List tags
            let tags = repo.tag_names(None)?;
            if tags.is_empty() {
                println!("No tags found.");
            } else {
                for tag in tags.iter().flatten() {
                    println!("{}", tag);
                }
            }
        }
    }

    Ok(())
}
