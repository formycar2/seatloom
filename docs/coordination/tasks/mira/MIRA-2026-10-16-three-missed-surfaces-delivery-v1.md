# Delivery: three surfaces Lyra's plan listed but the dispatches missed

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-16-three-missed-surfaces-delivery-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-16 |
| version | v1 |
| dispatch | disp-qR65V3mi0m-4yYl4J8mmk |
| depends_on | LYRA-2026-10-16-next-round-plan-v1 |
| tags | goals-stages, dependency-column, wiring-probe, delivery, mira |

---

## 0. The one-paragraph version

Three surfaces, all measured before changing anything. The goals/stages bands now say
plainly that they have **no data source** rather than rendering an empty shell from a
mock key that can never match a real project. The dispatch dependency column's tooltip
is now one plain sentence per dependency instead of raw `dep_kind: dep_value`, with the
read's bound stated so absence is never read as proof. And the knowledge panel's
hand-written `= true` wiring constant is gone, replaced by a probe that actually calls
the command. Item 4 (the knowledge panel has no entry in the main navigation) was not
dispatched and was not touched.

## 1. Item 1 — goals and stages: no source, said out loud

**Root cause, verified.** `ProjectDashboard.tsx` read
`MOCK_CHANNEL_DATA[channelId]`. `MOCK_CHANNEL_DATA`'s only keys are `ch-p1`, `ch-p2`,
`ch-p3`. `channelId` is built by `useContacts` as `'ch-' + p.id` from the **real**
`projects` rows (`cmd_list_projects`), so for every project that exists the lookup is
`MOCK_CHANNEL_DATA['ch-seatloom']` → `undefined` → the fallback's empty arrays. The
section rendered a heading, an empty overall-goals row, and a roadmap title with an empty
stage list.

**Then I checked whether there was a real source to wire to. There is not.** Grep across
`src-tauri`, `crates/seatloom-core`, every file in `infra/postgres/schema/`, and
`ui/src/lib/api.ts`: no goals column, no stages column, no DTO, no Tauri command, no api
entry. So this was never a rewiring job — it is a "say what is missing" job, and faking a
band to make it non-empty is exactly the shape the dispatch forbids.

**What changed.** `ProjectDashboard` no longer sources `goals` / `currentGoal` / `stages`
from the mock; they are explicitly empty with a comment recording the measurement.
`GoalsSection` gained a first-class state for "no source": it says these two bands have no
data source — **not** "you have no goals" — names what is missing (a store, a command, an
api entry), says what happens next, and points out that the blockers / active work /
timeline above it are real and unaffected.

**Reported rather than quietly changed:** the same dead mock read also feeds the DAG
scaffolding and the timeline below (`currentStage.workflow`, `nextStep`,
`workItemsDone`/`Total`). Those have real counterparts in `truthData` and are outside this
dispatch's scope, so they were left alone and are named here.

## 2. Item 2 — the dependency column: a sentence, with the bound stated

**Before.** The cell showed `↑N`; the tooltip was
`deps.map(x => \`${x.depKind}: ${x.depValue}\`)` — so a hover read
`event: ev-history-sessions-read-live`. The expanded row's dependency meta line had the same raw
shape. He could see *that* something was blocked, not *what on*.

**What it can and cannot measure.** `api.listEvents(limit)` is the only read available,
and it is **bounded** — the repository function is
`... ORDER BY occurred_at DESC LIMIT $1`. The unbounded existence answer lives in the MCP
`get_dispatch` tool, which is **not** registered as a Tauri command in this tree (measured:
no `cmd_get_dispatch` anywhere in `src-tauri`). So:

- a referent found in the newest 500 rows → one plain sentence, produced by the **same
  translator** the Supervisor feed uses (`eventToText`), so the two surfaces cannot drift;
- a referent **not** in the window → a line saying it is not among the newest 500 ledger rows
  and that this view therefore cannot tell what it is or whether it exists. It never says
  "does not exist" and never says "satisfied";
