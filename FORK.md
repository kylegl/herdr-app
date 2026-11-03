# Maintaining your Herdr fork

`personal` is the default branch of `kylegl/herdr-app` and contains your
pane-width feature. `master` is the original upstream baseline, not your
custom build. Upstream is `herdrdev/herdr-app`, for fetching only.

## Check for updates

GitHub checks upstream daily at 09:23 UTC. New commits open or update one
PR from `sync/upstream` into `personal`, entirely inside your fork.
Nothing is merged or installed automatically. Check immediately from this
checkout with:

```sh
git upstream-update
gh pr list --repo kylegl/herdr-app --base personal
```

The equivalent command from anywhere with GitHub CLI is:

```sh
gh workflow run fork-sync.yml --repo kylegl/herdr-app
```

Scheduled workflows can be disabled by GitHub after 60 days without repository
activity. Re-enable the workflow in the fork's Actions tab if that happens.

## Accept an update

Ask your coding agent to review and integrate the update PR, test the merged
result on Linux and native Windows, and install it only when requested.
Use an isolated worktree for integration if this checkout has local changes.
Resolve conflicts without discarding personal features. **Use a merge commit**,
not squash or rebase, so later checks recognize upstream commits already included.

GitHub's built-in workflow token cannot trigger CI by creating a PR. This
automation therefore prepares an update for review, not a tested release.
Run `just check` on the merged result and the native Windows checks before
accepting an app update. Never interpret the update workflow's green check as
an application-test result. Build and installation remain separate, manual steps.

The existing Windows clipboard fixes are still uncommitted local overlays in
both checkouts. Keep them when rebuilding. They are not yet protected by the
fork's branch history. Do not commit or discard them without coordinating with
their owner. PR #1 is the original pane-width comparison against the baseline;
future work targets `personal` instead.

## Fork-only safeguards

Run `bash scripts/setup-fork` in a new checkout. It makes Git pushes and GitHub
CLI default to your fork, disables upstream pushes, and installs a pre-push
hook that rejects every destination except `kylegl/herdr-app`. It also adds
`git upstream-update`. Setup refuses to replace an unrelated existing hook.

These local protections are not an account-wide permission boundary. Explicit
API calls or bypassing hooks could still create an upstream PR. Agents must
follow the fork-only policy in `AGENTS.md` and always name
`--repo kylegl/herdr-app` when publishing. Never submit anything upstream.
