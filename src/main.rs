use clap::{Parser, Subcommand};
use three_g::commands::{clone, branch, add, commit, stash, log, reset, push, pull, diff, show};
use three_g::ipc::get_socket_path;
use std::os::unix::net::UnixStream;
use std::process::Command;

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
    Push {
        /// Force push (behavior: force-with-lease)
        #[arg(short, long)]
        force: bool,
    },
    /// Pull changes from the origin remote and rebase
    Pull {
        /// Optional branch to pull from
        branch: Option<String>,
    },
    /// Show the difference between HEAD and origin (default) or another branch
    Diff {
        /// Optional branch to compare against
        branch: Option<String>,
    },
    /// Show a commit and its changes
    Show {
        /// The commit hash to show
        hash: String,
    },
    /// Manage the background fetch daemon
    Daemon {
        /// Action: start | stop | status
        action: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Clone { url, name } => {
            clone::clone_repo(&url, name)?;
        }
        Commands::Branch { name, base } => {
            branch::add_branch(&name, base.as_deref())?;
        }
        Commands::Add => {
            add::add_all()?;
        }
        Commands::Commit => {
            commit::commit_changes()?;
        }
        Commands::Stash { arg } => {
            stash::handle_stash(arg.as_deref())?;
        }
        Commands::Log => {
            log::show_log()?;
        }
        Commands::Reset => {
            reset::reset_hard()?;
        }
        Commands::Push { force } => {
            push::push_current_branch(force)?;
        }
        Commands::Pull { branch } => {
            pull::pull_rebase(branch.as_deref())?;
        }
        Commands::Diff { branch } => {
            diff::show_diff(branch.as_deref())?;
        }
        Commands::Show { hash } => {
            show::show_commit(&hash)?;
        }
        Commands::Daemon { action } => {
            handle_daemon_command(&action)?;
        }
    }

    Ok(())
}

fn handle_daemon_command(action: &str) -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = get_socket_path();

    match action {
        "start" => {
            if socket_path.exists() {
                // Check if it's actually running by trying to connect
                if UnixStream::connect(&socket_path).is_ok() {
                    println!("Daemon is already running.");
                    return Ok(());
                } else {
                    println!("Socket exists but connection failed. Cleaning up stale socket...");
                    std::fs::remove_file(&socket_path)?;
                }
            }

            println!("Starting 3g-daemon...");
            // Spawn the daemon process
            // We assume the binary '3g-daemon' is in the same directory as '3g' or in PATH
            // For development (cargo run), it's in target/debug/ or target/release/
            
            let exe_path = std::env::current_exe()?;
            let daemon_path = exe_path.parent().unwrap().join("3g-daemon");
            
            Command::new(daemon_path)
                .spawn()?;
                
            println!("Daemon started in background.");
        }
        "stop" => {
            if !socket_path.exists() {
                println!("Daemon is not running.");
                return Ok(());
            }
            // For now, we just kill the process manually or remove socket? 
            // Ideally send a "shutdown" message.
            // Since we didn't implement shutdown, we'll just say "Stop not fully implemented, please kill 3g-daemon process".
            // Or simpler: just remove the socket file to signal "stop" to future clients, 
            // but the process keeps running. 
            // Let's implement a 'shutdown' check in the daemon later.
            // For now, let user know.
            println!("To stop the daemon, please run: pkill 3g-daemon");
        }
        "status" => {
            if socket_path.exists() {
                 if UnixStream::connect(&socket_path).is_ok() {
                    println!("Daemon is running.");
                 } else {
                    println!("Daemon is NOT running (stale socket found).");
                 }
            } else {
                println!("Daemon is NOT running.");
            }
        }
        _ => {
            println!("Unknown action: {}. Use start, stop, or status.", action);
        }
    }
    Ok(())
}
