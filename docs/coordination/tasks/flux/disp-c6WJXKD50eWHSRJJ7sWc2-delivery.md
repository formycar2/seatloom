# Delivery: the deletion exemption list — PASS verdict, all three hard requirements load-bearing

| Field | Value |
|---|---|
| dispatch_id | disp-c6WJXKD50eWHSRJJ7sWc2 |
| verified | disp-5WQ7UeLjxhEd6j8PsuOxL (AEGIS-2026-10-22-containment-allowlist-v1) |
| verdict | pass |
| verdict event | ev-kZ5x8eixmJZYrzbj7GRk- (ReviewVerdictIssued, emitted) |
| evidence | docs/coordination/tasks/flux/FLUX-2026-10-22-containment-allowlist-verdict-v1.md |
| branch | packet/flux/intent-verdict @ 1e48e77 |
| tested at | 6fc6b4a, in Nimbus's worktree |

## Verdict: PASS

The exemption list is a compile-time constant with an exact-match matcher whose signature takes
nothing but a path; `BranchNotSynced` is structurally unreachable to it; an undecided deletion is
still refused and still named; the ruling fields cannot be stripped without breaking the build; and
the pre-existing containment test is byte-for-byte what it was before.

## One limit about where I looked

The dispatch names branch `packet/nimbus/containment-allowlist`. That ref exists in **no** worktree
on this machine and on **neither** remote (`origin` and `gitlab` both return nothing for
`containment`). The commit `6fc6b4a` does exist locally in Nimbus's worktree, as an ancestor of its
checked-out branch, so that tree is what I verified. This verdict covers `6fc6b4a` — which is what
the dispatch asked for ("验 Nimbus 的 6fc6b4a") — but I could not confirm it is the branch's **tip**,
and said so in both the verdict and the event rather than glossing it.

## Method — four mutations, one per hard requirement

| mutation | result |
|---|---|
| `path: "…AppV2.tsx"` → `path: "*"` | 2 assertions fail |
| `BranchNotSynced` given an exemption side-door | 1 assertion fails |
| the exemption filter removed (list consumed nowhere) | 1 assertion fails |
| `ruled_by: ""` | **BUILD fails** — `error[E0080]: evaluation panicked: no ruled_by` |

All restored byte-identically; Nimbus's worktree is clean.

Two checks I made that the dispatch didn't ask for but that decide whether the requirements hold:

- **Staticity checked structurally, not by prose.** `DELETION_EXEMPTIONS` is a `const` with no
  `Lazy`/`OnceCell`/`OnceLock`/`static mut` behind it, and the matcher is literally
  `fn is_deletion_exempt(path: &str) -> bool`. Four pre-existing `SEATLOOM_*` env vars exist in the
  crate; none is on the exemption path.
- **Exotic paths fail closed.** A filename containing a literal tab is quoted by `git diff`
  (`"ui/src/app-v2/AppV2\tweird.tsx"`), so it can never equal a plain const entry and the deletion
  stays defended. Exactness losing the unquotable case is the safe direction.

## Gates, run by me in Nimbus's worktree

`cargo build -p seatloom-mcp-emit` clean, zero warnings · `cargo build --workspace` clean ·
`cargo test --workspace` **280 passed / 0 failed**, emit crate **83 passed** (matching Aegis's own
independently-run number) · `cargo clippy -p seatloom-mcp-emit --all-targets` **0 warnings** ·
`cargo fmt -p seatloom-mcp-emit -- --check` clean. The 22 ignored in the emit crate are pre-existing
DB tests.

## One finding — which the implementer already escalated, and I endorse

`crates/seatloom-core/src/git/ops.rs`'s header claims `cmd_accept_delivery` re-runs the three
containment helpers at decision time. It does not. `merge_would_delete` / `branch_contains` are
reached from exactly one non-test place — `request_acceptance` — and `merge_branch_into_base`
(`src-tauri/src/commands/acceptance_cmds.rs:311`) checks only git-tree / clean-tracked-tree /
branch-exists / base-exists / on-base checkout / merge-with-conflict-abort.

So a pending card recorded before the guard existed would, on Accept, execute the deletion the guard
was meant to forbid. This change **neither caused nor widened** that: the list acts only inside
`request_acceptance`, and the accept-time merge path pulls the same deletion either way. Nimbus was
right not to touch it, right that it is Aegis's decision, and right that the misleading part is the
comment. I make no recommendation about the fix, because choosing it is Aegis's.

## Boundary

No code, DB, or merge change by me. My four mutations were applied and restored in Nimbus's
worktree, which is clean. No acceptance card opened.

## Artifact index

- Verdict: `docs/coordination/tasks/flux/FLUX-2026-10-22-containment-allowlist-verdict-v1.md`
- Ledger event: `ev-kZ5x8eixmJZYrzbj7GRk-` (the commit id is in its payload, since
  `ref_type: "commit"` is not an allowed ref type)
- This delivery note.
