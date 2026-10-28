# Delivery: sync the project-filtering delivery branch without restarting the session

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-19-heartbeat-project-filtering-sync-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-19 |
| version | v1 |
| tags | heartbeat, branch-sync, project-filtering, verification |

## Sync

Merged local `track/infra-foundation` at `98876aa931aa144ab873d9deba5aae9ed972931f`
into `packet/mira/knowledge-panel`. Merge commit:
`54190431c8f580424336d71356e2f4f196544434`.
No conflicts. `git merge-base --is-ancestor 98876aa HEAD` exits 0.
Nothing was merged into track. No database writes, model changes, agent restarts or
session changes were performed.

## Behaviour delta

Relative to pre-sync HEAD `afdb633`, only four operations scripts changed:

- `scripts/fix-session-key-stem.sh`: added, read-only by default;
- `scripts/set-seat-default-models.sh`: added, read-only by default, explicit operator apply;
- `scripts/restart-aegis.sh`: changes the default model used on a future launch;
- `scripts/seat-launch.sh`: changes the default model used on a future launch.

These scripts were not executed. Application source under `ui/`, `crates/` and
`src-tauri/` is unchanged by this sync (`git diff --quiet afdb633 HEAD -- ui crates
src-tauri` exits 0). Thus there is no new project-filtering application behaviour,
but there ARE future launcher-default behaviour changes. No claim that the entire
merged branch is behaviour-neutral is made.

## Verification

- `tsc --noEmit`: pass.
- `vitest run`: 13 files, 199 tests passed.
- `vite build`: pass; existing >500 KB chunk advisory remains.
- `cargo check --workspace --all-targets`: pass, including Tauri.
- `cargo test --workspace`: 275 passed, 0 failed; ignored integration tests remain ignored.
- `cargo clippy --workspace --all-targets -- -D warnings`: pass.
- `cargo fmt --all -- --check`: pass.
- `bash -n` on the four operations scripts: pass (syntax only; no script execution).

The first frontend test attempt failed because inherited `NODE_OPTIONS` referenced
the deleted `.agentd/.../loader.cjs`. Tests were rerun in child shells with ONLY that
stale preload variable unset. tmux global environment, current session and model
were not modified. The codemode sandbox remains unavailable; governed MCP reads and
the report used the existing stdio server via the Python adapter.

## Old Flux verdict coverage

Read `ev-UKSo1P85Leoi8bm0hRjfB` from the ledger: ReviewVerdictIssued by Flux, verdict
`pass`, with an explicit dispatch ref for `disp-RUnXBR3uICuWmmFizJ7vX`.
It pinned Mira's implementation at `21633ad` and Nimbus's separate implementation at
`94daf79`. The project-filtering implementation/test files are unchanged from
`21633ad`, so the OLD verdict still covers that unchanged implementation at its
original scope. It is NOT a fresh Flux verdict for this merge commit, the added
operations scripts, or an integrated product.

The original verdict explicitly records three shared-file integration conflicts,
the supervisor continuity/probe default-scope gap, and the Artifacts tab's lack of an
on-screen unscoped-data note. These are not erased by this sync. The current tree
still lacks Nimbus's `require_project_id` project-scoped implementation. The two
halves have NOT been jointly verified. No IPC-error-text or click/pixel-level
verification is claimed.

Read back acceptance `acc-disp-RUnXBR3uICuWmmFizJ7vX`: still `pending`, same branch,
base and Flux reference. No card was reopened, edited, cancelled or accepted.

## Worktree state

Only this delivery note was authored for the heartbeat. Two pre-existing untracked
2026-10-09 scratch documents were left untouched and excluded from commits.
