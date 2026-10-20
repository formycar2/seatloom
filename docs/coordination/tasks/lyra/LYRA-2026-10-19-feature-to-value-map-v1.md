# SeatLoom feature-to-value map — what each function actually serves

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-19-feature-to-value-map-v1 |
| status | delivered |
| author | lyra |
| date | 2026-10-19 |
| version | v1 |
| tags | value-map, feature-to-value, mapping, operator-perceivable |
| depends_on | LYRA-2026-10-19-top-level-value-design-v1, LYRA-2026-10-19-whole-product-audit-v1, LYRA-2026-10-19-top-level-design-v1 |

## 0. How to read this map

Two documents already exist: the capability audit (what exists, entry by entry) and the value design
(what he can perceive, five values). This map joins them in one direction only:

> **For each function that exists, which value does it serve, and how would he know?**

A function is not "mapped" because it is well built. It is mapped only when the answer to *how would
he know* is a thing he can see or do. Where the honest answer is "he could not know", the function is
marked as **serving no perceivable value yet** — which is the most useful cell in this table, because
it is where engineering effort and operator benefit have come apart.

The five values, restated in one line each (full definitions in the value design):

- **V1** — know at a glance what the whole team is doing
- **V2** — click anything and understand why it was done
- **V3** — what only he can decide arrives on its own
- **V4** — what he said and what was done match up
- **V5** — what the product says and what it does are the same thing

Mapping notation: **serves** = he can perceive the value through it today. **partially** = the path
exists and is real, but a required half (rendering, integration, or install) is missing.
**enables** = necessary plumbing whose value is only perceivable through another function; never
count it as a value on its own. **none yet** = reachable and functional, but nothing he can perceive.

## 1. The map, by existing function

| Function that exists | V1 | V2 | V3 | V4 | V5 | How he would know | Verdict |
|---|---|---|---|---|---|---|---|
| **Governed dispatch inbox** (list, per-seat grouping, dependency edges, reuse, live poll) | ● | ○ | ○ | | ○ | One screen, grouped by seat, real statuses; a row expands to its full instruction | **serves V1**; feeds V3 |
| **Acceptance cards** (list, plain-language how-to-verify, accept/reject, reject requires a reason) | | | ● | ○ | ● | A card states what to do, what passes, what is rejected; the merge happens only on his click | **serves V3**; V5 evidence |
| **Merge guard** (verdict value + subject validated before the merge) | | | | ○ | ● | A verdict about another dispatch, or a fail, cannot be merged | **serves V5** — invisible when working, which is correct |
| **Supervisor intent entry** (`plan_intent` → proposal, confirm then enqueue; read-only until confirmed) | ○ | | ● | ○ | ● | One sentence becomes a reviewable proposal naming the seat and the evidence before any write | **serves V3**; wired live |
| **Supervisor IM / messages** (append, list, per-seat routing, PTY forward when attached) | | | ○ | ● | | He can see what was said; but a message is not a decision obligation | **partially V4**; PTY path still stubbed |
| **Historical session replay** (project list, session list, conversation with line anchors, source paths) | | ● | | ● | | He reads a past session turn by turn with the original file and line number | **serves V2 and V4** |
| **Knowledge panel** (topic tree, memory cards, three-level disclosure, line-level evidence) | | ○ | | ● | ○ | A distilled claim carries its confidence and its source lines | **serves V4**; discovery is poor |
| **Ledger timeline** (event list, event detail, 8s refresh) | ○ | ● | | ● | ○ | He reads what actually happened, in order, from the canonical record | **serves V2/V4**; recent-only, not exhaustive |
| **Lineage read model** (`lineage(ref)`, MCP tool + Tauri command, verified against live) | | ● | | ○ | ○ | Given any object, the ordered facts about it, with every unknown stated | **partially V2** — built and verified, not yet rendered |
| **Session↔work join** (view + read model, three numbers: resolve / do not resolve / what is missing) | | ● | | ● | ○ | From a past session to what it distilled | **partially V2/V4** — backend only |
| **Project-scoped reads** (projects, seats, workitems, handoffs, sessions, events, documents) | ○ | | | | ● | Switching projects actually changes what he sees | **serves V5** — the correction he reported himself |
| **Provenance badges** (live / ledger / seed / fixture / no-live / discovery-failed) | | | | | ● | A tab never claims LIVE unless the read behind it succeeded | **serves V5** |
| **Cannot-store disclosures** (four forms state they cannot persist) | | | | | ● | The form says the input will be gone, rather than implying a save | **serves V5** |
| **Runtime artifacts + documents** (scoped documents, global runtime artifacts explicitly labelled) | | ○ | | ● | ○ | He browses what was produced, with the global/local limit stated | **partially V4** |
| **Seats page** (project bindings, unbound context, per-seat sessions) | ○ | | | | | He sees who belongs where and which seat is merely unbound | **partially V1** — membership, not work |
| **Live seat mirror** (tmux attach, scrollback backfill, durable mirror state) | ○ | | | | | He watches a seat's raw terminal | **partially V1** — a process, not progress |
| **Governance read models** (acceptance lineage, knowledge tree, history reads; MCP tools + thin commands) | ○ | ○ | ○ | ○ | | They make the above queryable by other layers | **enables** — value only through a surface |
| **Event write path** (`append(event) → projections`, single transaction, schema-validated) | | | | | | Nothing — he never sees it | **enables** |
| **Governed dispatch engine** (ready/blocked scheduling, dependency edges as data, escalation) | | | | | ○ | Work arrives only when its prerequisites exist | **enables**; V5 only indirectly |
| **Discovery / hydration layer** (project scoping, failure handling, refresh cadence) | ○ | | | | ● | The whole surface either matches the project or says why not | **enables V5** |
| **Supervision dashboard** (metric cards, action queue, activity) | ○ | | | | | Metric cards show numbers | **none yet** — competing counters, fixed 72% budget, render clock as sync |
| **Global cross-project dashboard** | ○ | | | | | Cross-project summary cards | **none yet** — fed by `MOCK_GLOBAL_SUMMARY`, ids p-1/p-2/p-3 |
| **Work graph (DAG)** | ○ | | | | | Who depends on whom | **none yet** — mock workflow, empty node set |
| **Goals / stages bands** | ○ | | | | | Where this project is heading | **none yet** — explicitly no data source |
| **Work items view** | ○ | | | | | Semantic units of work | **partially V1** — a second population, unlinked to dispatches |
| **Inbox** | ○ | | | | | Items needing attention | **none yet** — workitem/handoff subset, Details is a no-op |
| **Playbook view** | | ○ | | ○ | | Reusable capability | **none yet** — a T7 document filter, not a published capability |
| **V1 command proposal** (timeout, fixed owner, local workitem) | | | | | | A suggested proposal | **none yet** — fabricated; retire |
| **Four local-only create forms** | | | | | ○ | Honest "cannot store" notice | **none yet** for creation; **serves V5** for honesty |
| **Mobile companion view** | ○ | ○ | ○ | ○ | | A narrow phone view | **none yet** — unreachable from any entry |

