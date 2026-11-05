# The deletion-guard handoff, repaired: card withdrawn, branch rebuilt to contain only the guard

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | NIMBUS-2026-11-07-deletion-guard-handoff-repaired-v1 |
| status | delivered |
| author | seat-nimbus-001 |
| date | 2026-11-07 |
| version | v1 |
| depends_on | disp-OSfAH1EdDaZGvyfaRDnkJ |
| supersedes | NIMBUS-2026-11-07-deletion-guard-acceptance-v1.md |
| tags | deletion-guard, acceptance, handoff, measured, correction |

## What Aegis found, and it was all true

Heartbeat review of `acc-disp-AbQ4m1TKQZGLMeZ4bhwjy` found three defects. Each was
measured against the source before I acted, and each was mine:

| the finding | measured |
|---|---|
| the plan says commit `1cee50c`, the artifact event says `bc4c0a4` | true — the branch tip was `bc4c0a4`, and the card named `1cee50c` |
| the branch diff is not the guard alone | true — 6 files: the guard's 4, plus `scripts/prepare-sandbox-build.sh` and the delivery doc |
| the plan asks the operator to repoint the same card at a destructive branch | true, and worse than it sounds — a card is **terminal** once accepted |

## Why the negative case could not be a card at all

This is the part worth writing down, because it is a design limit rather than a
mistake I can correct by being more careful.

`cmd_accept_delivery` takes a card id, so demonstrating the guard's refusal through
the UI requires a card. But:

- **One pending acceptance per dispatch.** `request_acceptance` refuses a second
  one with "already has a pending acceptance (one at a time; await the operator's
  decision)". So there is no honest way to ask the operator to accept two branches
  on one delivery.
- **A card is terminal.** Once accepted it cannot be repointed, and once cancelled
  it cannot be edited. `cancel_acceptance` records a reason; it does not reopen.
- **Editing the ledger row by hand is not an acceptance.** It would produce the
  appearance of a governed decision with no decision behind it.

So "click Accept on the safe branch, then click Accept on a destructive branch"
describes a sequence the operator cannot perform. My plan described it anyway.
That is the defect: not a wrong step, a step that does not exist.

## What replaced it: a proof that drives the guard's own functions

`crates/seatloom-mcp-emit/proof/accept_deletion_guard_proof.sh` — five points
against real repositories built in a temp directory:

1. a branch that deletes a base-tracked file is refused, **and the refusal names
   the path** — the operator must learn *what* would be lost, not just that
   something would
2. an additive branch is allowed
3. an unreadable repository is an `Err`, not an empty list — "cannot tell" must
   never look like "nothing would be deleted"
4. the guard is wired **before** the merge, in the command's own source
5. the refusal names the branch and the base

It calls the production helpers `merge_would_delete` and `undeleted_deletions`
directly. It does **not** drive `guard_would_refuse`, the copy the unit tests use —
a proof that drives a reimplementation proves the reimplementation.

The proof is mutation-checked. Replacing the exemption filter with `Vec::new()`
makes it fail; restoring makes it pass. A proof that cannot fail is worse than none.

## The branch rebuilt to contain nothing else

`packet/nimbus/deletion-guard-only`, cut fresh from `track/infra-foundation`
(`track/infra-foundation`). `git diff --cached --stat` is the entire list:

```
crates/seatloom-core/src/git/ops.rs                |  87 +++
crates/seatloom-core/tests/accept_deletion_guard.rs | 391 ++++++
crates/seatloom-mcp-emit/proof/accept_deletion_guard_proof.sh | 150 +++
crates/seatloom-mcp-emit/src/dispatch.rs           |  80 +--
src-tauri/src/commands/acceptance_cmds.rs          |  40 ++
```

1 ahead, 0 behind, and `merge-tree --write-tree` against track is clean. The
sandbox-script fix is real work and stays on its own branch
(`packet/nimbus/deletion-guard-accept`, commit `1cee50c`) rather than riding along.

## The verdict names a different branch — what closes the gap, and what does not

`ev-hW75U60DweOY5-qM5pc5r` names `6f1c566` on
`packet/nimbus/project-scoped-reads`. This branch is neither of those things.

What closes the gap is measured, not asserted: **all four guard blobs are
byte-identical** between `6f1c566` and this branch.

| file | blob at 6f1c566 | blob here |
|---|---|---|
| `crates/seatloom-core/src/git/ops.rs` | `f61767f32f` | `f61767f32f` |
| `crates/seatloom-core/tests/accept_deletion_guard.rs` | `29f74ba6fc` | `29f74ba6fc` |
| `crates/seatloom-mcp-emit/src/dispatch.rs` | `487795e310` | `487795e310` |
| `src-tauri/src/commands/acceptance_cmds.rs` | `e3ee47e848` | `e3ee47e848` |

What does **not** close it: my own measurement of my own branch. Flux has to confirm
that comparison himself before the card is reopened. I am not treating my
measurement as his verdict, and the card stays closed until he does.

## Exact scope of what was rebuilt and restarted

Aegis asked for this precisely, so: the **main application was never restarted**.
pid 90805, still the 28 Oct 22:16 binary, and it is not on the list of things I
touched at any point.

What was rebuilt and restarted is the sandbox only, and it is now **stale**: the
running sandbox was built from the previous branch at `1cee50c`. Rebuilding it for
the new branch is a separate act that has not happened, and I am not doing it
before Lyra has a runnable plan — building a sandbox nobody has been asked to
click is how the last round's confusion started.

## Gates

`cargo build --workspace` 0 errors · `cargo test --workspace` 372 passed, 0 failed
· `accept_deletion_guard` 5/5 · clippy 0 on the guard's three crates · proof ALL
POINTS PASS, and mutation-verified to fail when the guard is broken.

Pushed to gitlab (`ac3d1f3`, matching local). origin was unreachable during this
round — `ssh_dispatch_run_fatal: Connection to 172.182.252.133 port 22: Operation
timed out` — so it is not on origin yet.

## Not done, and why

- **No card open.** The previous one is cancelled with the reason recorded. A fresh
  card needs a fresh verdict Flux has confirmed against this branch, and a plan
  Lyra has run.
- **No merge, no deploy, no live-DB write, no app install or restart.**
- **The sandbox is not rebuilt for the new branch**, deliberately, until there is a
  plan that asks the operator to click something.
