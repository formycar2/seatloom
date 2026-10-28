# Delivery: the interface stops asserting things that are not true

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-16-wiring-honesty-ui-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-16 |
| version | v1 |
| dispatch | disp-wiring-honesty-ui |
| depends_on | LYRA-2026-10-16-next-round-plan-v1 |
| tags | wiring-honesty, provenance, dead-click, write-only-forms, delivery, mira |

---

## 0. The one-paragraph version

Five items, each measured before changing anything. The timeline's badge no longer claims
SAMPLE above ~600 real captured events; a clicked timeline row now opens a detail instead
of doing nothing, and that detail says plainly that the full lineage pane does not exist
yet; the knowledge panel's hand-written constant was already replaced last dispatch with a
measured probe, and the remaining half is blocked on a capability that is not in this tree;
four write-only forms now say at save time that this build cannot store the input and name
the missing command; and the shortcut layer was not touched.

## 1. Item 1 — the timeline badge was false

**Measured.** `provenance.ts`'s `LIVE_TABS` is `{live-seats, sessions, dispatches}` and
`LEDGER_TABS` did not exist, so `provenanceForTab('timeline', …)` returned
`backendHydrated ? 'seed' : 'fixture'` — badge SAMPLE / MOCK, headline "SAMPLE DATA — not
yet from live capture". But the timeline reads `projectData[activeProjectId].events`,
which `useDataStore.refreshEvents()` fills from `api.listEvents(400)`, and `App.tsx`
polls it every 8 s. Those rows are real captured ledger events, including ones other
processes append through MCP. The badge was false in the unsafe direction.

**Fixed.** A new `'ledger'` provenance, a `LEDGER_TABS` set, and
`provenanceForTab` returning `backendHydrated ? 'ledger' : 'fixture'` for the timeline.
The headline says "LIVE — read from the canonical ledger", the detail names the read and
the 8-second cadence, and it says what would make it stop being live (the backend
becoming unreachable — which `backendHydrated` already reports, so it falls back to
`fixture`, never to `ledger`). `LIVE_TABS` and `LEDGER_TABS` are asserted disjoint.

Seven new tests pin it, including that the mirror tabs, the dispatch tab, the seed-backed
tabs and the two discovery-failure states are all unchanged.

## 2. Item 2 — the dead timeline click

**Measured.** `SelectedObjectType` (`ui/src/types/index.ts`) had no `'Event'`, so
`TimelineView`'s `onSelectObject('Event', event)` produced a selection that fell to
`MasterDetail`'s `default: return null` → `hasSelection` false → no detail pane at all.

**Fixed.** `'Event'` added to the type, `selectedObject` carries the whole event through,
and a new `EventDetail` renders what the event itself carries: type, when, who (the
`ActorRef` is a union, so it is rendered rather than assumed a string), the event id, and
the payload verbatim.

**What it deliberately is not.** It is not the lineage pane. `lineage` is served only by a
binary built from `packet/nimbus/gate-vs-announcement` and is absent from this tree and
from `track` — measured five ways earlier for `disp-lineage-ui`: 22 tools and no `lineage`
on both binaries, no `cmd_lineage` in `src-tauri`, no lineage entry or DTO in
`api.ts` / `types-dto.ts`. So `EventDetail` carries an explicit block saying the full
story of this event is not readable yet, naming what is missing. It does not pretend, and
the click is not a dead end. Coordination went out as
`SupervisorMessage ev-XoyLVdbQi9LTKRgM4dASS`; when the pane lands, `EventDetail` is its
entry point and only that block is replaced.

## 3. Item 3 — the third hand-written constant

**Already done last dispatch.** `KnowledgeSection.tsx`'s `KNOWLEDGE_CMDS_REGISTERED = true`
was deleted and replaced with `useKnowledgeWiring`, a measured probe in the shape
`SupervisorIntent` already uses in-tree: three states, no fourth "assume it works". Its
comment had claimed it was already proven, but what was proven was a code-landed fact, not that this
build can call the commands.

