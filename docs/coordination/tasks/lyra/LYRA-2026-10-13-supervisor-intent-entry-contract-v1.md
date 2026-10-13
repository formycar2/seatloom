---
template: T5
subtype: product_contract
doc_id: LYRA-2026-10-13-supervisor-intent-entry-contract-v1
title: Product contract — the Supervisor Layer intent entry (user story + interaction flow + screen definition + decomposition)
status: active
author: lyra
doc_date: 2026-10-13
version: v1
depends_on: docs/prd-v0.5.md §2/§6.4, docs/interaction-spec-v1.1.md INT-02/INT-13/INT-14, docs/ux-spec-v1.1.md UX-02/UX-09/UX-10, AEGIS-2026-10-13-supervisor-intent-entry-brief.md
tags: product-contract, supervisor-layer, intent-entry, decomposition, p0
---

# Product contract — the Supervisor Layer intent entry

**Owner:** Lyra (PO). **Purpose:** satisfy PRD v0.5's two hard rules — a feature may not
enter implementation unless it exists in all three layers (user story, interaction flow,
screen/UI surface definition), and may not stay in scope unless the PRD states what pain it
solves, what happens if it is omitted, and how user behavior changes.

**This document is the contract.** It is what `ev-supervisor-intent-contract-landed`
announces, and it is the judgment basis for every slice below. It is a *delivery
announcement*, not a capability gate: it asserts that this file exists at a commit, never
that any capability runs.

---

## 1. Verified current state (measured, not reported)

Every claim below was read out of the working tree on `track/infra-foundation` at
`8179efb` before this contract was written. Where a claim in someone else's brief was
wrong or incomplete, the measurement wins.

| # | Claim | Measurement |
|---|---|---|
| V1 | `ui/src/components/SupervisorCommandBar.tsx` is an empty shell | **Confirmed and worse than "empty".** 385 lines. `useDataStore` is imported at line 17 and never used. `generateProposal()` is a `setTimeout(800)` returning a **hardcoded object**: `owner: 'seat-2'`, `ownerName: 'nimbus'`, a hardcoded rationale sentence, `evidence: ['docs/prd-v0.5.md','docs/architecture-design.md']`, `budget: '150,000 tokens'`. `handleConfirm()` builds a **local** WorkItem with `id: wi-${Math.floor(Math.random()*900+400)}` and calls `onConfirmProposal` → `addWorkItem` (a Zustand store write). **Zero backend calls, zero ledger writes.** |
| V2 | `SupervisorPanel.tsx` is half-wired | **Confirmed.** 575 lines; real calls are `api.appendSupervisorMessage` (line 275) and `api.ptyWrite` (line 284). |
| V3 | `SupervisorApp.tsx` is tiny | **Confirmed.** 45 lines; `hydrateFromBackend()` plus `api.closeSupervisorWindow()`. |
| V4 | **The real write path already exists** | **`cmd_enqueue_dispatch` is registered** (`src-tauri/src/main.rs:171`) and **already exposed to the UI** as `api.enqueueDispatch(targetSeatId, title, instructions, priority)` (`ui/src/lib/api.ts:160`). Its implementation (`src-tauri/src/commands/dispatch_cmds.rs:57`) shells out to `seatloom-mcp-emit` over stdio with `SEATLOOM_ACTOR_REF = "human:supervisor"`. **A sentence typed by the user can already become a governed dispatch, attributed to the human.** This is the single most important fact in this contract: the hard part is built. |
| V5 | **…but that write path cannot express a precondition** | **Confirmed gap.** The Tauri wrapper passes only `target_seat_id`, `title`, `payload:{note}`, `priority`. The underlying MCP tool **does** accept `dependencies: [{dep_kind, dep_value}]`. The gap is in the wrapper, not the tool. An intent of the form "do X, but only after Y lands" is currently **not expressible** from the UI. |
| V6 | **`⌘K` is bound twice, and every description of it is false** | **Confirmed defect.** `ui/src/hooks/useGlobalShortcuts.ts:24` binds `Cmd/Ctrl+K` → `onOpenCommandBar()` (the V1 prototype). `ui/src/App.tsx:139` binds `Cmd/Ctrl+K` → toggle the Supervisor IM. Both are `window` keydown listeners; neither calls `stopImmediatePropagation`, so **both fire**. Meanwhile `App.tsx:133-135` comments that the V1 bar "is still registered on ⌘⇧K", and `ShortcutHelpDialog.tsx:11` tells the user `Cmd/Ctrl + K` opens the Command Bar. **Three mutually inconsistent statements about one chord, and the code does a fourth thing: it opens both surfaces at once.** |
| V7 | Routing rationale is computable, not narratable | **Confirmed.** `seats(id,name,default_runtime,capability_tags,status)` and `project_role_bindings(seat_id,project_id,role,authority_doc_refs,constraints,collaboration_template_ref,active_delegation_id)` and `dispatches(target_seat_id,status,priority)` already hold everything a "why this seat" answer needs. The hardcoded rationale string in V1 is not a limitation of the data; it is a placeholder that was never replaced. |
| V8 | A hardcoded wiring flag already exists | **Confirmed.** `ACCEPTANCE_CMDS_REGISTERED = true` (`ui/src/lib/api.ts:67`) is a hand-maintained boolean asserting that three Tauri commands are registered. The commands *are* in `main.rs`'s handler list on this commit, so the flag happens to be correct — but it is a claim about wiring that nothing measures. This is the exact shape Aegis named in boundary 3. |

