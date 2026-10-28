# Delivery: the project-filtering acceptance card is open — and the branch carries more than one thing

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-17-project-filtering-card-opened-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-17 |
| version | v1 |
| dispatch | disp-QK9JQlso-SEVwemhnIZ_R |
| depends_on | MIRA-2026-10-17-project-filtering-synced-delivery-v1 |
| tags | acceptance-card, project-filtering, whole-branch-merge, delivery, mira |

---

## 0. The one-paragraph version

`acc-disp-RUnXBR3uICuWmmFizJ7vX` is open, citing `ev-UKSo1P85Leoi8bm0hRjfB` which I
verified myself names this dispatch and concludes pass. Four preconditions are written
into it rather than smoothed over — the two-halves split, the artifacts gap, Flux's own
declared-unverified parts, and the fact that **this branch carries more than one thing,
so accepting it merges all of them together**.

## 1. The verdict, verified rather than taken on trust

Read out of the event, not from the dispatch's prose:

| Check | Result |
|---|---|
| `ev-UKSo1P85Leoi8bm0hRjfB` exists, `ReviewVerdictIssued`, `seat-flux-001` | ✅ |
| `payload.verdict` | **pass** |
| names `disp-RUnXBR3uICuWmmFizJ7vX` via its own `event_object_refs` row | ✅ |

Its `payload.dispatch_id` is a different id (`disp-8jqnVUeRz1AZ4nVWrUJVE`) — the dispatch
that asked for the verdict — which is not a rejection: that field carries one value and the
per-dispatch refs are the channel that carries several.

Also read `does_not_assert` and `residual_findings`; both are now in the card.

## 2. The branch-sync, and why it was needed first

`packet/mira/knowledge-panel` was behind `track` by one commit (`eaee65b`) when the
dispatch arrived, and the acceptance gate refuses a branch that does not contain its base.
Merged, clean, no conflicts. Track is now an ancestor.

## 3. Four things written into the card, not smoothed over

1. **The two halves.** Mine is the store layer (`projectId` on the row; seats by role
   binding) plus the dispatches call site. Nimbus's `project-scoped-reads` is **not**
   merged, so `cmd_list_seats` / `cmd_list_workitems` / `cmd_list_handoffs` /
   `cmd_list_sessions` still take no `project_id` — measured in the tree. So four modules
   filter at the store, not at the call.
2. **The artifacts gap.** `ArtifactDto` has no `project_id`, so the artifacts tab shows the
   same unscoped rows under every project. That is a schema fact, not an omission.
3. **Flux's declared-unverified parts.** She does not assert the rendered app (her screen is
   locked; the click-level acceptance is the operator's), does not assert IPC-level error
   text, and did not touch the live database. So nothing here has been seen running.
4. **The whole-branch point.** This branch carries project filtering, the three corpus
   screens, the app icon, the pending-acceptance count, the dependency column and the
   wiring-honesty pass. Accepting this card merges **all** of it. That is one decision, not
   seven, and the card says so in the first precondition instead of leaving it to be
   discovered at merge time.

## 4. Evidence, restated for the card

- live read works: the repo's own `probe_seats` against live with current source returns
  OK: 8 seats, all in-domain (measured in the previous delivery);
- filtering: tests drive the real hydration function with project-tagged rows and prove
  work items / handoffs / sessions / events / documents land under their own project only;
- guards: no active project or a failed hydration renders an explicit notice, never the
  backend's default project's rows;
- gates: tsc clean, vitest 199/199, vite build green, cargo test 275/0, clippy 0, fmt 0;
  lint zero-delta across six files measured in a `track` worktree.

## 5. What is not proven

Nobody has looked at the running app; the operator's click is the first look, and the card
says so. Also noted there: the main checkout's `seatloom-mcp-emit` was the pre-Pi-fix build,
so a rebuild and reinstall are needed before the running app changes.

## 6. Boundaries

No live-database write. No gate event. No merge. `ui/src/lib/api.ts` and
`ui/src/lib/types-dto.ts` untouched.

## 7. Artifact index

- Branch: `packet/mira/knowledge-panel`, containing `track/infra-foundation` @ `eaee65b`.
- Ledger: `AcceptanceRequested`, and `emit_event(ArtifactCreated)`.
- This delivery note.
