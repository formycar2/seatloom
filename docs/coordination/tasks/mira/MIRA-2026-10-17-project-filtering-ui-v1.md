# Delivery: switching a project now filters, and an unresolved project says so

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-17-project-filtering-ui-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-17 |
| version | v1 |
| dispatch | disp-d_N8MKRKAwj7492ZVwCGN |
| tags | project-filtering, silent-fallback, delivery, mira |

---

## 0. The one-paragraph version

Mr. Zhang reported that switching projects filtered nothing. The rendering was already
per-project; what was global was the **content** — the hydration loop put every row under
every project. Every list whose DTO carries `projectId` is now filtered by it in that loop,
the dispatches inbox now passes the active project to a backend call that always accepted
one, and the three views that silently rendered an empty list with no project resolved now
say "please choose a project".

## 1. What I measured before changing anything

The dispatch named `api.ts` as the root cause. Confirmed, and then measured further, because
the boundary says `api.ts` is Nimbus's fence:

| wrapper | accepts a project? | backend command accepts `project_id`? |
|---|---|---|
| `listSeats` | no | **no** (`cmd_list_seats` takes only `State`) |
| `listWorkitems` | no | **no** |
| `listHandoffs` | no | **no** |
| `listSessions` | no | **no** |
| `listDispatches(projectId?, limit?)` | **yes** | **yes** |
| `listDocuments({projectId?})` | yes | yes |

So there were two different defects, not one:

1. **`DispatchInboxView` never passed a project** to a call that has always accepted one —
   a pure call-site bug, fixed here, no fence touched.
2. **The store's hydration put every row under every project** — which is why the other
   modules did not filter even though they render `projectData[activeProjectId]`.

## 2. What changed

**`useDataStore.hydrateFromBackend`** — each list is filtered by the row's own
`projectId` before the mapper, inside the per-project loop. Measured which DTOs carry it:
`WorkItemDto`, `HandoffDto`, `SessionDto`, `DocumentDto`, `CanonicalEventDto` all do.
`ArtifactDto` does **not**, so the runtime artifacts stay global — reported, not guessed at.

**`DispatchInboxView`** — `api.listDispatches(activeProjectId ?? undefined)`, and
`activeProjectId` added to the `load` callback's deps so switching projects reloads instead
of showing the previous project's rows until the next 5 s poll.

**`SeatsView` already had the guard; `WorkItemsView` and `TimelineView` did not** — both
rendered a silent empty list that reads as "this project has nothing". Both now say
"please choose a project".

**`DispatchInboxView` guard** — with no project resolved, or with the hydration read failed,
it says "no project chosen yet" rather than rendering the backend's default project's rows. That
silent fallback was the defect.

## 3. A bug I introduced and caught in the same pass

My first cut put the `WorkItemsView` guard **above** a `useMemo`, which eslint flagged as
`react-hooks/rules-of-hooks` — a hook after a conditional return. Fixed by deriving
`workItems` and `filteredItems` before the guard. Recorded because the guard was the point,
so "just move the guard down" was not available: the derivation had to move up instead.

## 4. What is NOT filtered, stated plainly

| module | filtered now? | why |
|---|---|---|
| seats | yes — by role binding (previous dispatch) | `project_role_bindings` |
| work items / handoffs / sessions / timeline | yes — by `projectId` | the DTOs carry it |
| documents (the 291 coordination docs) | yes — by `projectId` | `DocumentDto` carries it |
| runtime artifacts (seed/001) | **no** | `ArtifactDto` has no `project_id` — schema gap |
| dispatches | yes — by the call site | the backend accepts it |
| live seats | n/a — tmux discovery | not a database read |

The runtime artifacts appear under every project. A test pins that asymmetry so it cannot
later be read as an oversight.

## 5. The fence, and why I did not touch it

`api.ts`'s four wrappers take no project argument, and their backend commands do not accept
one. Adding either is Nimbus's work (the dispatch says so, and every recent dispatch repeats
that `api.ts` and `types-dto.ts` are his fence). So those four modules are filtered at the
**store** level rather than at the call, which achieves the operator's outcome without a fence
edit — and the store is the layer that already owns per-project shaping.

## 6. Gates

| Gate | Result |
|---|---|
| `tsc --noEmit` | clean |
| `vitest` | **199/199** (7 new: 4 for the per-project filtering, 3 for the inbox guard) |
| `vite build` | green |
| `cargo test --workspace` | 271 passed / 0 failed (no Rust changed) |
| `lint-changed.sh` | 4 files, **all zero-delta** (below) |
| `verify-docs.sh` | this note clean |

Lint measured both ways in a `track` worktree at the real paths: `useDataStore.ts` 2/2,
`DispatchInboxView.tsx` 0/0, `WorkItemsView.tsx` 3/3, `TimelineView.tsx` 10/10 — zero new.

Two of my own test-authoring errors were caught and fixed on the way: a helper that
overwrote each row's `projectId` with `''` (so the filter had nothing to match), and a
`cleanup` import from vitest that is a testing-library export. Both are recorded because a
test that passes for the wrong reason is worse than no test.

## 7. What is not proven

The live database cannot be read right now (`default_harness='Pi'` on all 8 seat rows is
outside the `Harness` domain, so `list_seats` throws and hydration aborts), so nothing here
has been observed against live rows. The filtering is proved by tests that drive the real
hydration function with project-tagged rows; whether the operator sees SeatLoom's 7 seats and
`ql_delivery`'s 1 is the acceptance card's job, and it is still blocked on that live defect.

## 8. Boundaries

No backend signature change (Nimbus's). No live-database write. No gate event. No merge.
`ui/src/lib/api.ts` and `ui/src/lib/types-dto.ts` untouched.

## 9. Artifact index

- Branch: `packet/mira/knowledge-panel`.
- Ledger: see `emit_event(ArtifactCreated)`.
- This delivery note.