---

## 2. Layer 1 — User story

**As** Mr. Zhang, the operator of this project,
**I want to** type one sentence — "给 Nimbus 派个活,把唤醒服务的硬底兜底接上" or
"我们为什么放弃了 passport.js?" — and get back a **reviewable, evidence-backed structured
proposal** naming the seat, the reason, the budget and the evidence,
**so that** I drive the project by intent instead of pulling agents by hand in a terminal.

**Who is stuck today, and how.** The governed-dispatch mechanism works: dependency edges
are data, gates are events, Flux verdicts are commit-pinned, acceptance cards carry lineage.
None of it is reachable by the person who owns the product. Every decomposition, dispatch,
gate check and ruling in this project's history is one agent's turn in a terminal. When
Mr. Zhang asked on 2026-10-13 "你在 SeatLoom 中体现在哪里?", the honest answer was "不在".
The sentence he types today goes into a 385-line prototype that writes to a Zustand store
and stops there.

**Secondary user:** any human operator of the project, who has the same need and the same
absence.

---

## 3. Layer 2 — Interaction flow

Entry point and surfaces are decided in §4. The flow below is surface-independent.

### 3.1 The main path (structured action)

1. **Input.** The user types one sentence into the intent entry and submits.
2. **Deterministic parse.** The system extracts, with no LLM in this step:
   `verb` ∈ {create, route, explain, find-evidence, recover} · `object_type` ∈
   {dispatch, workitem, handoff, comment} · `owner` (resolved against the seat registry by
   exact name or seat id) · `qualifiers` (topic, branch, precondition, budget hint).
   **Unsupported or unparsed is a result, not an error to be repaired by guessing.**
3. **Resolve or STOP.** Any of the conditions in §3.3 ends the flow at a clarification
   card. The system never selects a default to keep moving.
4. **Ground in real reads.** Seat registry + project role bindings + current dispatch load;
   `recall` / `get_knowledge_tree` / `query_ledger` for anything the intent references;
   budget computed from the seat's actual configuration and the actual pack, never a
   constant string.
5. **Suggestion card.** Intent restated in one line · proposed object · owner with
   capability match, role constraints and authority docs · evidence links · computed
   budget · impact (what changes on confirm) · `Confirm` / `Edit` / `Cancel`. `Edit` is
   inline on the card, never a separate blank form (UX-02).
6. **Routing rationale, on demand.** A "why this seat" disclosure on the card. Pressing it
   shows the computed facts: which capability tag matched, which constraint excluded each
   other seat, how many dispatches that seat currently holds. **Every line is a fact with a
   source. No line is a sentence an LLM composed.**
7. **Confirm before any write.** Nothing is written before `Confirm`. On confirm the write
   goes through the existing governed path (`enqueue_dispatch` via the existing Tauri
   wrapper) with actor `human:supervisor`.
8. **Result.** The card resolves to the created object: the dispatch id and the ledger
   event id are shown and are linkable. If the write fails, the id is **not** shown as
   created and the error text is shown verbatim.

### 3.2 The why-question path (decision-answer card)

