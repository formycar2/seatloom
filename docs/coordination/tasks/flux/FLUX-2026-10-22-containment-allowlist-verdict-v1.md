# Verdict — the deletion exemption list: **PASS**, and the three hard requirements are load-bearing

- **Verification dispatch**: `disp-c6WJXKD50eWHSRJJ7sWc2`.
- **Implementation dispatch**: **`disp-5WQ7UeLjxhEd6j8PsuOxL`** (`verify_dispatch_id` in the payload);
  packet `AEGIS-2026-10-22-containment-allowlist-v1`, commit `6fc6b4a` on
  `packet/nimbus/containment-allowlist` (the object lives in Nimbus's own worktree, where it is
  reachable as an ancestor of the checked-out branch).
- **Verdict event**: `ev-kZ5x8eixmJZYrzbj7GRk-` — emitted as required. (The commit is recorded
  in the event payload rather than as a ref: `ref_type: "commit"` is not an allowed ref type.)
- **VERDICT: PASS** — with **one finding**, and it is the one Nimbus himself escalated: the
  `git/ops.rs` header comment promises a containment re-run at decision time that does not happen.

## Where I looked, and what I could not look at

The dispatch names branch `packet/nimbus/containment-allowlist`. That ref does not exist in any
worktree on this machine (no remote has it; `git ls-remote` on both `origin` and `gitlab` returns
nothing for `containment`). The commit **`6fc6b4a` does exist** locally in Nimbus's worktree and is
an ancestor of its checked-out branch, so I verified that tree. What I could not do is confirm the
branch's tip *is* `6fc6b4a` — if later commits exist on that branch, this verdict covers `6fc6b4a`
only, which is what the dispatch asked for ("验 Nimbus 的 6fc6b4a"). Stated rather than glossed.

## Method — read the decision path, then try to defeat it

For a defence, reading is not enough: the question is whether it can be widened. So beyond the gates
I ran four mutations designed to defeat each hard requirement, plus a source-level assertion of the
staticity claim that the delivery note only asserts in prose:

