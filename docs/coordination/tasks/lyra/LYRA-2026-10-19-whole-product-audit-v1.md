# SeatLoom whole-product capability audit

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-19-whole-product-audit-v1 |
| status | delivered |
| author | lyra |
| date | 2026-10-19 |
| version | v1 |
| tags | product-audit, capability-map, end-state, decision-surface |
| depends_on | disp-ZsT2wt_em9pdTQLWmHOZQ, docs/coordination/ROADMAP.md |

## Executive judgment: one product, not eleven modules

SeatLoom's coherent product promise is: **give the team work, see what happened, decide what only
I can decide, and recover the evidence later**. The existing interface already contains the parts
of that loop. Its main weakness is not lack of screens: it offers several competing accounts of
progress, with different underlying objects and different claims of authority.

The dispatch page describes governed work. The supervision dashboards describe legacy workitems,
sessions and handoffs. The supervisor panel mixes actual messages with a ledger feed and a mock
cross-project dashboard. History describes archived conversations; knowledge describes distilled
memories. Documents and Playbook describe files. These are useful layers of ONE work story, but the
product currently leaves the user to reconcile them.

**Disposition:** keep the structured work/evidence/decision loop; repair its truth and continuity
before adding capability; consolidate duplicated presentations without redesigning the skeleton;
retain the terminal mirror as a detail instrument, not the product's primary face. Do not delete a
reachable component merely because its directory is named `app-v2`.

## 1. Measurement boundary and corrections to the supplied facts

Baseline: clean integration worktree `track/infra-foundation@cfda7407e2a8fe2b4148deff62515539dc51f12a`,
read on 2026-10-19. File references below are relative to that repository and line numbers refer to
this baseline. No application click/IPC proof was performed. **Usable** below means a reachable
code path with a real data source, not a claim of operator acceptance or current installed-build
verification. The recent ledger records the combined filtering acceptance at 08:10:35 UTC;
that establishes merge history, not rendered-app behaviour.

Three corrections matter to the product judgment:

1. **Sessions is not a missing view.** `App.tsx:426-428` renders `SessionsWorkspace`;
   `app-v2/panel/SessionsWorkspace.tsx:18,24,93` renders history or `TmuxMirrorWorkspace`.
   `HistorySessionsPanel.tsx:78-83` calls real history APIs. `components/SessionDetail.tsx`
   describes runtime sessions, not the same object as an archived conversation. File placement
   alone proves neither absence nor a need to merge those object types.
2. **Most of app-v2 is reachable, not dead.** A static transitive relative-import traversal from
   `ui/src/main.tsx` and `ui/src/supervisor/main.tsx` reaches SupervisorPanel, both dashboards,
   DagWorkflow, all dashboard sections, the chat components, SupervisorIntent, history and mirror
   workspaces. `SupervisorPanel.tsx:521-546` actually renders the dashboards and intent surface;
   `ProjectDashboard.tsx:646,835` mounts DagWorkflow and KnowledgeSection. Only `AppV2.tsx` and
   its `views/DocumentsWorkspace.tsx` are unreachable TSX production candidates in this subtree
   under this traversal. Tests are excluded. Dynamic/external consumers must be checked before
   deleting candidates. The claimed roughly 25 dead files is not supported.
3. **Timeline has time information and upstream ordering.** `EventRow.tsx:57` renders the event
   timestamp; `repositories.rs:409,442` queries events in descending occurred_at order. The view
   does not group them or impose its own sort, but absence of a local sort is not absence of time.
   `TimelineView.tsx:26,32-44,60-63` does retain an unused templateFilter; importantly, no template
   selector is rendered, so this is dead state, not evidence of a visible non-working selector.

Supplied live counts (208 dispatches, all seatloom; 39 projects; ql_delivery zero dispatches) are
**Aegis's snapshot, not re-measured in this audit**. They must not be turned into permanent
product invariants. No raw database access was needed for this review.

