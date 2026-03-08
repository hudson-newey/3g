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
