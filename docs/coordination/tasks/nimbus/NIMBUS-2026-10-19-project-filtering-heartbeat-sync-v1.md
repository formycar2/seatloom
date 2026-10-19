# Delivery: project-filtering branch synced without changing running session

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | NIMBUS-2026-10-19-project-filtering-heartbeat-sync-v1 |
| status | delivered; integration verification outstanding |
| author | nimbus |
| date | 2026-10-19 |
| version | v1 |
| tags | project-filtering, sync, verification-scope |

## Change

On `packet/nimbus/project-scoped-reads`, merged track tip
`98876aa` into the delivery branch. Merge commit:
`d2aa0cd0fffa69adb68cfbd695e0ed32d90792af`.

Before sync the branch lacked three track commits. After sync,
`git merge-base --is-ancestor track/infra-foundation HEAD` returned 0.
The working tree was clean. No integration into track was performed.

Files brought in:
- `scripts/fix-session-key-stem.sh`: default dry-run; explicit `--apply` for writes.
- `scripts/set-seat-default-models.sh`: operator-controlled preview/apply script.
- `scripts/restart-aegis.sh`: default becomes GPT PTU, overridable by environment.
- `scripts/seat-launch.sh`: fallback becomes the Step system model.

These are real changes to future operator/launcher behavior. None was executed.
The active session was not restarted or reconfigured; no model switch was requested.
No live database writes or application installation occurred.

`git diff --exit-code HEAD^ HEAD -- crates src-tauri ui infra` returned 0:
project-filtering implementation, frontend and schema are unchanged by this sync.

## Verification

All commands were run with fail-fast shell behavior; `NODE_OPTIONS` was unset only
for test child processes because its inherited preload path is missing.

- `cargo fmt --all -- --check`: pass.
- `cargo check --workspace --all-targets`: pass.
- `cargo clippy --workspace --all-targets -- -D warnings`: pass.
- `cargo test --workspace`: 278 passed, 0 failed; DB-dependent tests remain ignored.
  Core 168, CLI 3, MCP 81, waker 16, Tauri 10.
- `ui`: `tsc --noEmit`: pass; vitest 194 passed, 0 failed.
- `bash -n` on the four incoming scripts: pass. Syntax checks did not execute scripts.

## Old Flux verdict scope

Read the complete evidence document for `ev-UKSo1P85Leoi8bm0hRjfB`:
`FLUX-2026-10-17-project-filtering-verdict-v1.md`.
It pins Nimbus at `94daf79` and Mira at `21633ad`, passing each half separately.
Because this synchronization changes no product-filtering files, the prior verdict's
findings remain relevant to that unchanged implementation. This is not a new Flux
verification of the current commit, and the old verdict does not cover the incoming
operations scripts or their execution.

The verdict explicitly reports conflicts between the two halves in:
`useDataStore.ts`, `DispatchInboxView.tsx`, and `DispatchInboxView.test.tsx`.
No attempt to integrate Mira's branch was made here. Integrated behavior remains
unverified; passing each half does not establish that their combination works.

Preserved exclusions: rendered applications and operator clicks were not observed;
IPC-level `require_project_id` error text was not exercised; five other fallback
commands remain outside verdict scope. Artifacts still have an unscoped runtime
half, and Flux noted the missing on-screen disclosure. No click-level success is
claimed. The pending card is not treated as evidence of successful integration.

## Next action

Aegis must coordinate resolution of the two-half integration and commission its
verification. Operator acceptance remains pending. This delivery is synchronization
and regression testing only, not a release or an integration verdict.