| mutation | result |
|---|---|
| `path: "…AppV2.tsx"` → `path: "*"` (the wildcard the packet warns about) | **2 assertions FAIL** |
| `BranchNotSynced` given an exemption side-door | **1 assertion FAILS** |
| the exemption filter removed (the list consumed nowhere) | **1 assertion FAILS** |
| `ruled_by: ""` (the compile-time audit's teeth) | **BUILD FAILS** — `error[E0080]: evaluation panicked: no ruled_by: nobody can tell whose ruling this is` |
| an exotic path (a tab in the filename) reaching the matcher | **fails closed** — git quotes such paths (`"…\tweird.tsx"`), so they cannot equal a plain const; exactness loses the unquotable case, which is the safe direction |

All mutations restored; `git status`/`git diff` empty in Nimbus's worktree.

## The three hard requirements

**1. Static, no env-var switch.** ✔ And I checked this structurally, not just by reading: `DELETION_EXEMPTIONS`
is a `const`, with no `Lazy`/`OnceCell`/`OnceLock`/`static mut` behind it, and the matcher's whole
signature is `fn is_deletion_exempt(path: &str) -> bool` — it takes a path and nothing else. Four
`SEATLOOM_*` env vars exist in this crate (`ACTOR_REF`, `PROJECT_ID`, `INTEGRATION_BRANCH`,
`REPO_ROOT`), all pre-existing, and none is read on the exemption path. The only way onto the list
is a code change that leaves a git record.

**2. Only `MergeWouldDelete` opens; `BranchNotSynced` does not.** ✔ Verified **by control flow, not
by comment**: the `BranchNotSynced` return at line ~1815 happens *before* `merge_would_delete` is
even called, let alone before the filter is constructed. There is no reachable path where an
exemption can influence the sync verdict. Mutation 2 proves the isolation is load-bearing.

**3. Undecided deletions are still refused.** ✔ This is the subtraction semantics, and the test is
precise about it: `mixed` (exempt path **plus** `keep.txt`) is still refused with `paths` exactly
`["keep.txt"]`, and the error message does not mention `AppV2.tsx`. So the exemption narrows the
refusal to what remains rather than widening what passes.

## The matching is exact, not fuzzy

`exemption.path == path`, non-negotiable. The near-miss set asserted is stronger than the delivery
note claims (it says eight near-misses; I counted eight, including `*` itself) — and covers a
directory, a directory-with-slash, a suffix, a bare filename, a relative-substring, the empty string
and `*`. The glob check (`*`, `?`, `[`) is applied per entry, so a future `"ui/src/*"` fails too.

**The compile-time audit is real.** Not a test someone could delete: `const _: () = { … assert!(…) }`.
Emptying `ruled_by` fails the *build* with the exact message quoted in the note. This is the right
design decision and it is worth saying so — the failure mode the list guards against is "six months
from now nobody remembers why this line is here and treats it as legacy", and an audit field that
cannot be deleted is a stronger answer than a test that audits it.

## The pre-existing defence was not weakened

`request_acceptance_refuses_unsynced_and_destructive_branches` is **byte-identical** to its
pre-patch version (diffed the whole function body against `6fc6b4a^`), and it still passes. Nothing
in `merge_would_delete`, `branch_contains`, or the refusal wording was touched. The new tests were
added beside it rather than by rewriting it.

## Gates, run by me in Nimbus's worktree

`cargo build -p seatloom-mcp-emit` clean, zero warnings · `cargo build --workspace` clean ·
`cargo test --workspace` **280 passed, 0 failed** (emit crate: **83 passed** — matching Aegis's
independently-run number) · `cargo clippy -p seatloom-mcp-emit --all-targets` **0 warnings** ·
`cargo fmt -p seatloom-mcp-emit -- --check` clean. `22 ignored` in the emit crate are pre-existing
DB tests, unchanged.

## Finding — the delivery note escalated it, and the escalation is correct

The note says `crates/seatloom-core/src/git/ops.rs`'s header claims *"`request_acceptance` runs all
three at request time and `cmd_accept_delivery` re-runs them at decision time"*. **This is false**,
and I confirmed it independently:

- `merge_would_delete` / `branch_contains` are called from exactly one place on the non-test path:
  `dispatch.rs`'s `request_acceptance`.
- `cmd_accept_delivery` → `merge_branch_into_base` (`src-tauri/src/commands/acceptance_cmds.rs:311`)
  checks only: is-a-git-tree, clean tracked tree, branch exists, base exists, on-base checkout, then
  merge with conflict→abort. **No containment re-run.**

Consequence: a pending acceptance card recorded before the guard existed would, on Accept, execute
the deletion the guard was meant to forbid. **This change neither caused nor widened that** — the
exemption list acts only inside `request_acceptance`, and the accept-time merge path would pull the
same deletion regardless of whether it is on the list. Nimbus was right not to touch it, right that
it is Aegis's decision, and right that the *comment* is the part that is misleading. I endorse his
escalation unchanged and make no recommendation about the fix, because choosing it is Aegis's.

## Honest limits

- **I could not reach the branch the dispatch names.** See above; I verified the commit object.
- **I did not exercise the defence against a live git server or a real merge**; the scratch-repo
  tests are the mechanism, and my mutation of the wildcard/`BranchNotSynced` cases used the same
  fixtures they use.
- **I did not attempt to bypass the exemption at runtime.** There is no runtime to bypass it with —
  which is the point of the requirement — but it means my staticity check is a source-inspection
  claim (a structural one, since I asserted the matcher's signature and the absence of lazy
  initialisers), not a runtime measurement.
- **The "0 ignored" caveat above**: I filtered `--ignored` output for containment-relevant tests and
  found none; I did not enumerate all 22.

## Bottom line

**PASS.** The exemption list is a compile-time constant with an exact-match matcher whose signature
takes nothing but a path; `BranchNotSynced` is structurally unreachable to it; an undecided deletion
is still refused and still named; the ruling fields cannot be stripped without breaking the build;
and the pre-existing containment test is byte-for-byte the one that was there before. Four
mutations, including the wildcard the packet specifically warns about, are caught. Every gate is
green at `6fc6b4a` (280 workspace tests, clippy 0, fmt clean). One finding, which the implementer
already escalated correctly: the accept-time path does not re-run containment despite the comment
saying it does — pre-existing, untouched by this change, and Aegis's call.
