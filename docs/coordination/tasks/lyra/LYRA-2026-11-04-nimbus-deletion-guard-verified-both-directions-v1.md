# Standing-duty verification of Nimbus's delivery: the deletion guard is now on the accept path, before the merge, reusing the same exemption helper, and it is mutation-proven in **both** directions by me — short-circuiting it fails 2 of 5, and moving it after the merge fails 2 of 5. His tests also cover the direction that would make the guard useless if it over-refused. One caveat measured: `cargo fmt --all -- --check` fails on this branch, but on a file from 10-29 that he did not touch — all four of his files are fmt-clean.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-nimbus-deletion-guard-verified-both-directions-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, verification, mutation, measured, gate |

## What he delivered, and where

`6e04dae` on `packet/nimbus/project-scoped-reads` — **not** on `packet/nimbus/session-work-readmodel`, which is the
branch his worktree has checked out. **I nearly reported that he had not done the work**, because I looked at his
checked-out branch first and found nothing. The commit exists, on a different branch of his, and Aegis found the
same thing from the other side by watching file modification times.

| file | change |
|---|---|
| `src-tauri/src/commands/acceptance_cmds.rs` | +40 — the guard |
| `crates/seatloom-core/src/git/ops.rs` | +87 — `undeleted_deletions` and the real-branch test |
| `crates/seatloom-core/tests/accept_deletion_guard.rs` | +371 — five tests |
| `crates/seatloom-mcp-emit/src/dispatch.rs` | −71 — the duplicated logic moved into core |

**The `−71` on `dispatch.rs` is the part worth naming**: rather than copying the guard, he moved the shared part
into `git/ops.rs` so both call sites use one helper. **That is the opposite of the second-exemption-list failure I
warned about in the dispatch, without being asked twice.**

## What I verified, without reading his report first

### The guard is in the right place

`acceptance_cmds.rs:101-112` — `merge_would_delete`, then `undeleted_deletions`, then the refusal, and only then
`merge_branch_into_base` at line 129. **The preview happens before the merge, which is the whole point.**

### It reuses the exemption helper

```rust
let undeleted = seatloom_core::git::ops::undeleted_deletions(would_delete);
```

**Same helper the card-opening gate uses.** So a ruling honoured at one moment cannot be ignored at the other —
which is what the dispatch asked for by name.

### The refusal is actionable

It names the branch, the base, the count, and every path. *A refusal the operator cannot act on is not a refusal.*

## Mutation testing, both directions, by me

**Baseline:** `cargo test -p seatloom-core --test accept_deletion_guard` → **5 passed / 0 failed**, exit 0.

**Mutation 1 — the guard can no longer refuse** (`undeleted` forced to an empty vector):

```
EXIT=101
FAIL ... the_commands_own_composition_refuses_a_destructive_merge_and_allows_a_safe_one
FAIL ... the_guard_is_wired_into_the_accept_command_before_the_merge
test result: FAILED. 3 passed; 2 failed
```

**Mutation 2 — the guard moved to AFTER the merge** (I relocated the block past the `merge_branch_into_base`
call):

```
EXIT=101
FAIL ... the_guard_is_wired_into_the_accept_command_before_the_merge
FAIL ... the_commands_own_composition_refuses_a_destructive_merge_and_allows_a_safe_one
test result: FAILED. 3 passed; 2 failed
```

**Both restored from a `cp` backup, verified byte-identical, baseline back to 5/5, worktree clean.**

**The second mutation is the one I care about most**: it proves the tests pin the guard's *position*, not merely its
existence. A guard that runs after the merge is worse than no guard — it reports a deletion that already happened.

## The criterion from my dispatch that protects the operator, and it is met

I required that the two real pending cards must still accept, because I measured their merge at 24 added / 17
modified / **0 deleted**. **He built that into the tests** (`a_safe_branch_is_not_refused`), with my measurement
quoted in its comment:

```
// THE OTHER DIRECTION, and the one that would make the guard useless if it
// over-refused: the two REAL pending cards must still accept. Measured on
// live for packet/mira/knowledge-panel into track/infra-foundation: 24
// added / 17 modified / 0 DELETED. So a guard that refused any modification
// would break real work.
```

**And I re-measured it against the live branches myself** — `merge-tree --write-tree` on the real pair, then
`diff --name-status`: **0 deletions.** So the guard will not refuse either pending card.

## Gates, and one caveat measured rather than glossed

| gate | result |
|---|---|
| `cargo test -p seatloom-core` | exit 0 |
| `cargo test -p seatloom-tauri --bin seatloom-tauri` | exit 0, 21 passed |
| `cargo clippy -p seatloom-core -p seatloom-tauri --all-targets` | exit 0, zero warnings/errors |
| `cargo fmt --all -- --check` | **exit 1** — see below |

**The `fmt` failure is not his.** It is in `crates/seatloom-core/tests/pipe_guard_real_steal.rs`, last touched by
`d5ac9f0` on 10-29. I checked all four of his files individually against the fmt diff: **zero lines each.** So his
change is fmt-clean and the branch carries pre-existing debt.

**I am recording it rather than reporting his delivery as green**, because "fmt fails on this branch" is true and
"his change is fmt-clean" is also true, and a reader needs both.

## What this unblocks

**This is the prerequisite for the ruling I made two rounds ago** — that clickability must not depend on a delivery
branch being level with an hourly-moving mainline. With the deletion guard on the accept path, the thing the proxy
was standing in for is finally checked where the merge happens. **The second change is still outstanding and still
the operator's to approve.**

## What I did not do

I mutated one file, `src-tauri/src/commands/acceptance_cmds.rs`, twice, in the `nimbus-psr` worktree, restoring each
time from a `cp` backup with `diff -q` verification and re-running the baseline. I did not use `stash`, `reset` or
`clean`. I did not edit his code, merge, push, accept, cancel, or re-open any card, restart any application, or
write to the live database. My measurements were reads plus those two mutations: `git show`/`git log` for the commit
and its branch, `cargo test`/`clippy`/`fmt`, the two mutations with restores, and `merge-tree`/`diff --name-status`
on the real branches. The ledger check is exit 0, `seats` is 10, `canonical_events` is 26.
