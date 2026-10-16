# Product contract — the first screen: what the whole fleet is doing, what is stuck, what needs your decision

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-16-first-screen-contract-v1 |
| status | issued |
| author | lyra |
| date | 2026-10-16 |
| version | v1 |
| to | aegis, mira, nimbus, flux |
| depends_on | docs/coordination/ROADMAP.md (§2b UM2, the 2026-10-16 priority correction), LYRA-2026-10-16-next-round-plan-v1.md |
| tags | product-contract, first-screen, um2, decomposition, heartbeat |

**Owner:** Lyra (PO). **Raised by:** Aegis's dispatch `disp-598TQuyRewCXs8aG8GKmK`. **Measured on:**
`track/infra-foundation` @ its current tip plus the live ledger (177 dispatches, 1181 events) and the
live database's `seats` table (7 rows). **The hard constraint is honoured throughout:** existing
partitions, existing components, existing information architecture. No new interaction, no new
navigation, no new visual language. Where satisfying his ask would require one, it is flagged in §7
rather than built.

The operator's two sentences, verbatim (translated): *what is the whole fleet doing, what is stuck,
what needs my decision*, and *improve the features on the basis of the existing SeatLoom design — do
not do new interaction design any more*.

---

## 1. Verified current state (measured, not reported)

| # | Claim in the dispatch | Measurement | Verdict |
|---|---|---|---|
| S1 | `ProjectDashboard` composes in an order close to his three questions | `ui/src/app-v2/dashboard/ProjectDashboard.tsx`: Blockers → ActiveWork → StageProgress + DagWorkflow → Next step → Artifacts → Timeline → Goals → Knowledge | ✅ the order matches |
| S2 | `DagWorkflow.tsx` + `dag-model.ts` are a capable work-graph | nodes carry `dependsOn`, statuses include `blocked` and `failed`; edges carry `dependency / feedback / escalation / delegation`; `computeLayers`, `getUpstream`, `getDownstream` exist | ✅ the component is capable |
| S3 | `DispatchInboxView` already reads real dispatches | `api.listDispatches()` polled every 5s (`DispatchInboxView.tsx:561`), and the comment records *why* it deliberately does **not** use the UI store, whose active project may be mock data with different ids | ✅ real, and correctly so |
| S4 | 🔴 `ProjectDashboard` eats `useDataStore`, which holds only seed rows | the four seed files insert 1 workitem, 1 session, 1 handoff, 1 seat, 1 delegation, 7 artifacts, 8 events, 7 documents | ✅ confirmed |
| S5 | 🔴 **The work graph is fed by mock and can never fill** | `ProjectDashboard.tsx:454`: `workflow: mockPlanData?.currentStage?.workflow \|\| { nodes: [], edges: [] }`, where `mockPlanData = MOCK_CHANNEL_DATA[channelId]` and the mock's keys are `ch-p1 / ch-p2 / ch-p3` while the only live project id is `seatloom` | **new, and worse than reported** |
| S6 | The dispatch page **already groups by seat** with a live status | `groupMode: 'seat' \| 'flat'` defaulting to `'seat'`, a per-seat status strip as the group header, and a deterministic per-seat tint whose comment says the operator learns "the teal one is Nimbus" | ✅ **it already answers his first question** |
| S7 | The dispatch page's empty state already distinguishes three cases | `EmptyState` separates *nothing ever dispatched* / *search matched nothing* / *status filter empty*, each with a different next action | ✅ the discipline he wants already exists |
| S8 | `AcceptanceSection` already has an honest empty state | "no deliveries awaiting acceptance" | ✅ |
| S9 | The **seats** page shows **one** seat while the database holds seven | `SeatsView` reads `useDataStore` (seed, 1 seat); live `seats` = aegis, flux, lyra, mira, nimbus, onyx, vela | new — but out of scope, see §8 |
| S10 | The default landing is the mirror | `App.tsx:92`, `useState<Tab>('live-seats')` | ✅ |
| S11 | *(not in the dispatch)* | **Nothing in the ledger records which seat is expected to produce a given referent.** A blocked dispatch's edge names the event id it waits for; if that event does not exist, no row says who will emit it. | new — §7 gap 1 |

**What S5 and S6 together decide.** The one existing partition that already answers his first
question from a real table is the **dispatch page**. The partition whose *layout* most resembles his
three questions — `ProjectDashboard` — is the one whose work graph is fed by mock keyed to projects
that do not exist. So the first screen is not the dashboard. It is the dispatch page.

---

## 2. The decision: what the first screen is

**The first screen is the existing dispatch page** (`dispatches` tab, `DispatchInboxView` +
`AcceptanceSection`), and it becomes the default landing.

It is chosen because it is the only surface in the product that reads a real, non-seed, non-mock
table at scale — 177 dispatches with live statuses and real dependency edges — and because its
existing composition already answers his first question outright (S6). Nothing new is built to
make it the first screen: one line changes.

