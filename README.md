# 3g

A fast git alternative with compatability with most git commands.

## Benefits

### 1. Ability to check out multiple branches

Yes, I know about worktree's, but (imo) they have some bad DX (e.g. `git worktree add -b my-feature my-feature base-feature`)

### 2. Shorter/nicer defaults

`3g add` adds all files
`3g push` pushes to the upstream branch of the same name (without having to use `-u` on the initial push)

### Git history fetching is not a blocking operation.

I use a 3g-daemon to fetch history of fetched repo's in the background, meaning that you can start work right away instead of waiting for history to download.
Hopefully by the time you have a commit to push, the Git history has been downloaded.

## Daemon Management

The `3g-daemon` runs in the background to handle long-running operations like fetching repository history without blocking your workflow. You can manage the daemon using the following commands:

- `3g daemon start`: Starts the background daemon if it's not already running.
- `3g daemon stop`: Gracefully shuts down the background daemon.
- `3g daemon restart`: Restarts the background daemon.
- `3g daemon status`: Checks if the daemon is currently running.

If the daemon is not running when a repository is cloned, fetch requests are buffered and will be processed automatically the next time the daemon starts.