**The remaining half is blocked, and I did not work around it.** Nimbus's
`ev-wiring-honesty-backend-landed` is real and measured, but `capability.ts` and the
`api.ts` wrapper are on his branch, not in this tree and not on `track` — and
`ui/src/lib/api.ts` is his fence. So `AcceptanceSection` cannot consume his probe. The
alternative was to write my own second probe, which is the same question answered twice —
exactly what the dispatch forbids. The rule I applied is the one I recorded for
`disp-lineage-ui`: a fence workaround is legitimate only for a **missing wrapper**, never
for a **missing module**. Reported to Aegis rather than solved.

## 4. Item 4 — four write-only forms

**Measured.** All four make zero `api.*` calls, and `main.rs` registers no create command
for workitem / handoff / seat / delegation. They accept input, imply success, and the data
is gone on reload — while the button says "create and write".

**Fixed, the second of the dispatch's two options.** A shared `CannotStoreNotice` renders
above each form's button row: this build cannot store it, closing the window discards what
you just typed, the missing command is named exactly
(`cmd_create_workitem` / `cmd_create_handoff` / `cmd_create_seat` /
`cmd_create_delegation`), and what is missing is a backend command rather than a button.
The local store write is not removed — that would change what the form does beyond
honesty — so the notice makes the truth visible at the moment it matters instead of
leaving a middle state where it looks saved. Four tests pin it, including that the notice
never claims saved / created / written.

## 5. Item 5 — the shortcut layer

Not touched. `App.tsx:139` and `useGlobalShortcuts.ts:24` both fire on plain ⌘K; that is a
live defect recorded in the plan, and the chord belongs to the operator with the ruling
still open with Aegis. Zero `metaKey` / `ctrlKey` lines in this dispatch's diff.

## 6. Gates

| Gate | Result |
|---|---|
| `tsc --noEmit` (whole tree) | clean |
| `vitest` | **174/174** (11 new: 7 provenance, 4 the notice) |
| `vite build` | green |
| `cargo test --workspace` | 251 passed / 0 failed (no Rust changed) |
| `scripts/verify-docs.sh` | this note clean |

**Lint, measured both ways rather than asserted.** In a `track` worktree at the real paths
versus my tree, for every file this dispatch changed:

| file | track | mine |
|---|---|---|
| `src/lib/provenance.ts` | 0 | 0 |
| `src/types/index.ts` | 2 | 2 |
| `src/App.tsx` | 15 | 15 |
| `src/components/WorkItemForm.tsx` | 0 | 0 |
| `src/components/HandoffForm.tsx` | 1 | 1 |
| `src/components/AddSeatDialog.tsx` | 1 | 1 |
| `src/components/DelegationOverlay.tsx` | 2 | 2 |

**Zero new.** One transient regression was caught and fixed during the work: my first
draft of the `SelectedObject` type used `event?: any`, which took `App.tsx` from 15 to 16;
it is now typed as `CanonicalEvent`. `lint-changed.sh` still names these files because its
rule is "changed files must be clean" and they were already red on `track`; closing
pre-existing errors is outside this dispatch's boundary, which says to leave
already-dirty files exactly as dirty.

## 7. What is not proven

Nobody has looked at the rendered surfaces. The badge text and the notice are pinned by
tests that render the real components; the dead-click fix is proven by the type, the
selection path and the detail renderer existing — but whether a clicked row reads as an
answer rather than as a dead end with an apology is the operator's judgement. His click is
the first look.

## 8. Boundaries

No skeleton re-layout. `ui/src/lib/api.ts` and `ui/src/lib/types-dto.ts` untouched. No
backend change. The shortcut layer untouched. No merge, no live-database write, no gate
event emitted as a measurement.

## 9. Artifact index

- Branch: `packet/mira/knowledge-panel`.
- Ledger: `ev-wiring-honesty-ui-landed` (ArtifactCreated, announcement — no `verified`
  field), and `SupervisorMessage ev-XoyLVdbQi9LTKRgM4dASS` for the lineage-pane
  coordination.
- This delivery note.
