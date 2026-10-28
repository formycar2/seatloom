# Delivery: branch synced to `track` @ `6a54351`, the live blockers cleared, and per-module project support stated

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-17-project-filtering-synced-delivery-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-17 |
| version | v1 |
| dispatch | disp-RUnXBR3uICuWmmFizJ7vX |
| depends_on | MIRA-2026-10-17-project-filtering-ui-v1 |
| tags | project-filtering, branch-sync, live-blockers-cleared, delivery, mira |

---

## 0. The one-paragraph version

`packet/mira/knowledge-panel` synced to `track/infra-foundation` @ `6a54351` (27 commits,
clean merge), all gates re-run green, and — the thing that changes the operator's outcome —
**both live data defects that made the app unable to read the database at all are now
fixed**, so the project filtering can actually be seen. Per-module project support is
stated below rather than implied.

## 1. The sync, and what it brought

27 commits, clean merge, no conflicts. Two of them matter to this dispatch directly:

- `1b791a8` — `Pi` joins `enum Agent`
- `1549299` — `Pi` joins `enum Harness`
- `575d85d` — `ql_delivery`'s role `worker` was outside the `SeatRole` domain → `custom(worker)`
- plus the operator's harness script run, which set `default_harness='Pi'` on all 8 rows

Together those closed the whole-table read failure: `api.listSeats()` rejected because two
rows spelled `default_agent='Pi'` with no `Pi` in the enum, and `hydrateFromBackend`'s
eight-read `Promise.all` aborted — which is why the store kept its three fake project cards.

## 2. Measured: the read works now

Ran the repo's own probe (`crates/seatloom-core/examples/probe_seats.rs`) against the live
database, with the current source:

```
OK: 8 seats
  seat-aegis-001: agent=Some("ClaudeCode")(in-domain=true) harness=Some("Pi")(in-domain=true)
  … all 8 rows, every one in-domain=true
```

So `list_seats` succeeds on live today. The blocking condition my earlier delivery reported
is gone.

**One caveat, measured rather than assumed:** the main checkout's
`target/debug/seatloom-mcp-emit` is still the Oct 16 12:58 build — pre-Pi-fix. I rebuilt it
in my worktree to prove the current source reads. **The operator must rebuild and reinstall
before the running app changes** — the same 2026-10-10 lesson: bytes on disk are not the
process that is running.

## 3. Per-module project support, stated plainly

The dispatch required this list, and required that I not pretend a module filters when its
backend does not support it.

| module | filtered by project? | how | backend accepts `project_id`? |
|---|---|---|---|
| seats | yes | role binding, in the store | **no** — `cmd_list_seats` takes only `State` |
| work items | yes | the row's own `projectId`, in the store | **no** |
| handoffs | yes | the row's own `projectId`, in the store | **no** |
| sessions | yes | the row's own `projectId`, in the store | **no** |
| dispatches | yes | **at the call site** | **yes** — `cmd_list_dispatches` |
| documents | yes | the row's own `projectId`, in the store | yes |
| runtime artifacts | **no** | **not filterable** | **no** — `ArtifactDto` has no `project_id` |
| Continuity delta (Supervisor IM) | n/a — global by design, and says so | — | — |

**So four modules are filtered at the store rather than at the call, because their backend
commands take no project argument.** That is not a workaround: the store is the layer that
already owns per-project shaping, and the row's own `projectId` is the fact being filtered
on. It is, however, a different layer than the dispatch's "every backend call takes the
project" phrasing, so it is stated rather than glossed.

**The one module that genuinely does not filter** is the runtime artifacts — `ArtifactDto`
carries no `project_id`. A test pins that asymmetry so it cannot later be read as an
oversight.

The Continuity delta in the Supervisor IM reads globally by design (it answers "what changed
since the last checkpoint", which is not a per-project question) and **already says so** in
its own copy: with no supervision checkpoint it cannot give a "since when", so it shows the
full recent picture. So it is not a module silently showing another project's data.

## 4. The no-project and hydration-failed guards

Both still hold after the sync, and are pinned by tests:

- `DispatchInboxView` — no active project, or `backendHydrated === false` → a notice saying
  no project is chosen yet and that what follows is not the operator's dispatches, with the
  reason and a pointer to the red notice above. It does **not** fall back to the backend's
  default project.
- `WorkItemsView`, `TimelineView` — no active project → "please choose a project" (matching
  `SeatsView`, which already had it). Both previously rendered a silent empty list that read
  as "this project has nothing".
- `HydrationFailure` (the red notice) — unchanged, still renders the backend's own error text
  verbatim.

## 5. Gates, re-run after the merge rather than assumed

| Gate | Result |
|---|---|
| `tsc --noEmit` | clean |
| `vitest` | **199/199** (21 of them the project-filtering and inbox-guard tests) |
| `vite build` | green |
| `cargo test --workspace` | **275 passed / 0 failed** |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 issues |
| `cargo fmt --all -- --check` | 0 diffs |
| `probe_seats` against live | OK: 8 seats, all in-domain |
| `lint-changed.sh` | 6 files, **all zero-delta** (below) |
| `verify-docs.sh` | this note clean |

Lint measured both ways in a `track` worktree at the real paths: `useDataStore.ts` 2/2,
`DispatchInboxView.tsx` 0/0, `WorkItemsView.tsx` 3/3, `TimelineView.tsx` 10/10,
`SeatsView.tsx` 3/3, `SupervisorIntent.tsx` 0/0 — **zero new**.

## 6. What is not proven

Nobody has looked at the running app. The live read is proved through the repo's own probe,
and the filtering through tests that drive the real hydration function with project-tagged
rows; but "switch to SeatLoom and see 7 seats, switch to ql_delivery and see 1" has not been
observed, because the app binary in use predates these fixes. That is the acceptance card's
job.

## 7. Boundaries

No backend signature change (Nimbus's). No live-database write. No gate event. No merge into
`track`. `ui/src/lib/api.ts` and `ui/src/lib/types-dto.ts` untouched.

## 8. Artifact index

- Branch: `packet/mira/knowledge-panel`, containing `track/infra-foundation` @ `6a54351`.
- Ledger: see `emit_event(ArtifactCreated)`.
- This delivery note.
