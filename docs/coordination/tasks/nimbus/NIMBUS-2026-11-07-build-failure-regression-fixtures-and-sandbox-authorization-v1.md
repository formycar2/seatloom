# T3 — Nimbus — 2026-11-07 — Build-failure regression fixtures are shipped, and the sandbox authorization request

Dispatch: `disp-ffJ5K0Cb4IM3HJikkKk-d`
Branch: `packet/nimbus/build-failure-regression` (from `46869cf`)

## What changed

### 1. `scripts/test-build-failure.sh` (new, 18 checks, exit 0)

Ships the regression tests Flux's `ev-M_SmZjyZ6eJjJ4eYWxqNH` said were missing. The verdict
measured three injected shapes at `77a7c5f` — pnpm nonzero with output, pnpm nonzero with no
output, tauri nonzero with stale artifacts — and found all three exiting 1 with zero success
strings. Its criterion `build_failure_paths_have_tests` was `NOT PINNED`, because that
measurement existed as a one-off and not as a guard. This closes it.

Shape of the suite:

| Section | What it pins |
|---|---|
| 1 | Clean positive case first — otherwise a red below is indistinguishable from "always fails" |
| 2 | pnpm nonzero (with output): nonzero exit, no success claim, pnpm's own error surfaced |
| 3 | pnpm nonzero (no output): nonzero exit, no success claim — the silent case, which only the exit code can catch |
| 4 | tauri nonzero with the previous bundle still on disk: nonzero exit, no success claim, no install |
| 5 | Mutation restores the original fail-open → section 2/3 must go red |
| 6 | Mutation deletes the tauri exit-code check → section 4 must go red |
| 7 | Registration: removing only `rm -rf ui/dist` does **not** break section 4 — recorded, not deleted |

The mutations are the load-bearing part. Each restores a specific historical defect rather
than making a cosmetic change, and each asserts the *intended* assertion goes red for the
*intended* reason.

### 2. A real defect the fixture was hiding

While building the fixture I found that `prepare-sandbox-build.sh`'s **`--build-only` path was
missing `mkdir -p "$WT/target"`** before its first log write. The `--apply` path had it; the
build-only path did not. The same rule fixed in one place and not the other is exactly how
this bug class recurs.

How it was hidden: my first `reset_fixture` did `mkdir -p "$WT/target"`, i.e. **the fixture
was creating the precondition the code under test is responsible for**. Every case was green
and the defect was invisible. Removing that line turned section 1 red with
`line 348: .../target/ui-build.log: No such file or directory` — pnpm receiving exit code 2
from a failed redirection rather than its own failure, and the report saying "frontend build
failed (exit code 2)" with no evidence above it. A failure with no evidence is not a
measurement.

Fix: both paths now create `target/` before writing, with the reason recorded at both sites.

### 3. `rm -rf ui/dist` is defence-in-depth, not section 4's load-bearing gate — recorded honestly

Section 6 originally claimed that removing `rm -rf ui/dist` must break section 4. It does not.
With the exit-code check still in place, pnpm's failure is caught anyway, so removing `rm -rf`
changes nothing observable.

This is worth writing down rather than deleting: **when two gates are stacked, taking one away
and seeing no red does not show it is not guarding anything — it shows it is not the only
load-bearing gate on that path.** A mutation aimed at the wrong object is worse than no
mutation, because it sends someone to "fix" a gate that did not need fixing.

So section 6 now mutates the tauri exit-code check (the gate that actually catches the stale
bundle), and section 7 records the `rm -rf` finding as a named, non-failing check with its
conclusion spelled out. I did not delete the assertion to make the suite green.

## Fixture defects I hit and fixed — each one was a fake green or a fake red

