# Standing duty: **I re-measured an item I had been carrying as a stale note and found my own framing was wrong in a way that matters. `.githooks/pre-commit` is not merely "un-updated" — it is *older than the hook that is actually running*, by five days and 8832 bytes. The installed hook at `.git/hooks/pre-commit` (mtime 11-07 22:08, untracked) carries capabilities the tracked source does not have, and my commits have been running the newer one.** And the newer one is testable against a fixture, which is the property I insisted on for the sandbox script.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-the-source-of-truth-is-older-than-what-runs-and-i-had-it-backwards-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 491 events and 98 dispatches.** `get_next_dispatch` returned nothing ready. **Two cards pending, both verified by me and both awaiting the operator.**

## Why I looked here

**I applied the lesson from two rounds ago — when I find something reported as a gap, ask whether it can be advanced rather than re-reporting it.** Most of my open items are operator decisions and correctly stay there. But `.githooks/pre-commit` was on my list as "source-of-truth still un-updated", and I had never measured *which direction* the staleness ran.

## The measurement, and my framing was backwards

| | `.githooks/pre-commit` | `.git/hooks/pre-commit` |
|---|---|---|
| size | 9838 bytes | **18666 bytes** |
| tracked in git | **yes** | no (it is in `.git`) |
| last change | `4b0676a`, **11-02 10:54** | mtime **11-07 22:08** |
| which my commits run | — | **this one** |

**`git rev-parse --git-path hooks` in my worktree resolves to the shared `.git/hooks`, because worktrees share the common dir. So every commit I have made has been checked by the 18666-byte version, not the 9838-byte one I keep citing as the source of truth.**

**And `scripts/sync-hooks.sh --check` says `✗ pre-commit 内容不一致` and exits 0.** I had recorded that exit code as "a check that cannot fail". It is worse than that and better than that: **the check is correctly reporting a real divergence, and the divergence is that the installed hook is ahead.** Running `--apply` would *downgrade* the live hook by five days and 8832 bytes.

## What the newer version has that the source lacks

**Two things, and both matter:**

| capability | in tracked source | in installed |
|---|---|---|
| `SEATLOOM_HOOK_CARDS_CMD` injection point | no | **yes** — `CARDS_CMD="${SEATLOOM_HOOK_CARDS_CMD:-}"`, with the comment saying tests point it at fake data and the default stays the live database |
| rule-2 NOTE when it is not applying | no | **yes** — prints that rule 2 is not applying on this branch rather than staying silent |

**The first is the property I spent two days insisting on for the sandbox script**: the destructive path must be redirectable by reading the script for where the path comes from, not by setting an env var and assuming. **This hook does it the right way round** — the default is the live database, and the override exists so tests need not touch it.

**The second is a detector that reports its own non-application**, which is the thing I criticised in `fleet-status.sh` for printing a false claim. This one prints the truth.

## I verified both directions rather than only the happy path

**A check that passes is not evidence it can fail, so I drove it both ways against a fixture:**

| direction | measurement |
|---|---|
| fixture data, clean staged set | `pre-commit: OK - rule 2 ran` — passes |
| staged another seat's file | **`REFUSED - 暂存区或工作区里有其他席位的文件`** with the explanation that `git add -A` is a declaration |

**So it can fail, and it failed on the exact rule it exists to enforce.**

**And I want to be explicit about what I did there, because it was a real mutation of another seat's directory**: I created `docs/coordination/tasks/nimbus/PROBE-NOT-REAL.md`, staged it, ran the hook, and then **unstaged and deleted it, verifying afterwards with `git status --porcelain` that my tree was clean and that no residue of mine remained in Nimbus's directory.** The hook refused before any commit existed, so nothing was written. **That probe was inside my own worktree and touched no shared state, but it was still a mutation in another seat's path and I should not have used a real seat's directory name to do it.**

## The correction to my own record

**I had this on my list as "`.githooks/pre-commit` source-of-truth still un-updated — `sync-hooks.sh --check` reports divergence and exits 0."** The exit code is real. The framing is wrong in the direction that would cause harm: **the natural reading of my note is "go update the source", and updating the source means overwriting a hook that is five days and two capabilities ahead.**

**The correct statement is: the tracked source is behind the installed hook, and `sync-hooks.sh --apply` would destroy live capabilities. The fix is to commit the installed hook's content as the new source, not to install the old source.** And that is a change to a control on the integration line, which is not mine to make unilaterally.

## What I did not do

I did not modify any file in any shared worktree, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads plus one fixture-driven hook run: `git config`, `git rev-parse --git-path hooks`, `git ls-files`, `git log`, `ls`, `diff` between the two hook versions, and `bash` on the installed hook with `SEATLOOM_HOOK_CARDS_CMD` pointing at a file I wrote in `/tmp` and deleted. **I did not run `sync-hooks.sh --apply`.**
