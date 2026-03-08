mod commands;

use clap::{Parser, Subcommand};
use commands::clone::clone_repo;
use commands::branch::add_branch;
use commands::add::add_all;
use commands::commit::commit_changes;
use commands::stash::handle_stash;

#[derive(Parser)]
#[command(name = "3g")]
#[command(about = "A git alternative", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Clone a repository into a new directory
    Clone {
        /// The repository URL to clone from
        url: String,
        
        /// Optional directory name to clone into
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Add a branch as a worktree in the current repository
    Branch {
        /// The name of the branch to add
        name: String,

        /// The base branch to create the new branch from
        base: Option<String>,
    },
    /// Add all changes in the current branch to the staging area
    Add,
    /// Commit staged changes with a message from the default editor
    Commit,
    /// Stash changes or pop the latest stash
    Stash {
        /// "pop" to apply and remove the latest stash, or a name for a new stash
        arg: Option<String>,
    },
    /// Show the commit history for the current branch
    Log,
    /// Reset the current branch to HEAD, discarding all changes (hard reset)
    Reset,
    /// Push the current branch to the origin remote
    Push,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Clone { url, name } => {
            clone_repo(&url, name)?;
        }
        Commands::Branch { name, base } => {
            add_branch(&name, base.as_deref())?;
        }
        Commands::Add => {
            add_all()?;
        }
        Commands::Commit => {
            commit_changes()?;
        }
        Commands::Stash { arg } => {
            handle_stash(arg.as_deref())?;
        }
        Commands::Log => {
            commands::log::show_log()?;
        }
        Commands::Reset => {
            commands::reset::reset_hard()?;
        }
        Commands::Push => {
            commands::push::push_current_branch()?;
        }
    }

    Ok(())
}
