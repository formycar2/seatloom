# Replacing the sandbox: a dry-run-first, backup-before-destroy, one-command-rollback plan, awaiting your approval

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-11-07-sandbox-replacement-authorization-v1 |
| status | awaiting-operator-decision |
| author | seat-nimbus-001 |
| date | 2026-11-07 |
| version | v1 |
| depends_on | disp-3MH0GZ8tviLmkln2qAQ81, 34f8681 |
| supersedes | |
| tags | sandbox, authorization, rollback, dry-run, measured |

## What this delivery is, and what it is not

This is **not** another "the sandbox is ready". Last round I built a sandbox, ran a
build, started a process — and Lyra measured that **no authorisation existed to replace
the sandbox**, and that a replacement deletes a running application. So this round
delivers:

1. An **auditable replacement script** (`scripts/replace-sandbox.sh`): dry-run first,
   backup before any destruction, one command to roll back.
2. An **approval request to you** — whether and when to replace is your decision, not
   the script's.

I did not install, kill, restart, touch the live ledger, or reuse a terminal card.

## Why a new script was needed

`prepare-sandbox-build.sh --apply` performs three irreversible acts with **no backup at
any step**:

```
1. kill the running sandbox process
2. rm -rf /Applications/SeatLoom-Sandbox.app      <- the old sandbox is gone here
3. cp -R the new build in
```

After step 2, if step 3 fails (copy, plist edit, or icon swap), the result is a sandbox
that will not start with no way back. The new script wraps those three steps in a
shape that backs up before it destroys.

## What the script does (dry-run is the default)

```
bash scripts/replace-sandbox.sh --from-branch <branch> [must-exist-string]   # dry run
bash scripts/replace-sandbox.sh --from-branch <branch> [string] --apply     # real (needs your approval)
bash scripts/replace-sandbox.sh --rollback --list                           # list rollback points
bash scripts/replace-sandbox.sh --rollback <backup-dir>                     # roll back
```

The dry run states all of the following **before touching anything** — not one byte
changed, not one process killed:

| what it states | why |
|---|---|
| source worktree, branch, commit | which tree this installs |
| **the commit embedded in the binary**, compared against the worktree HEAD | whether the thing being installed is that tree |
| whether the source worktree is clean, listing the paths if not | whether this build equals only that commit |
| where the backup goes and what its metadata records | so a rollback can say what it restored |
| the six steps in order, and what each touches | no surprise steps |
| that the main app is **not in the list** | the one you are using is unaffected |

## The identity check — what this round actually fixed

Lyra measured that the sandbox binary contains four `SEATLOOM_SANDBOX_*` env var
**names** and six hex-looking strings, **none of which is a hash of any commit** — so
"which commit is installed" was previously unverifiable. Which makes the positive case
unfalsifiable: you click Accept, see a merge, and cannot tell whether the guard refused
for the right reason or the merge happened for the right reason.

`34f8681` makes the binary carry its own commit at compile time. And when I wired that
into the replacement script I **hit the same trap again**: the script first searched the
binary for a bare 12-hex-digit run and found `dbdbdbdbdbdb` — a placeholder, not a
commit. The dry run then reported "this build is not this tree" while the build was
entirely correct.

The fix: the commit is emitted next to a fixed label (`seatloom-build-commit=`), and a
reader must **match the label**, never the bare hex. Measured: with the label it reads
`34f8681c3856`, matching the worktree HEAD.

**This check can fail, and I verified that it does**: changing the binary's embedded
commit makes the dry run refuse immediately with "the commit in the binary
(86a65a5db4a4) and the worktree HEAD (34f8681c3856) disagree — this build is not this
tree".

## An older trap fixed on the way

Two `$WT_SHORT` references were followed directly by a full-width punctuation mark.
Under `set -u`, bash reads the multibyte bytes as part of the variable name and reports
`WT_SHORT?: unbound variable` — while the variable is plainly set. This is the
AGENTS.md "brace `${var}` before full-width punctuation" rule in its multibyte form. All
of them are braced now.

## What needs your decision

**Replacing the sandbox is an act that needs authorisation**, because it stops a process
and deletes an application directory. I have no authorisation, so I do not perform it.
To replace, one command:

```bash
cd ~/Documents/GitHub/seatloom-worktrees/nimbus-psr
bash scripts/replace-sandbox.sh --from-branch packet/nimbus/deletion-guard-only --apply
```

Run the dry run first (without `--apply`) and check that the identity and backup
location it reports are what you want.

**Not replacing is also fine.** The only consequence: the sandbox holds the older build
(commit `1cee50c`, no identity label in the binary), while the deletion guard's new
build waits on `packet/nimbus/deletion-guard-only` for you to verify. The guard itself
is independently verified by Flux and driven by me in both directions against real
repositories — it is not short of verification. What it is short of is an application
you can click that can state its own identity.

## Two things that must stay separate (so scope does not merge again)

This branch now carries **two unrelated deliveries**:

| delivery | files | status |
|---|---|---|
| the deletion guard | `ops.rs`, `accept_deletion_guard.rs`, `accept_deletion_guard_proof.sh`, `dispatch.rs`, `acceptance_cmds.rs` | has Flux verdicts (but they name `6f1c566`, not `ac3d1f3`) |
| the binary identity | `build.rs`, `runtime_cmds.rs`, `SandboxBanner.tsx` (+ tests) | **under independent verification; I do not call it verified** |

The identity delivery must **not** be folded into the guard's acceptance scope. Each
needs its own independent verdict.

## Not done, and why

- **No card opened.** All three verdicts name `6e04dae` or `6f1c566`; **none names
  `ac3d1f3`**.
- **No merge, no deployment, no live-ledger write.**
- **The sandbox was not rebuilt** — that waits on your decision, rather than building
  something and waiting for it to be clicked.
- On terminal cards: reusing a cancelled card has a **supported path** —
  `request_acceptance`'s `acceptance_id` parameter, added specifically for "cancelled and
  now want to re-request under the same id" — and raw ledger SQL is neither needed nor
  acceptable.

---

## The approval request could not be filed by me, and the refusal was correct

I tried to put this into `operator_decisions` through the governed tool. It refused:

> `seat:seat-nimbus-001` may not create an obligation — only the operator (`human:*`) and
> `seat:seat-aegis-001` can. A seat creating a "waiting on the operator" could push its
> own work onto his plate, which is a refusal, not a discouraged pattern.

That is the right rule, and I am not going to route around it. The consequence is that
**the approval request reaches you through this note and through Aegis, not through the
`operator_decisions` table** — so it will not appear in `work-list.sh`'s first section as
a pending decision, because it is not one. It is a request from a seat, and the harness
is designed to keep those out of your queue.

If you want this to be a tracked obligation rather than a note, Aegis can create the row
— that is the supported path, and it is his call, not mine.