| His question | Existing partition that answers it | The real table it eats |
|---|---|---|
| What the whole fleet is doing | the dispatch table, **grouped by seat**, with the per-seat status strip as the group header (S6) | `dispatches` — `target_seat_id`, `status`, `liveStatus`, `priority`, `issued_at` |
| What is stuck | the blocked rows and their dependency column | `dispatches` + `dispatch_dependencies` (`dep_kind`, `dep_value`) |
| What needs my decision | `AcceptanceSection`, already mounted on the same page | `acceptances` (`status='pending'`) |

**What is deliberately not the first screen:** `ProjectDashboard`. Its section order is right and
its mock-fed work graph is not, and re-feeding the DAG from real dispatches is a larger piece of
work with its own contract. It stays where it is, inside the ⌘K panel, unchanged. Naming that as
deferred rather than quietly half-doing it is the point.

---

## 3. What each partition must say when it is empty

The discipline already exists (S7, S8) and is extended, not replaced. An empty frame that reads as
"broken" is the failure he has already hit once today.

| Partition | Empty because… | What it must say |
|---|---|---|
| The dispatch table | nothing has ever been dispatched | "no dispatches yet" + how to create one (the existing hint already does this) |
| | the search matched nothing | "no matching dispatches" + how to clear it (existing) |
| | the status filter is empty | "none in this state" + switch to All (existing) |
| | the read failed | **the measured error text, and what is missing.** No forever-spinner, no silent empty list |
| The blocked rows | nothing is blocked | say so plainly — "nothing is stuck" — not an absent section |
| The acceptance section | no card is pending | "no deliveries awaiting acceptance" (existing) |
| | the read failed | the measured error text |

---

## 4. The criterion for "what needs my decision" (judgment — my answer is that two kinds are not enough)

Aegis measured two: cards awaiting the operator's click, and things blocked on Aegis. **My judgment
is that this misses the third kind, and the third kind is the one he has been asking about all
day.** The criterion, stated once:

> **A thing needs his decision if and only if the next action is one only he can take.** Three kinds,
> and only three:

| Kind | Test | What it looks like today (measured) |
|---|---|---|
| **K1 only he can click** | the next action is a click only he can make | acceptance cards with `status='pending'` — the merge happens only on his Accept click |
| **K2 only he (or Aegis) can produce** | the thing everyone waits for can only be produced by him or Aegis | a gate ruling only Aegis emits; a branch only Aegis integrates; a live-database command only the operator can run. `disp-session-key-ui` is the live example: its referent `ev-session-key-readmodel-live` was **deliberately deferred** by Nimbus at 09:29 because the bridge view is not on live, so no seat is producing it |
| **K3 only he can decide** | it is a product judgment, not a technical one | U1 (should the supervisor column default-collapse memory-type internal actions) and U2 (is the id→title fix what he meant) — recorded in the ROADMAP and nowhere he would meet them |

**K3 is the kind with no home.** K1 has a partition. K2 is partially visible as a blocked row. K3
exists only in the roadmap and in SupervisorMessages addressed to Aegis, which he reads only if he
opens the ledger. That is §7 gap 2.

**Explicitly not his:** anything a seat can measure and decide. Gate conditions, dispatch routing,
verification, the acceptance gate's own criteria. Pushing those to him would be the failure mode
the heartbeat names — do not push the user's decisions to Aegis, and do not make the user's
decisions for him.

---

## 5. The two judgment calls the dispatch asked me to make

### 5.1 The default landing

`App.tsx:92` becomes `'dispatches'`. One line. The mirror stays as a NavRail tab, undeleted, as the
"drop into a seat's raw terminal" detail view — consistent with his own 2026-10-08 KEEP and his
correction today that the mirror is transitional rather than the product's face.

### 5.2 The ⌘K panel and the first screen

**They coexist, with different jobs, and the first screen is not the ⌘K panel enlarged.** His own
words, recorded today at `8d0ad95`: ⌘K opens the supervisor panel, which is a **message dialog** —
a draggable, resizable IM-style float with a contact list and a conversation area. The mirror is a
different NavRail tab and has nothing to do with ⌘K.

So: ⌘K is where he **talks** to the fleet. The first screen is where he **sees** how it is going.
Merging them would be a new interaction and is forbidden by his second sentence.

---

## 6. The decomposition, with edges on event ids that exist

Verified against the ledger before writing any edge: `ev-session-key-readmodel-landed`,
`ev-verdict-session-key`, `ev-lineage-readmodel-landed`, `ev-lineage-readmodel-live`,
`ev-history-sessions-read-live` and `ev-NFPr-zN36by-i9SE-0FPH` **exist**;
`ev-wiring-honesty-backend-landed`, `ev-session-key-readmodel-live` and `ev-lineage-ui-landed`
**do not**.

| # | Slice | Seat | Dispatch | Edges (all `dep_kind=event`) |
|---|---|---|---|---|
| 0 | This contract | Lyra | `disp-598TQuyRewCXs8aG8GKmK` | — → emits `ev-first-screen-contract-landed` |
| 1 | Make the dispatch page the default landing | Mira | **`disp-first-screen-default-tab`** (new) | none → ready |
| 2 | The blocked-row annotation's data | Nimbus | **`disp-blocked-on-whom-dto`** (new) | none → ready |
| 3 | Render the annotation in the existing dependency column | Mira | **`disp-blocked-on-whom-ui`** (new) | `ev-blocked-on-whom-dto-landed` → blocked |
| 4 | Verify the first screen adversarially | Flux | **`disp-first-screen-verify`** (new) | `ev-first-screen-default-tab-landed` → blocked; pre-declared verdict `ev-verdict-first-screen` |