## 2. What the map shows

**V1 is over-supplied and under-trusted.** Six functions claim to answer "what is the team doing":
the dispatch inbox (real), the supervision dashboard (fabricated numbers), the global dashboard
(mock), the work graph (mock), workitems (unlinked population) and the inbox (subset). Only the
first is real. Five competing answers is why he cannot trust any of them — the map makes the
consolidation target concrete: keep one, label the others until they are linked, retire the fakes.

**V2 is built but not reachable.** Three functions serve it and all three are real in the backend:
history replay (perceivable), timeline (perceivable), lineage read model (verified against live,
not rendered). This is the cheapest remaining win in the product — the value exists and only the
presentation half is missing.

**V3 has exactly one complete path and three missing kinds.** Acceptance cards work. Product
rulings, scope questions and operator-only prerequisites have no structured object at all — which
is why they arrive as messages and get missed.

**V4 is the most honest layer.** History, knowledge and messages all carry provenance and all state
their limits. What is missing is traversal in the other direction: from a present decision back to
the sentence that caused it.

**V5 is now the widest layer by function count** — provenance, project scoping, cannot-store
disclosures, the merge guard — and it is the layer that was almost entirely absent before. It is
also the layer whose value is invisible when it works, so it must be verified by deliberately
breaking reads, not by admiring a green page.

**Eleven functions serve no perceivable value yet.** Nine of them are reachable. That is the real
size of the gap between what is built and what he can use — and none of them requires new
architecture, only a decision about which claims the product is allowed to make.

## 3. Operator-facing summary

The Chinese companion (`docs/coordination/acceptance/2026-10-19-feature-to-value-map.zhs.md`) states
the same mapping in his language, leads with what he can use today, and lists the four decisions
only he can make. This document is authoritative.

## 4. Evidence and limits

Baseline `track/infra-foundation@cfda740`, clean. Function inventory from the delivered capability
audit; value definitions from the value design; layer ownership from the top-level design. Source
checks for each function's data path were made during those reviews and are not repeated here.

Not claimed: any rendered-app behaviour; the installed build's contents; per-project live counts;
that a "none yet" function is worthless — several are necessary plumbing whose value arrives only
through a surface that does not exist yet. No code, live-database, gate, merge, card or routing
change.
