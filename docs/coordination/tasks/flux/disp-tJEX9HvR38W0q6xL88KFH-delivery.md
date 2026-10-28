# Delivery: the deletion exemption list — PASS (re-issue), branch premise confirmed

| Field | Value |
|---|---|
| dispatch_id | disp-tJEX9HvR38W0q6xL88KFH |
| verified | disp-5WQ7UeLjxhEd6j8PsuOxL (AEGIS-2026-10-22-containment-allowlist-v1) |
| verdict | pass |
| verdict event | ev-4fKDMZztUVkasOWvCmxfn (ReviewVerdictIssued, emitted) |
| evidence | docs/coordination/tasks/flux/FLUX-2026-10-22-containment-allowlist-verdict-v2.md |
| branch | packet/flux/intent-verdict @ d769ee0 |
| tested at | packet/nimbus/project-scoped-reads @ 6fc6b4a (checked out and the tip) |

## About the re-issue

The first dispatch named `packet/nimbus/containment-allowlist` — a branch that exists in no worktree
and on neither remote. I verified the commit object anyway and recorded that limit explicitly. Aegis
then cancelled it, saying it had the branch name wrong, and re-issued naming
`packet/nimbus/project-scoped-reads`. That branch is real, checked out, and **`6fc6b4a` is its tip**
(`rev-parse HEAD` == `6fc6b4a`, tree clean).

So the one open question from the previous PASS is resolved rather than restated. This verdict
carries the same measured evidence with the artifact confirmed, and I re-ran the two most decisive
mutations after establishing the tree is byte-identical.

## Verdict: PASS

The exemption list is a compile-time constant with an exact-match matcher that reads nothing but
that constant; `BranchNotSynced` is unreachable to it by control flow; an unruled deletion is still
refused and still named precisely; the ruling fields cannot be stripped without failing the
**build**; and the pre-existing containment test is byte-for-byte what it was before.

## Method — four mutations, one per hard requirement

| mutation | result |
|---|---|
| `path: "…AppV2.tsx"` → `path: "*"` | 2 assertions fail |
| `BranchNotSynced` given a side-door | 1 assertion fails |
| the filter removed (list consumed nowhere) | 1 assertion fails |
| `ruled_by: ""` | **BUILD fails** — `error[E0080]` |

Two checks I added because reading alone cannot answer them:
- **Staticity structural, not prose** — `const`, no `Lazy`/`OnceCell`/`OnceLock`/`static mut`, and
  the matcher is literally `fn is_deletion_exempt(path: &str) -> bool`. The crate's four `SEATLOOM_*`
  vars are pre-existing and none is on the exemption path.
- **Exotic paths fail closed** — a filename with a literal tab is quoted by `git diff`
  (`"ui/src/app-v2/AppV2\tweird.tsx"`), so it can never equal a plain entry; the deletion stays
  defended.

All mutations restored byte-identically; build re-verified green afterwards.

## Gates, run by me in Nimbus's worktree at `6fc6b4a`

`cargo build -p seatloom-mcp-emit` clean, zero warnings · `cargo build --workspace` clean ·
`cargo test --workspace` **280 passed / 0 failed**, emit crate **83 passed** (matching Aegis's own
count) · clippy **0 warnings** · fmt clean. The 22 ignored are pre-existing DB tests.

## One finding

`crates/seatloom-core/src/git/ops.rs`'s header claims `cmd_accept_delivery` re-runs containment at
decision time. It does not: `merge_would_delete`/`branch_contains` have exactly one non-test caller
(`request_acceptance`), and `merge_branch_into_base` checks only git-tree / clean-tree / branch /
base / checkout / merge-conflict. A card recorded before the guard existed would execute the
deletion on Accept.

Pre-existing, and **neither caused nor widened** by this change — the list is consumed only inside
`request_acceptance`, and the accept-time path pulls the same deletion whether or not the path is
exempt. Nimbus escalated it and declined to fix it himself, correctly; I confirm it and offer no
recommendation, because choosing the fix is Aegis's.

## Boundary

No code, DB, or merge change by me. Four mutations applied and restored in Nimbus's worktree, which
is clean. No acceptance card opened. I did not attempt a runtime bypass — there is no runtime
surface for one, which is the point of the requirement.

## Artifact index

- Verdict: `docs/coordination/tasks/flux/FLUX-2026-10-22-containment-allowlist-verdict-v2.md`
- Superseded draft: `FLUX-2026-10-22-containment-allowlist-verdict-v1.md` (written under the
  cancelled dispatch; kept because it records the branch-absent limit Aegis then corrected)
- Ledger event: `ev-4fKDMZztUVkasOWvCmxfn`
- This delivery note.