## 2. The capability map: eleven entries, one user journey

| User question in the whole loop | Existing tab / entry | Actual source and role | Relationship / end-state judgment |
|---|---|---|---|
| Which project am I considering? | all-projects + project switcher | `AllProjectsView.tsx:10,85-86`; real project catalog, lazily hydrated per-project data | Keep project context. Catalog presence does not prove active work. Two displays of selection should share identity, not be counted as two capabilities. |
| What is the team doing and what is blocked? | dispatches (default, `App.tsx:106`) | `DispatchInboxView.tsx:644-685`; selected-project dispatch query and per-seat grouping | Core end-state work view. Includes acceptance decisions on the same page. Not a cross-project fleet summary today. |
| What should I decide? | AcceptanceSection inside dispatches | `AcceptanceSection.tsx:103,172-241`; scoped acceptance rows and real accept/reject commands | Closest existing decision surface. It covers delivery acceptance, not all product rulings or operator-only prerequisites. Keep this loop and expand its truthful coverage, not a new decision application. |
| How is progress? | dashboard (SupervisionDashboard) | `App.tsx:429`; `SupervisionDashboard.tsx:37-64`; workitem/session/inbox projections | Duplicates dispatch progress but counts different objects. Keep only a reconciled readout in the end state; present implementation needs repair/consolidation. |
| What requires attention? | inbox | `InboxView.tsx:21-28`; store-derived workitem/handoff rows, not acceptances | Duplicates a subset of dashboard/action queues. End-state attention capability stays; independent competing inbox meaning should be consolidated. |
| What is the unit of work? | workitems | `WorkItemsView.tsx:21-32`; project-scoped workitems | Keep semantic work objects distinct from dispatch instructions, but do not present workitems as a complete account of governed work until linked. Creation is not durable today. |
| Who belongs here? | seats | `SeatsView.tsx:41,57-78`; project bindings and explicit unbound context | Keep ownership/context, not a second terminal browser. Good explanation of unbound seats; membership is not measured execution or availability. |
| What actually happened? | timeline | `TimelineView.tsx:30-44,131-160`; canonical event list, EventDetail | Keep the audit/replay entry. Currently a recent event list, not a complete replay. Competes visually with the supervisor feed but should share event meaning. |
| What was produced? | artifacts | `ArtifactsView.tsx:25,80`; scoped documents plus unscoped runtime artifacts | Keep evidence browsing. Explicit global-artifact notice is good; global outputs are not automatically this project's products. |
| What can be reused? | playbook | `PlaybookView.tsx:16-18`; artifacts filtered to template T7 | Current implementation is a governance-document subset, not a published/versioned capability library. Merge this browsing meaning with evidence; retain the future reuse capability without claiming it exists now. |
| What did I say and why did we do that? | sessions | `SessionsWorkspace.tsx:24,93`; history list, readable contexts and source paths, plus mirror mode | Keep archived conversation replay. Runtime SessionDetail is a different zoom layer, not a missing SessionsView. History-to-memory/work linkage still incomplete. |
| What is the raw process doing? | live-seats | `LiveSeatsView.tsx`; tmux attach, SessionTerminal, durable mirror store | Transitional as primary face; survives in end-state ONLY as raw terminal detail/diagnosis. Do not delete the working mirror. |
| How do I instruct/review the team? (cross-cutting, not a twelfth tab) | Cmd+K supervisor panel | `SupervisorPanel.tsx:266-285,521-567`; messages, intent, project/global dashboards | Keep structured input and conversation alongside the first screen, not as a competing second application shell. |

The map reveals the gap: **execution → decision → evidence is real but disconnected; several
screens say “progress” while observing different ledgers**. The cost to the user is reconciliation:
read dispatches for execution, a different dashboard for workitems, the supervisor feed for rulings,
then history/knowledge/documents for evidence. A page with zero workitems can coexist with a busy
dispatch queue without either read failing. That is semantic incompleteness, not a DB outage.