**Why so small.** S6 means his first question is already answered by existing code; S7 and S8 mean
the empty-state discipline already exists. What is genuinely missing is one annotation and one
default. Everything larger is either already dispatched by Aegis (`disp-qR65V3mi0m-4yYl4J8mmk`
covers the machine-code dependency column, the empty goals/stages bands and the knowledge panel's
hand-written constant) or is a named gap in §7.

**Why slice 3 waits for slice 2 and not the reverse.** The annotation's fact — *does the thing this
row waits for exist yet* — is not in `DispatchDependencyDto`, and `dto.rs` is Nimbus's fence. Mira
cannot render a fact the DTO does not carry, so she waits for the announcement rather than
inventing a local seam. This is the lesson of `disp-lineage-ui`, which failed this morning for
exactly that reason.

**Why slice 4 is edged on the announcement, not a gate.** Flux verifies the code, so an
announcement is the right edge. The click-level check is the operator's acceptance card, not a
gate.

### 6.1 The event contract — exact ids and required payloads

| Event id | Emitter | Type | Required payload |
|---|---|---|---|
| `ev-first-screen-contract-landed` | Lyra | `ArtifactCreated` | `contract`: this file's path + commit. Announcement; no `verified` field. |
| `ev-first-screen-default-tab-landed` | Mira | `ArtifactCreated` | `branch`, `commit`, `files`, and the exact one-line change. Announcement; no `verified` field. |
| `ev-blocked-on-whom-dto-landed` | Nimbus | `ArtifactCreated` | `branch`, `commit`, `files`, and the exact DTO addition. Announcement; no `verified` field. |
| `ev-verdict-first-screen` | Flux | `ReviewVerdictIssued` | Pre-declared at enqueue time; Flux must emit exactly this id. |

### 6.2 Done definition, per slice

**Slice 1 (Mira) — one line.** `App.tsx:92` becomes `'dispatches'`. Nothing else changes: no
re-layout, no new navigation, no new component, the mirror keeps its tab. Prove the mirror is still
reachable and still attaches.

**Slice 2 (Nimbus) — one fact, added to the DTO.** `DispatchDependencyDto` gains whether the
referent exists, and if it does, its event type and when it happened. Read it from the ledger at
read time; do not cache it into the dispatch row. `dto.rs` is shared — additive only, no existing
struct's behaviour changes.

**Slice 3 (Mira) — render it in the existing column.** The dependency column shows, per edge: what it
waits for in words, and whether that thing exists yet. No new column, no new partition, no new
affordance. A row waiting on something that does not exist says so in words, not as
`event:ev-…`.

**Slice 4 (Flux) — attack the three claims.** At minimum: a dispatch blocked on a referent that
exists (must show what it was); a dispatch blocked on a referent that does not exist (must say so,
and must not look like the previous case); a dispatch with no edges; an empty dispatch list; a
failed read. Report exact input and exact output for each.

---

## 7. Two things I am flagging rather than building, because building them would violate his second sentence

**Gap 1 — "is anyone working on the thing I am blocked on" is not answerable from the data.** A
blocked dispatch's edge names the event it waits for. If that event exists, its `actor_ref` says
who produced it — measurable. If it does not exist, **no row says who will produce it.** The
mapping from referent to expected producer lives in prose (my own contracts' event tables), not in
a table. So the first screen can say *what* a row waits for and *whether it exists yet*, and it
cannot say *whether anyone is on it*. Adding a producer column to `dispatches` would be a schema
change and a new piece of information architecture; I am not doing that unilaterally. **This is
the one place where his question is only half-answerable today, and he should know it.**

**Gap 2 — K3, the product decisions that are only his, have no home on any screen.** U1 and U2 are
recorded in the roadmap. The rulings he has been asked for (the ⌘K chord, the roadmap-versus-doc-gate
conflict, whether `4cfd2302` stands) live in SupervisorMessages to Aegis. The nearest existing
surface is the timeline's event-type filter, and whether filtering the timeline to "things awaiting
a decision" counts as using an existing partition or as a new interaction is exactly the judgment
his second sentence reserves to him. **I am not deciding it.**

---

## 8. Out of scope, with reasons

- **Re-feeding `DagWorkflow` from real dispatches.** The component is capable (S2) and the data is
  mock keyed to non-existent projects (S5). It deserves its own contract; folding it in here would
  be the largest piece of work on the page disguised as a detail.
- **The seats page showing one seat when the database holds seven (S9).** Real defect, wrong round:
  fixing it means changing what `SeatsView` reads, and this round's rule is no new information
  architecture.
- **Any re-layout of the interface skeleton.** Forbidden by the operator, restated.
- **Touching the live database, emitting gate events, merging.**
