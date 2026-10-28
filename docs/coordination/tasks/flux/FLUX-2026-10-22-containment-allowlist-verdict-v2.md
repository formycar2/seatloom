# Verdict — the deletion exemption list (re-issue): **PASS**, branch premise now confirmed

- **Verification dispatch**: `disp-tJEX9HvR38W0q6xL88KFH` (a re-issue; Aegis cancelled
  `disp-c6WJXKD50eWHSRJJ7sWc2` because its payload named a branch that does not exist).
- **Implementation dispatch**: **`disp-5WQ7UeLjxhEd6j8PsuOxL`** (`verify_dispatch_id`).
- **Branch under test**: `packet/nimbus/project-scoped-reads` @ `6fc6b4a` — and this dispatch's
  premise is now airtight: `6fc6b4a` is **both** checked out and the **tip** of that branch
  (`rev-parse HEAD` == `6fc6b4a`). The working tree is clean, so the tree I tested is exactly the
  artifact's tree.
- **Verdict event**: `ev-4fKDMZztUVkasOWvCmxfn`, emitted with `turn_seq` null.
- **VERDICT: PASS** — with one finding, already escalated by Nimbus and confirmed independently
  below.

## A note on the cancelled dispatch, since I did that work under it

The first dispatch named `packet/nimbus/containment-allowlist`, which exists in no worktree and on
neither remote; I verified the commit object and recorded that limit. Aegis then cancelled it, saying
it had the branch name wrong, and re-issued with `packet/nimbus/project-scoped-reads`. That is
correct — the branch exists, is checked out, and `6fc6b4a` is its tip.

So the limit that was attached to my previous PASS is now resolved: the artifact is confirmed. This
verdict is not a fresh independent investigation for its own sake — it re-states the same measured
evidence with the one open question closed, and I re-ran the two most decisive mutations after
establishing the tree is byte-identical, so the numbers below are measured under this dispatch.

## Method — four mutations, one per hard requirement

| mutation | result |
|---|---|
| `path: "ui/src/app-v2/AppV2.tsx"` → `path: "*"` (the exact risk Aegis asked about) | **2 assertions FAIL** |
| `BranchNotSynced` given an exemption side-door | **1 assertion FAILS** |
| the exemption filter removed so the list is consumed nowhere | **1 assertion FAILS** |
| `ruled_by: ""` | **BUILD FAILS** — `error[E0080]: evaluation panicked: no ruled_by: nobody can tell whose ruling this is` |

Plus a path that cannot appear in a const entry at all: a filename containing a literal tab is
quoted by `git diff` (`"ui/src/app-v2/AppV2\tweird.tsx"`), so it can never equal a plain entry and
the deletion stays defended — exactness fails closed on the unquotable case.

Every mutation restored byte-identically; `git status` clean afterwards, `cargo build` green again.

## The three hard requirements, each with how it was checked

**1. Static, no env-var switch.** ✔ Checked **structurally rather than by prose**: `DELETION_EXEMPTIONS`
is a `const` with no `Lazy`/`OnceCell`/`OnceLock`/`static mut` behind it, and the matcher's entire
signature is `fn is_deletion_exempt(path: &str) -> bool` — a path in, a bool out, no environment. The
crate does have four `SEATLOOM_*` env vars (`ACTOR_REF`, `PROJECT_ID`, `INTEGRATION_BRANCH`,
`REPO_ROOT`), all pre-existing, and none on the exemption path.

**2. Only `MergeWouldDelete` opens; `BranchNotSynced` does not.** ✔ By **control flow, not comment**:
the `BranchNotSynced` return (~line 1815) precedes the call to `merge_would_delete` entirely, so no
exemption can reach the sync verdict even in principle. Mutation 2 proves the isolation is real.

**3. An unruled deletion is still refused.** ✔ The subtraction is precise: the `mixed` fixture deletes
the exempt path **plus** `keep.txt`, is still refused, and `paths` is exactly `["keep.txt"]` — while
the error message does not mention `AppV2.tsx`, because it was already subtracted. So the exemption
narrows the refusal to what remains; it does not widen what passes.

## Not weakened

`request_acceptance_refuses_unsynced_and_destructive_branches` — the pre-existing containment test
Aegis read — is **byte-identical** to its version at `6fc6b4a^` (diffed as a whole function body) and
still passes. The new tests were added beside it, not by rewriting it. `merge_would_delete`,
`branch_contains` and the refusal wording are untouched.

Aegis's own reading is accurate: `exempt_deletion_passes_while_the_rest_are_still_refused` really
does assert that an outside-the-list deletion is still refused *and* that the message names only the
un-exempted paths.

## Gates, run by me in Nimbus's worktree at `6fc6b4a`

`cargo build -p seatloom-mcp-emit` clean, zero warnings · `cargo build --workspace` clean ·
`cargo test --workspace` **280 passed / 0 failed** (emit crate **83 passed**, matching Aegis's
independently-run count) · `cargo clippy -p seatloom-mcp-emit --all-targets` **0 warnings** ·
`cargo fmt -p seatloom-mcp-emit -- --check` clean. The 22 `--ignored` in the emit crate are
pre-existing DB tests.

## Finding — Nimbus escalated it; I confirm it independently

`crates/seatloom-core/src/git/ops.rs`'s header states that `cmd_accept_delivery` re-runs all three
containment helpers at decision time. **It does not.** `merge_would_delete`/`branch_contains` are
called from exactly one non-test place — `request_acceptance` — and `merge_branch_into_base`
(`src-tauri/src/commands/acceptance_cmds.rs:311`) checks only: is-a-git-tree, clean tracked tree,
branch exists, base exists, base checkout, then merge with conflict→abort. No containment re-run.

Consequence: a pending acceptance card recorded before the guard existed would, on the operator's
click, execute the deletion the guard exists to forbid. This change **neither caused nor widened**
that — the list is consumed only inside `request_acceptance`, and the accept-time merge path would
pull the same deletion whether or not the path is on the list. Nimbus was right not to touch it,
right that it is Aegis's decision, and right that the misleading part is the comment. I make no
recommendation on the fix; choosing it is Aegis's.

## Honest limits

- **The defence was exercised against the scratch-repo fixtures, not a live git server or a real
  merge.** Those fixtures are the mechanism the tests use, and my mutations used the same fixtures,
  so this is the layer the code is proven at — no more.
- **Staticity is a source-level structural claim**, not a runtime one: I asserted the `const` with no
  lazy initialiser, and the matcher's signature, rather than attempting a runtime bypass. There is no
  runtime surface to bypass it with, which is the point.
- **`git/ops.rs`'s claim as a whole**: I traced the non-test call sites of the two helpers and read
  `merge_branch_into_base` in full; I did not enumerate every caller in `src-tauri` for other guards.

## Bottom line

**PASS.** The exemption list is a compile-time constant with an exact-match matcher that reads
nothing but that constant; `BranchNotSynced` is unreachable to it by control flow; an unruled deletion
is still refused and still named precisely; the ruling fields cannot be stripped without failing the
**build** rather than a test; and the pre-existing containment test is byte-for-byte what it was
before this change. Four mutations — including the wildcard Aegis specifically asked me to look for
— are all caught, by a different mechanism each. Every gate is green. One finding, correctly
escalated by the implementer and third-party confirmed here: the accept-time path does not re-run
containment despite the comment promising it. Pre-existing, untouched by this change, and Aegis's call.
