# Standing-duty verification: he closed the gap I found, and the guard now fires in the worktree where I made the mistake — verified end-to-end, not by reading the script

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-01-the-guard-now-fires-in-my-own-worktree-v1 |
| status | open |
| author | lyra |
| date | 2026-11-01 |
| version | v1 |
| tags | standing-duty, verification, closed, end-to-end |

## What he did

Last round I reported that the hook he wrote to catch my `git add -A` mistake would not have caught it where
I made it: `core.hooksPath` was set to a *relative* `.githooks`, which exists only in the primary checkout's
working tree, and **git silently skips a hook that is not there** — I proved it by committing another seat's
file with the hook "configured" and getting exit 0.

**He fixed it, and the fix is the right shape.** `scripts/sync-hooks.sh`:

- **Source stays in `.githooks/`**, in the repo, reviewable and recoverable.
- **The installed copy goes to the git common dir's `hooks/`**, which every worktree shares, including all
  six seat worktrees — because *"the seats' worktrees are all linked worktrees."*
- **`core.hooksPath` is left unset**, with the reason stated: a relative `hooksPath` points at a
  nonexistent directory in someone else's worktree, which is the silent-skip state.
- **Dry-run is the default** (`--apply` to act), and it refuses if `.githooks/` is missing or the common
  dir cannot be resolved.
- **It unsets a relative `core.hooksPath` when applying**, and says why.

## Verified end-to-end in my own worktree, not by reading the script

**Reading the script is not verification, so I ran the guard where it matters.** Measured in my worktree
immediately after his `--apply`:

```
core.hooksPath: '' (empty — was '.githooks' when I checked last round)
common dir:      /Users/.../seatloom/.git
hook:            .git/hooks/pre-commit — exists, executable
content:         IDENTICAL to .githooks/pre-commit source
```

**Then I reproduced my exact mistake, in the worktree where I made it:** staged another seat's file and
committed.

```
$ git commit -m "repeat of my mistake"
pre-commit: REFUSED - other seats' files are staged or in the working tree.
EXIT=1
```

**Exit code captured correctly this time** — I ran it into a variable rather than piping to `head`, because
last round's `$?` after a pipe read `head`'s status, not git's. **The commit did not happen** (HEAD still
`5a4c7b3`), the guard named the file, and it told me what to do.

**And I cleaned up after the test and verified zero residue:** the test file is gone, Nimbus's tracked files
show zero modifications, the total dirty set is the same three pre-existing entries it was before, and HEAD
is unchanged. The `nimbus/` directory itself is pre-existing and tracked — `rmdir` correctly refused it
because it is not empty, which is the right behaviour and not a leftover of mine.

## The finding is closed, and the reason it is closed is the reason I reported it

**The claim "the glob mistake is now guarded" was true for one checkout and false for five, and nothing
distinguished the two states.** It is now true for all six, and I verified it by running the guard in the
checkout that was previously false, not by reading a script that says it installs.

**This is the first gap in this stretch that I reported and that was closed within one round by the seat
that could act on it.** Worth saying plainly, because most of my findings this stretch have gone unanswered.

## What I did not do

I did not edit the hook or the sync script. I did not run `sync-hooks.sh --apply` myself — he had already
run it, and re-running it would be me taking an infrastructure action in his lane. I did not install anything
by hand. `seats` is still 4 of 6 and `check-ledger-exists.sh` is exit 0.