- a `dispatch`-kind dep → "waiting on another dispatch (id)", because the row's own seat/status columns
  already say what it is.

The window (500) is stated wherever the answer is shown. Measured: both currently-live
dep referents (`ev-history-sessions-read-live`, `ev-lineage-readmodel-live`) are inside
the newest 200, so the common case resolves.

**A real gap this surfaced.** A `-live` gate event is a `SupervisorMessage` whose payload
has `statement`, not `content`. `eventToText` passed through `payload.content` only, so a
gate rendered as an empty-message marker while carrying a whole statement. Fixed in the translator:
`content` still passes through byte-for-byte (the acceptance criterion that must not
regress), and a content-less `SupervisorMessage` now falls back to its `statement`,
labelled as a gate statement.

## 3. Item 3 — the knowledge panel's hand-written `true`

`KnowledgeSection.tsx` had `const KNOWLEDGE_CMDS_REGISTERED = true`. Its comment claimed
it was "already proven", but what was proven is a **code-landed** fact — Nimbus's branch merged,
`main.rs` registers the two commands. Nothing had ever checked that this build can
actually call them. That is the shape the dispatch named.

**Nimbus's wiring-honesty probe is not in the tree** (measured: nothing matching in
`src-tauri` or `api.ts`), so rather than wait or add a fourth hand-written constant, the
constant was **deleted** and replaced with `useKnowledgeWiring` — the same measured-probe
shape `SupervisorIntent` already uses in-tree: mount, call `cmd_get_knowledge_tree` once,
render backend-reachable (measured) / backend-unreachable (measured) with the measured error
text verbatim. Three
states, no fourth "assume it works". When Nimbus's probe lands, this hook swaps to consume
his.

## 4. Gates

| Gate | Result |
|---|---|
| `tsc --noEmit` (whole tree) | clean |
| `vitest` | **163/163** (10 new: 3 dependency-column, 1 probing state, plus the reworked knowledge-gate test) |
| `vite build` | green |
| `cargo test --workspace` | 251 passed / 0 failed (no Rust changed) |
| `scripts/lint-changed.sh` | **zero new issues**, measured below |
| `scripts/verify-docs.sh` | this note clean |

**The lint number, measured both ways rather than asserted.** In a `track` worktree at the
real paths: `ProjectDashboard.tsx` **33 errors**, `GoalsSection.tsx` 0,
`KnowledgeSection.tsx` 0, `DispatchInboxView.tsx` 0. In my tree, same files: 33, 0, 0, 0.
**33 before, 33 after — zero new.** Both unused imports (`StageWorkflow`, `TimelineEntry`)
were already unused on track. `lint-changed.sh` still reports the file, because its rule
is "changed files must be clean" and that file has been red since before this dispatch;
closing those 33 pre-existing `any`s is a refactor of a file this dispatch barely touches,
which the "do not re-layout the skeleton" boundary does not cover. Reported rather than
done silently.

## 5. Not done, on purpose

**Item 4 — the knowledge panel has no entry in the main navigation.** The dispatch says
Lyra flagged it, that navigation is the operator's call, and that Aegis will ask him
first. Not touched, and no entry was rendered that does nothing.

## 6. What is not proven

Nobody has looked at the rendered surfaces. The goals/stages state and the dependency
tooltip are covered by tests that mount the real components with mocked reads; what is
unverified is the operator's actual reading of "no data source, which is not the same as no
goals" — whether that
sentence lands as an explanation or as an excuse. His click is the first look.

## 7. Boundaries

No skeleton re-layout. No backend change (no command added, changed, or newly called
beyond `cmd_list_events`, which was already registered and already wrapped in `api.ts`).
No other view changed. No fake data anywhere: the goals bands are empty *and say why*,
and the dependency column says when it cannot answer.

## 8. Artifact index

- Branch: `packet/mira/knowledge-panel`.
- Ledger: see `emit_event(ArtifactCreated)`.
- Acceptance card: requested with a plain-language plan.