## 3. Three-way disposition, with evidence

### A. Usable and sensibly designed: retain as the product's spine

- **Scoped dispatches and acceptances.** Selected project flows into `api.listDispatches` and
  `api.listAcceptances`; guards clear rows on unresolved context. Grouped status, expansion,
  reuse and the rejection reason support an actual work loop (`DispatchInboxView.tsx:644-685`,
  `AcceptanceSection.tsx:103,139-140,447`). Keep; verify installed build separately.
- **History and source evidence.** `HistorySessionsPanel.tsx:78-83,533-537` dereferences contexts
  rather than calling pointer-only data a readable feature. Source paths/line references and
  missing-state handling are the right replay foundation. Keep, with archive privacy limits.
- **Project membership with unbound explanation.** `SeatsView.tsx:57-75` differentiates no binding
  from no seats in the system. Keep this semantics.
- **Explicitly qualified evidence scope.** `ArtifactsView.tsx:80` explains that runtime artifacts
  are global while documents are project-filtered. Keep the qualification until ownership is
  genuinely recorded; do not guess it from a file name.
- **Knowledge read/disclosure.** `KnowledgeSection.tsx:111-149` probes the real commands and reads
  progressively. Reachable via the project dashboard, not dead. Keep this capability; discoverability
  and unified provenance remain category B problems.

### B. Usable but badly designed or misleading: repair/consolidate

- **Two progress authorities.** Default dispatches tracks dispatch state; SupervisionDashboard
  counts workitem/session state (`:46-64`). Its “pending decisions” counts `ACTION_TYPES` in Chinese
  (`:20-28`) while the hydrator emits `WorkItemBlocked`, `ReviewPending`, `DriftDetected`,
  `HandoffPending` (`useDataStore.ts:1954-1996`). No acceptance read occurs there. Result:
  the metric cannot be relied on as the number of decisions he owes. Repair the projection before
  using the overview as the first screen's authority.
- **Fabricated health remains in a reachable dashboard.** `SupervisionDashboard.tsx:62` fixes budget
  health at 72%; `:103-104` calls render-time clock “last sync”; `:116-148` embeds impacts such as
  downstream two seats and average 64%. This is not measured health. Remove the claims or show
  unavailable, using existing sections. Do not manufacture a budget implementation for this audit.
- **Reachable mock global summary.** `SupervisorPanel.tsx:521` renders GlobalDashboard;
  `GlobalDashboard.tsx:11,45-48` reads `MOCK_GLOBAL_SUMMARY`, whose ids are p-1/p-2/p-3
  (`mock-data.ts:292-308`). It can present synthetic projects/health and navigate into an invalid
  context. Repair with a real projection or an explicit unavailable state, not delete all dashboards.
- **Timeline usability and incompleteness.** `TimelineView.tsx:32-44` searches only title/summary/type,
  despite saying “events or evidence”; `:148-151` blames search/filter even with no events and no
  search. Recent fetch is bounded at 400 (`useDataStore.ts:1907`), without full-history paging.
  Template state is unused, external `filters` props are not consulted. Fix truthful empty states
  and search/filter contract; time grouping is a usability improvement, not proof that ordering is
  absent. A recent event list must not be sold as exhaustive replay.
- **Evidence/Playbook duplication.** T7 filtering alone (`PlaybookView.tsx:16-18`) does not publish,
  version or execute skills/tools. Keep documents, narrow the label, then consolidate the redundant
  browser when the user agrees; reuse is a separate later capability.
- **Read freshness is not universal health.** `refreshEvents` only updates events and retains old
  events on error (`useDataStore.ts:1807-1831`). Knowledge tree loads once (`KnowledgeSection.tsx:258`).
  Most views use a single hydration flag; `provenance.ts:123` labels all other hydrated views seed.
  Neither a green/seed banner nor a current clock tells the freshness/provenance of each object.