1. The user asks a why-question ("为什么放弃 passport.js?").
2. **Exact retrieval first**: `recall` over the resolved topic with its ancestor chain,
   `l0` by default, `l1`/`l2` only on demand. Exact ledger/artifact matches before any
   interpretation (UX-10's design rule).
3. The card shows: one concise answer block · source evidence chips · a confidence note ·
   `Open sources` / `Pin to continuity` / `Copy answer`.
4. If semantic recall adds context, it is **visually marked as an explanation layer**, never
   as the truth source.
5. **Insufficient evidence is a first-class result**: `Evidence incomplete` plus the list of
   sources actually searched (INT-14). An answer with no evidence is not produced.

### 3.3 Ambiguity and failure paths — the load-bearing section

**The rule: when the intent is unclear, the system asks. It does not guess.** This project's
recurring disease is guessing — fabricated lineage, self-authored plans, delivery
announcements spent as gates. The entry layer is where a fourth instance would do the most
damage, because it would look like a feature.

| Condition | Required behavior | Forbidden |
|---|---|---|
| Verb not in the supported set | Show the five supported verbs with one-line examples and ask which | Inferring a verb from keywords |
| Owner named, zero seats match | `No seat named "X"` + the real seat list | Defaulting to Nimbus (this is what the current prototype does) |
| Owner named, more than one match | `Owner unclear` + the candidates | Picking the first |
| Owner omitted and not inferable | Ask which seat, with the computed candidate list and why each is a candidate | Choosing the "obvious" seat |
| Object type unclear | Ask which kind of object | Assuming a dispatch |
| Intent carries a precondition ("等 X 落地再动") and the write path cannot express an edge | **Refuse that part explicitly**, name the missing capability, and offer to create the dispatch without the precondition only after the user says so | Silently dropping the precondition (V5) |
| No capable seat | `No eligible seat found` + the blocked constraints listed (INT-02) | Routing to the least-bad seat |
| Budget would be exceeded | Show the cause before confirmation (INT-02) | Confirming anyway |
| Evidence insufficient for a why-question | `Evidence incomplete` + sources searched | Composing an answer anyway |
| Write fails | Show the error verbatim; do not show a dispatch id | Showing a plausible id |
| A surface's backing command is not callable | Say so, with the measured fact (the actual error), and name what is missing | Asserting a wiring state that was not measured (V8) |

### 3.4 The continuity-delta path

Deterministic reads only: unresolved blockers, open dispatches per seat, pending handoffs,
and the last gate decision, each row linking to its ledger object. "Since when" is the last
supervisor checkpoint; if there is none, the card says so rather than showing an empty frame.

---

## 4. Layer 3 — Screen definition

**Placement decision (mine, and it is a decision — see §8 for the conflict it resolves).**

The intent entry is the **input row at the top of the existing Supervisor IM panel**
(`ui/src/app-v2/panel/SupervisorPanel.tsx`), which is already bound to `⌘K` and is already
the canonical L1 surface with real calls. Suggestion cards, decision-answer cards and
continuity-delta cards render **inside that same panel**. No new window, no new route, no
re-layout of the existing main-interface skeleton.

The V1 `SupervisorCommandBar.tsx` prototype is **not** the target surface and is **not
touched**. It is a prototype; editing it would trip the file-granularity check on a file
that is already dirty, for no product gain.

| Surface | Definition |
|---|---|
| **Intent entry** | The panel's existing input row, with an explicit mode affordance: `派活` (structured, governed) and `说句话` (raw IM to a seat, the existing `appendSupervisorMessage` + `ptyWrite` behavior). Structured first, free text second — PRD §6.4 rule 4. The two are never blended into one submission. |
| **Suggestion card** | The UX-02 field set (Intent / Proposed objects / Ownership / Evidence / Budget / Impact / Actions) at the existing 640px card width, every field bound to real data, plus a `why this seat` disclosure. Inline `Edit`. |
| **Clarification card** | Replaces the suggestion card when §3.3 fires. One question, the real choices, and the reason each choice is offered. Not a toast, not a disabled state. |
| **Decision-answer card** | Answer block + source chips + confidence note + `Open sources` / `Pin to continuity` / `Copy answer` (UX-02 / UX-10). |
| **Continuity-delta card** | Four blocks — unresolved blockers / open dispatches / pending handoffs / last gate decision — each row linking to its ledger object. Empty means "why empty and what next", never a blank frame. |
| **Wiring honesty** | `ACCEPTANCE_CMDS_REGISTERED` and any sibling flag are **replaced by a measured capability probe** (call the command, observe the result). A surface either works or states the measured error. No hand-maintained boolean asserts a wiring state. |

---

## 5. The three must-answers (PRD v0.5 rule 2)

1. **What pain it solves.** The operator runs this project by hand. The product's own thesis
   is "a built-in supervisor that turns user intent into structured operations", and today
   that thesis is a sentence in a document: the mechanism exists and works, and the only way
   in is an agent in a terminal. The intent entry is the front door to machinery that is
   already built and already governed.
2. **What happens if it is omitted.** The governed-dispatch layer — dependency edges as data,
   gate events, commit-pinned verdicts, acceptance cards with lineage — stays reachable only
   by whoever happens to be holding the terminal. Every future operator inherits the same
   manual loop, and the project's audit trail keeps conflating "what a seat decided" with
   "what the user asked for", because there is no path by which the user asks for anything.
3. **How user behavior changes.** The unit of work becomes one sentence plus one
   confirmation click. Work enters the ledger as `human:supervisor`-issued dispatches with
   real preconditions and real evidence, so the audit trail distinguishes user intent from
   seat decision for the first time. The operator stops being the router.

---

## 6. Explicitly out of scope, with reasons

Per PRD rule 2, a feature that cannot state its cost of omission does not stay in scope.
These are out, and the reasons are recorded so they do not drift back in silently:

- **A new LLM "supervisor seat."** PRD §6.4: the Supervisor is a built-in orchestration
  layer, not a user-created seat. Adding a guessing machine at the entry point would be the
  fourth instance of this project's core disease, wearing a feature's clothes.
- **Playbooks.** PRD P1, and not verified as landed (Aegis's own table marks it unverified).
  The intent entry does not need them.
- **Batch proposals, rich edit-before-confirm, review-assist drafting, semantic recall as a
  truth source.** PRD P1/P2.
- **Mobile surfaces** (INT-17…INT-20). Separate slices with their own contracts.
- **Any re-layout of the existing main-interface skeleton.** Forbidden by the operator's
  explicit instruction; the checks are file-granular and the baseline is already red.
- **Touching `ui/src/components/SupervisorCommandBar.tsx`.** Prototype, already dirty, no
  product gain. Leave it alone.

---

## 7. Decomposition into dispatchable slices

Dependency edges are **event ids**, per the standing rule: a knowable-unmet precondition is
an edge or a pre-declared referent, never prose. `blocked` is not an error and not a stall.

| Slice | Seat | Dispatch | Edges (all `dep_kind=event`) | Status |
|---|---|---|---|---|
| 0. This contract | Lyra | `disp-supervisor-intent-layer-decompose` | none | in flight → emits `ev-supervisor-intent-contract-landed` |
| 1. Intent tool surface | Nimbus | `disp-supervisor-intent-mcp-surface-v2` (Aegis's) | `ev-supervisor-intent-contract-landed` | blocked until slice 0 emits |
| 2. UI against the contract | Mira | `disp-supervisor-intent-ui-v2` (Aegis's) | `ev-supervisor-intent-contract-landed` | blocked until slice 0 emits |
| 3. Verify the tool surface | Flux | **`disp-supervisor-intent-tools-verify`** (new, this contract) | `ev-supervisor-intent-tools-landed` | to be enqueued |
| 4. Wire the UI to the live tools | Mira | **`disp-supervisor-intent-ui-wiring`** (new, this contract) | `ev-supervisor-intent-tools-live` **and** `ev-verdict-supervisor-intent-tools` | to be enqueued |

**Why these edges and not others.** Slice 3 only needs the code to *exist* in order to
verify it, so a `-landed` announcement is the correct edge. Slice 4 actually *calls* the
capability, so it needs the measured `-live` gate, and it must not wire code that has not
passed verification — hence both edges.

### 7.1 The event contract — exact ids and required payloads

Seats emit these; nothing else releases the chain.

| Event id | Emitter | Type | Required payload |
|---|---|---|---|
| `ev-supervisor-intent-contract-landed` | Lyra | `ArtifactCreated` | `contract`: this file's path + commit. Announcement semantics — it asserts a file exists, **not** that any capability runs. No `verified` field. |
| `ev-supervisor-intent-tools-landed` | Nimbus | `ArtifactCreated` | `branch`, `commit`, `files`, `dispatch_id`. Announcement. No `verified` field. |
| `ev-supervisor-intent-tools-live` | Nimbus | `ArtifactCreated` | **`verified`** (mandatory, per the gate ruling): which call was made, exactly what it returned, on which database, at what time. This is a gate; it may only be emitted after a real end-to-end call succeeded against the live database. |
| `ev-verdict-supervisor-intent-tools` | Flux | `ReviewVerdictIssued` | Pre-declared when slice 3 is enqueued; Flux must emit with exactly this id. |

### 7.2 Acceptance criteria, per slice

**Slice 1 (Nimbus) — the intent tool surface.**
- A typed intent schema (verb / object_type / owner / qualifiers) as data, with the
  ambiguity outcomes of §3.3 as *typed results*, not as error strings.
- A deterministic parser. No LLM in the parse step.
- A proposal builder that fills every UX-02 field from real reads; **no hardcoded string
  survives in any field** — the current prototype's `budget: '150,000 tokens'` and its
  rationale sentence are the reference for what "hardcoded" means here.
- A routing-rationale computer that returns, per candidate seat, the matched capability, the
  excluding constraint, and the current in-flight count. Facts with sources.
- Both surfaces over the **same** repository functions: an MCP tool and a Tauri command
  (the established thin-wrapper pattern). No second implementation.
- The Tauri wrapper must accept `dependencies` (V5), or explicitly refuse an intent that
  carries a precondition and say so (never silently drop it).
- `human:supervisor` actor attribution preserved.
- Self-request Flux verification per the completion protocol.

**Slice 2 (Mira) — the UI against the contract.**
- Intent entry, suggestion card, clarification card, decision-answer card, continuity-delta
  card, built against this contract's shapes.
- Every §3.3 row reachable in the UI, with the required text and the real choices.
- The two-mode input (`派活` / `说句话`) with no blended submission.
- `ACCEPTANCE_CMDS_REGISTERED` and any sibling flag replaced by a measured probe.
- **Zero new interface-check issues, measured.** Files that were already red stay exactly as
  red as they were; no opportunistic re-layout. The V1 command bar file is not touched.
- Built against the contract now; **wired to live tools only in slice 4**.

**Slice 3 (Flux) — verification, commit-pinned.**
- §14 captures before starting; `HOLD` on mismatch or unauthorized dirt.
- The parser's ambiguity outcomes are typed results, not stringly-typed errors.
- **The no-guessing property is verified adversarially**: for each §3.3 row, Flux constructs
  the input and confirms the system stops and asks. A parser that "usually" stops is a fail.
- No hardcoded field survives in the proposal builder.
- The rationale computer's output is checked against the actual seat/role/dispatch data.
- Gates run for real, output pasted.

**Slice 4 (Mira) — wiring.**
- The UI calls the real commands; a real sentence produces a real governed dispatch with a
  real ledger event, end to end, against the live database.
- A precondition-carrying intent either produces a dispatch **with** the edge, or is refused
  explicitly.
- The end-to-end acceptance card is requested with the §3 interaction flow as its
  acceptance plan, verbatim. The card merges only on the operator's Accept click; no seat
  merges.

---

## 8. Decisions I need from Aegis (unresolved product truth)

**The `⌘K` conflict (V6).** The spec (`INT-02`) says `Cmd/Ctrl+K` opens the command bar.
The code's comment says `⌘K` opens the IM and `⌘⇧K` opens the command bar. The help dialog
tells the user `⌘K` opens the command bar. The code opens **both** on `⌘K`, because two
`window` listeners fire and neither stops the other.

My placement decision in §4 sidesteps it by putting the intent entry inside the IM panel,
which is already `⌘K`. But the duplicate binding is a live defect independent of this
feature: today `⌘K` opens the canonical IM **and** a 385-line prototype modal with a
hardcoded proposal. I am not going to move a keyboard shortcut the operator uses every day
on my own authority, and I am not going to amend `INT-02` unilaterally. **Ask: which chord
owns the intent entry, and what happens to the V1 prototype's binding?** Until that is
answered, slice 2 must not touch the shortcut layer at all.

---

## 9. What this contract does not do

It does not authorize any seat to write code, merge, restart a service, or deploy. Slices
1–4 do that, each behind its own edges and its own verification.
