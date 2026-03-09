use std::env;
use git2::{Repository, MergeOptions, build::CheckoutBuilder};

pub fn merge_branch(branch_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir()?;

    // 1. Check if we are in the repository root
    if current_dir.join(".git").exists() && current_dir.join(".3g").exists() {
        return Err("Cannot run 'merge' from the repository root. Please run it from inside a branch directory.".into());
    }

    // 2. Open the repository
    let repo = Repository::discover(&current_dir)?;
    
    // 3. Find the branch to merge
    let branch = repo.find_branch(branch_name, git2::BranchType::Local)
        .or_else(|_| repo.find_branch(&format!("origin/{}", branch_name), git2::BranchType::Remote))?;
    
    let fetch_commit = repo.reference_to_annotated_commit(&branch.into_reference())?;
    
    // 4. Analyze the merge
    let (analysis, _) = repo.merge_analysis(&[&fetch_commit])?;
    
    if analysis.is_up_to_date() {
        println!("Already up to date.");
        return Ok(());
    }
    
    if analysis.is_fast_forward() {
        println!("Fast-forwarding to {}...", branch_name);
        let ref_name = "HEAD";
        let mut reference = repo.find_reference(ref_name)?;
        reference.set_target(fetch_commit.id(), "merge: fast-forward")?;
        repo.checkout_head(Some(CheckoutBuilder::new().force()))?;
        println!("Successfully fast-forwarded.");
    } else if analysis.is_normal() {
        println!("Merging {} into current branch...", branch_name);
        
        let mut merge_opts = MergeOptions::new();
        repo.merge(&[&fetch_commit], Some(&mut merge_opts), None)?;
        
        if repo.index()?.has_conflicts() {
            println!("Automatic merge failed; fix conflicts and then commit the result.");
        } else {
            // Commit the merge result
            let signature = repo.signature()?;
            let mut index = repo.index()?;
            let tree_id = index.write_tree()?;
            let tree = repo.find_tree(tree_id)?;
            
            let head_commit = repo.head()?.peel_to_commit()?;
            let merge_commit = repo.find_commit(fetch_commit.id())?;
            
            let message = format!("Merge branch '{}'", branch_name);
            
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                &message,
                &tree,
                &[&head_commit, &merge_commit],
            )?;
            
            // Clean up the merge state
            repo.cleanup_state()?;
            println!("Successfully merged and committed.");
        }
    } else {
        println!("Merge analysis returned unknown status: {:?}", analysis);
    }

    Ok(())
}
