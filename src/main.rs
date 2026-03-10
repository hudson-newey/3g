use clap::{Parser, Subcommand};
use std::os::unix::net::UnixStream;
use std::process::Command;
use three_g::commands::{
    add, blame, branch, clone, commit, diff, log, merge, pull, push, reset, revert, show, stash,
    status, tag,
};
use three_g::ipc::get_socket_path;

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
    /// Add specific files or all changes to the staging area
    Add {
        /// The files to stage (defaults to all if none provided)
        files: Vec<String>,
    },
    /// Commit staged changes with a message from the default editor
    Commit {
        /// Amend the last commit
        #[arg(short, long)]
        amend: bool,
    },
    /// Amend the last commit (shortcut for 'commit --amend')
    Amend,
    /// Stash changes or pop the latest stash
    Stash {
        /// "pop" to apply and remove the latest stash, or a name for a new stash
        arg: Option<String>,
    },
    /// Show the commit history for the current branch
    Log,
    /// Reset the current branch to HEAD, discarding all changes (hard reset)
    Reset,
    Revert {
        /// The commit hash to revert
        reflog_hash: String,
    },
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
    /// Show what revision and author last modified each line of a file
    Blame {
        /// The file path to blame
        file: String,
    },
    /// Show the current working tree status
    Status,
    /// Merge a branch into the current branch
    Merge {
        /// The name of the branch to merge
        branch: String,
    },
    /// List tags or create a new tag at HEAD
    Tag {
        /// The name of the tag to create (leave empty to list tags)
        name: Option<String>,
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
        Commands::Add { files } => {
            add::add_files(Some(files))?;
        }
        Commands::Commit { amend } => {
            commit::commit_changes(amend)?;
        }
        Commands::Amend => {
            commit::commit_changes(true)?;
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
        Commands::Revert { reflog_hash } => {
            revert::revert_hash(&reflog_hash)?;
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
        Commands::Blame { file } => {
            blame::show_blame(&file)?;
        }
        Commands::Status => {
            status::show_status()?;
        }
        Commands::Merge { branch } => {
            merge::merge_branch(&branch)?;
        }
        Commands::Tag { name } => {
            tag::handle_tag(name.as_deref())?;
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
            start_daemon(&socket_path)?;
        }
        "stop" => {
            stop_daemon(&socket_path)?;
        }
        "restart" => {
            stop_daemon(&socket_path)?;
            // Give it a moment to cleanup
            std::thread::sleep(std::time::Duration::from_millis(500));
            start_daemon(&socket_path)?;
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
            println!(
                "Unknown action: {}. Use start, stop, restart, or status.",
                action
            );
        }
    }
    Ok(())
}

fn start_daemon(socket_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    if socket_path.exists() {
        if UnixStream::connect(socket_path).is_ok() {
            println!("Daemon is already running.");
            return Ok(());
        } else {
            println!("Socket exists but connection failed. Cleaning up stale socket...");
            std::fs::remove_file(socket_path)?;
        }
    }

    println!("Starting 3g-daemon...");
    let exe_path = std::env::current_exe()?;
    let daemon_path = exe_path.parent().unwrap().join("3g-daemon");

    Command::new(daemon_path).spawn()?;

    println!("Daemon started in background.");
    Ok(())
}

fn stop_daemon(socket_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    if !socket_path.exists() {
        println!("Daemon is not running (socket not found).");
        return Ok(());
    }

    match UnixStream::connect(socket_path) {
        Ok(mut stream) => {
            use std::io::Write;
            let request = three_g::ipc::DaemonRequest::Shutdown;
            let json = serde_json::to_string(&request).unwrap();
            stream.write_all(json.as_bytes())?;
            println!("Shutdown signal sent to 3g-daemon.");
        }
        Err(_) => {
            println!("Failed to connect to daemon. It might be hanging. Removing socket...");
            std::fs::remove_file(socket_path)?;
        }
    }
    Ok(())
}