- **Inbox action promises exceed implementation.** `InboxView.tsx:75` wires onAction to console.log;
  `InboxItem.tsx:99-128` exposes Details and conditionally receipt/reject buttons. Row click resolves
  an object; the Details action itself does not. The receipt controls are conditional on Chinese
  item types not produced by the current hydrator, so distinguish a reachable no-op from latent
  controls rather than claim all receipt buttons currently appear.

### C. Transitional, incomplete, or unreachable: retire the facade, preserve the useful capability

- **Mirror:** keep `LiveSeatsView`/SessionTerminal as terminal detail; retire its role as the primary
  progress/decision screen. “Transitional” does not mean “delete”.
- **V1 command proposal:** `App.tsx:563` still mounts SupervisorCommandBar; `SupervisorCommandBar.tsx:97-142`
  uses a timeout and fixed owner seat-2 then adds a local workitem. `useGlobalShortcuts.ts:24` and
  `App.tsx:153` both listen to Cmd+K. This prototype is reachable, not dead. Retire its fabricated
  proposal path and duplicate shortcut ownership after the user's ruling; retain the live intent input.
- **Four local-only create flows:** WorkItemForm/HandoffForm/AddSeatDialog/DelegationOverlay now show
  `CannotStoreNotice` (`:204/:122/:93/:192`) but still lack real writes. Notice source explicitly says
  local writes remain (`CannotStoreNotice.tsx:14-16`). These are honestly incomplete, NOT “fixed
  persistence”. Hide/disable the production promise or connect through governed writes later.
- **DAG scaffold:** `ProjectDashboard.tsx:477-490,646` still consumes mock workflow; goals/stages have
  explicit no-source handling (`:437-465`). Retain the graph component for real dependency data,
  retire the empty/mock scaffolding as an asserted work graph. Same rule for fake next-step text.
- **Unreachable alternative shell:** `AppV2.tsx` and `views/DocumentsWorkspace.tsx` form an alternative
  shell with no transitive root path measured. Deletion candidates after external-entry/reference
  check; preserve useful document reading through the actual artifact/history routes.
- **MobileCompanionView:** only its own declaration/export is found by `rg MobileCompanionView ui/src`.
  No App/NavRail entry; AppShell's responsive bottom bar maps to existing desktop tabs (`AppShell.tsx:10-20,66`).
  This file is an unreachable alternate presentation, NOT evidence that the whole product lacks
  responsive navigation. Retire the orphan view unless mobile delivery is explicitly selected.

## 4. Which screen is closest to “what needs my decision”?

**Dispatches + AcceptanceSection**, because it reads actual pending acceptance objects and performs
actual decisions. It is now the default. That is closer than SupervisionDashboard's mismatched
ACTION_TYPES count or the supervisor panel's chronological activity feed.

It still misses the whole question in four ways:

1. **Acceptance is only one decision type.** Product rulings, unresolved scope and operator-only
   integration/migration requirements have no structured pending/resolved decision object here.
   SupervisorMessage is a message, not evidence of a still-pending human decision. Do not derive
   human obligations from every blocked dispatch: most blockers belong to seats or Aegis.
2. **Project-local is not fleet-global.** Both dispatch and acceptance queries take activeProjectId.
   “This project has no work” can be true while other projects are busy. Aegis's supplied all-seatloom
   dispatch snapshot explains current distribution, not a licence to discard future project scope.
3. **The card explains how to test, but verdict subject and evidence are not visible enough.**
   `AcceptanceSection.tsx:406-417` shows verdict badge/id/time; getAcceptanceLineage exists
   (`api.ts:150`, `main.rs:183`) but is not consumed by this section. The operator must leave this
   screen to inspect what was actually verified. The merge guard now reads verdict value/subject;
   that does not itself make the card explain them.