| Defect | Symptom | Why it matters |
|---|---|---|
| `mkdir -p "$W"` before `git worktree add` | `already exists`, swallowed by `2>/dev/null`; every case red on "cannot read git" | A fixture that cannot build reports "the thing under test is broken" |
| Script copy in a non-repo dir | resolver says "cannot read git", message never mentions fixture shape | The error names the wrong cause |
| `.gitignore` missing in fixture | dirty check fires first; every case red on "worktree dirty" | The gate under test is never reached |
| Heredoc with escaped vars under `set -u` | `MODE: unbound variable` — the fake pnpm crashed itself; suite reports "clean build exited 1" | **A fixture that fails on its own cannot measure the code under test** |
| `exit 0` stub in `node_modules/.bin` | Script prefixes that dir on PATH, so the stub always won: always "success", never produced a bundle | Frontend passed, release stuck on "said success but produced nothing" |
| Fake pnpm responding to `tauri-fails` | Section 6's second run died at the frontend step; the tauri mutation never executed | A step's failure must come only from that step's injection, or every gate after it is untestable |
| `reset_fixture` doing `mkdir -p target` | Masked a **real missing-`mkdir` defect in the code under test** | The fixture manufactured a precondition the code owns |
| Mutation applied *after* `reset_fixture` | `git checkout -- .` erased the mutation; reported "the gate guards nothing" | The mutation never reached execution |
| Mutation targeted a copy that was never executed | Same false red | Changing a file nobody runs is not a mutation |
| `--allow-dirty` omitted in mutation sections | Dirty guard fired before the gate under test | The mutation itself dirties a tracked file |

## Verification

- `bash scripts/test-build-failure.sh` — 18 checks, exit 0, three consecutive runs identical
- Both mutations produce the intended red for the intended reason
- `prepare-sandbox-build.sh --build-only` still exits 0; replacement and rollback previews still exit 0
- `bash -n` clean on both scripts

## What the operator is being asked to authorize — sandbox preparation only

I cannot file this as an `operator_decisions` row: `may_create_obligation` in
`crates/seatloom-mcp-emit/src/decisions.rs` refuses any actor that is not the operator or
Aegis, and the refusal is correct. **A seat creating its own authorization is the same hole as
an agent granting itself permissions.** So this is a document, for Aegis to carry into the
governed channel, or for the operator to act on directly.

Measured state at time of writing:

| Fact | Value |
|---|---|
| Main app | pid **90805**, running, untouched |
| Installed sandbox binary | `seatloom-sandbox`, mtime **6 Nov 05:33**, **no** `seatloom-build-commit` label |
| Main app binary | `seatloom-tauri`, mtime 28 Oct 22:16 |
| This branch's HEAD | `46869cf` |

The installed sandbox predates the build-identity work — its binary carries no commit label at
all, so **it cannot state what it is**, which is the exact defect this line of work closed. A
sandbox built from `packet/nimbus/build-failure-regression` would state its own commit.

What is being requested, and nothing more:

1. **Build only.** `bash scripts/prepare-sandbox-build.sh --build-only <branch>` — no install,
   no replacement, no process touched. This already exits 0 and needs no authorization; it is
   listed so the request is bounded.
2. **Prepare the sandbox** so it can be launched for verification, still without touching the
   main app or any running process.

Explicitly **not** requested: replacing the running sandbox, restarting or killing pid 90805,
any live-database write, any migration, any merge to track.

## Not done

- No card, no merge, no install, no restart, no live-DB write, no sandbox replacement, no
  certified install, no required operator SQL.
- No Flux verification of this branch. `ev-M_SmZjyZ6eJjJ4eYWxqNH` verifies `77a7c5f`; this
  branch is a different commit and has no verdict naming it.
- WROOT is measured fixed on this branch (`worktree-root.sh` resolves from
  `git worktree list`); the earlier verdict's `next_steps` wording still describes it as open.
  Flux should re-read that item against the current source rather than the old wording.

## Scope discipline

The payload said "do not let optional unrelated improvements grow scope" and "coordinate CLI
scope results from Lyra separately." I did: no product redesign, no UI work, no unrelated
cleanup. The only production-code change is the two-line `mkdir -p` in
`prepare-sandbox-build.sh`, which the fixture proved was a live defect on the build-only path.
