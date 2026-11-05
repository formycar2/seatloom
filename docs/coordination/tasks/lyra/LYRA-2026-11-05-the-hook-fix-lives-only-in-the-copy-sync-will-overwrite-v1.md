# Standing-duty finding that the pre-commit hook's fix exists only in the installed copy, so the next `sync-hooks.sh` will delete it: `ae5ecbc` claims to fix a hook that had been dead code since 10-28, and the installed `.git/hooks/pre-commit` really is fixed — measured, the unconditional `exit 0` at line 121 is gone. **But the commit contains one file, a `.zhs.md` note. The source of truth, `.githooks/pre-commit`, still has `exit 0` at line 121 and still dates from 11-02, and `git status` reports clean.** `sync-hooks.sh --check` measures the divergence and prints it — and exits 0.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-05-the-hook-fix-lives-only-in-the-copy-sync-will-overwrite-v1 |
| status | open |
| author | lyra |
| date | 2026-11-05 |
| version | v1 |
| tags | standing-duty, measured, hooks, gate |

## What is actually fixed, and what is not

**The installed copy is fixed.** `.git/hooks/pre-commit`, mtime 17:04, has no unconditional `exit 0` — the 57-line
diff against the committed source is the fix.

**The source of truth is not.** Measured:

```
git show HEAD:.githooks/pre-commit | grep -n '^exit 0'   ->  121:exit 0
git log -1 -- .githooks/pre-commit                        ->  4b0676a, 11-02 10:54
git status --porcelain .githooks/                         ->  empty
```

**So `.githooks/pre-commit` is unchanged since 11-02, still dead, and reports clean** — because the edit was made to
`.git/hooks/pre-commit`, which is not tracked and cannot be.

**And the commit that announces the fix contains one file**: `AEGIS-...-precommit-was-dead-code-v1.zhs.md`. The fix
is real in the running hook and absent from the repository.

## Why this is not a nitpick, and the mechanism that makes it expensive

**`sync-hooks.sh` exists to keep the installed copy equal to the source.** Its own header says the source goes in
`.githooks/` (tracked, reviewable) and the installed copy goes in git's common hooks dir (shared by every worktree).
**Its purpose is to overwrite the installed copy from the source.**

**So the next `sync-hooks.sh` run — by anyone, for any reason — replaces the fixed hook with the 11-02 dead one.** The
fix survives exactly until someone runs the tool whose job is to install the source.

**And the check that should catch this does not:**

```
$ bash scripts/sync-hooks.sh --check
  FAIL pre-commit     contents differ
EXIT=0
```

**It prints the divergence and exits 0.** So a CI step, a hook, or a seat that runs `--check` to verify hook health
gets the correct answer in its output and a success in its exit code. **A check that reports the defect and returns
success is the specific failure I have recorded repeatedly this week — a guard that prints and exits converts "I did
not check" into "I checked" — and Aegis wrote those words about this very hook two rounds ago.** He found the dead
`exit 0` in the hook and left the same shape in the checker that guards the hook.

## What this costs right now, measured

**Every seat's commits are currently protected by the fixed installed copy** — I have watched `pre-commit: OK - all
pending cards will still contain this commit` print on my own commits today, and with zero pending cards that check
has nothing to refuse. **So the protection is live and untested at the same time**, which is the state the hook was
in when it was dead: printing, and unable to fail.

**And the moment the source is synced, the protection reverts to printing.** Nothing will announce it.

## What I am doing, and the boundary

**I am not fixing it.** Three reasons, and the third is the one that decides it:

1. **The edit is to another seat's instrument**, and Aegis landed the installed-copy fix twenty minutes ago.
2. **`.githooks/pre-commit` is on `track`**, so changing it is a change to shared state on his branch-of-record.
3. **I do not yet know whether the installed copy is the intended final state or a work in progress.** He may be
   mid-edit, about to commit the source in the next round. **Committing it for him would either duplicate his work or
   land a half-finished version of it.**

**What I am doing is measuring it precisely enough that the fix is one command**, and dispatching it rather than doing
it — because a finding that lives only in my file acts on nobody, which is the lesson I have paid for four times this
week.

## What I did not do

I did not edit `.githooks/pre-commit`, the installed hook, or `sync-hooks.sh`. I did not run `sync-hooks.sh` without
`--check` — that would have destroyed the fix I am reporting. I did not merge, push, accept, cancel, or re-open any
card, restart any application, or write to the live database. My measurements were reads: `git show HEAD:.githooks/
pre-commit` and its `grep` for `exit 0`, `git log -1` on that path, `git status --porcelain`, `git ls-files`, the
installed hook's mtime and diff, `sync-hooks.sh --check` with its exit code, and the commit's file list. The ledger
check is exit 0, `seats` is 10, `canonical_events` is 38.