4. **Idle/unbound is not a decision.** Seat membership, process liveness and dispatch readiness are
   different facts. No acceptance rows does not mean “nothing requires my decision”, nor does an
   idle terminal mean “delivery is stopped”.

The end-state direction is a SINGLE trustworthy structured work-and-attention view using existing
sections. The tab that currently best serves it should remain the anchor. The supervisor dialog is
its input/conversation companion; history, evidence, knowledge and raw terminals are its zooms.

## 5. Prioritized whole-product dispositions (recommendations, not issued work)

| Priority | Disposition | Cost / risk | User benefit / dependency |
|---|---|---|---|
| P0 | **Repair truth before layout:** remove fixed health/impact/sync claims and synthetic GlobalDashboard data; make unavailable explicit in existing sections | Small-to-medium view/projection change; needs tests of no data/read failure; no schema required just to stop lying | He can trust what “progress” means. No redesign needed. |
| P0 | **Keep dispatches as anchor; unify attention meaning:** acceptance-backed counts first, then separately model user-only rulings with explicit owner/status/evidence | Medium cross-seat data contract; user chooses default presentation. No inferred obligations from arbitrary messages | One answer to “what do I need to decide”, not four incompatible counters. |
| P1 | **Repair read-to-action continuity:** surface existing acceptance lineage, fix no-op Details, truthful timeline empty/search states, consistent recent-versus-complete wording | Medium UI tests/wrappers; existing backend lineage should be exercised, not assumed | From decision/work to its evidence without reconstructing the story outside the product. |
| P1 | **Consolidate semantic progress:** link governed dispatch instructions to work objects, or clearly label distinct counts until linkage is real; supply real dependency graph before claiming it | Medium-to-large semantic contract; cannot silently relabel all dispatches as workitems | Busy team and empty workitem dashboard stop contradicting each other. Graph is supporting detail, not another product. |
| P1 | **Retire fabricated input and qualify local-only forms:** one live intent path, shortcut ruling, durable-write-or-unavailable contract | Small removal for prototype; durable creation is larger governed-write work | No “saved” local data mistaken for work accepted by the team. |
| P2 | **Merge redundant document/Playbook and alternative shells after reference audit**; drop unreachable MobileCompanionView if mobile is not selected | Small code removal, potential external-entry risk; requires reference/test checks | Less maintenance and fewer contradictory names. Do not count lines as user value. |
| Keep | **Retain history, knowledge, scoped seats, qualified artifacts, terminals as detail** | No forced re-layout; improve shared identity/source links over time | Recoverable context and evidence, alongside a single first-screen authority. |

This audit does not enqueue sub-tasks: the dispatch asks for an overall product judgment and
prioritized recommendations, not another round of fragmented implementations. Routing, merging,
installation and gates remain Aegis's actions; default presentation and navigation remain the
user's decisions. No code, live rows, gate events, merge or acceptance card changed.

## 6. Verification record and limits

Commands used: `git rev-parse track/infra-foundation`, `git status --short` on the integration tree;
`rg` over App/NavRail/AppShell and each cited component; `read` of view/store/API/command sources;
a Python static relative-import traversal from both production entrypoints, excluding tests, with
paths/extensions/index resolution; governed `query_ledger(limit=5)` for the recent integration fact.

Checks verified: main + detached supervisor reachability; 11 App tab branches/default;
transitive app-v2 imports; Timeline local filter and backend event ORDER BY;
workitem/handoff inbox construction versus decision metric type domain; fixed dashboard claims;
scoped dispatch/acceptance/seat/document reads; explicit global runtime artifact warning;
CannotStoreNotice plus retained local writes; mobile responsive mapping versus orphan companion.

Not measured: every rendered screen, actual IPC responses, current install commit, per-project
live counts, external dynamic imports, end-to-end source opening, accepted build's migration state.
No claim that a missing local sort means unordered data; no claim that unbound means absent;
no claim that merged means installed; no claim that all indirectly imported files are dead.
