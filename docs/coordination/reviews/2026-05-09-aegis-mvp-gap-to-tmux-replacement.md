# MVP Gap Review — SeatLoom v0.1 Path to tmux Replacement

| Field | Value |
|---|---|
| template | T4 |
| subtype | gap_review |
| id | 2026-05-09-aegis-mvp-gap-to-tmux-replacement |
| status | **held** |
| author | aegis |
| date | 2026-05-09 |
| to | lyra (on re-issuance) |
| priority | P0 |
| depends_on | `docs/PRODUCT_TRUTH.md` (§1.2 active contract set), `docs/prd-v0.5.md` (§3 priority boundaries, §5 P0 story map US-P0-01 – US-P0-15, §6 module contracts), `docs/architecture-decisions.md` (AD-008 – AD-013 v2), `docs/architecture-design.md`, `docs/coordination/reviews/priorities.md` (L1/L2 framework, chan-06), `docs/coordination/reviews/2026-05-08-aegis-cli-plan-mode-integration-design.md` (chan-10), `docs/coordination/COORDINATION_RULES.md` |
| tags | mvp, tmux-replacement, gap-analysis, v0.1, coordination, held |
| acceptance owner | lyra (on re-issuance) |
| concurrency rule | Review authority only. Does not itself edit code or seed. Lyra sequences T3 packets citing this doc — **but only after re-issuance**. |

---

## Hold state (2026-05-09)

**Status: HELD.** This review is on disk and traceable, but it is **not dispatched to Lyra** and **no T3 packets may be drafted against it** until Mr. Zhang explicitly ratifies the upstream precondition below.

| Aspect | Value |
|---|---|
| Directly-engaged seat | **Mira** |
| Baseline under rework | V1 + v2 frontend shell — Mr. Zhang is working with Mira directly on frontend rework before the multi-seat coordination cascade kicks in |
| Explicit signal that re-issues this review | Mr. Zhang says he is satisfied with Mira's frontend rework, OR explicitly instructs Aegis to hand this review to Lyra |
| Effect during hold | Lyra must not acknowledge, must not draft A1, must not queue B1/B2/B3. Aegis will not push follow-up supervision artifacts that presume the packet list below is the baseline. §4 packet scopes may shift once Mira's rework lands. |

On re-issuance: flip `status: held` → `status: issued`, remove this section, tmux-notify Lyra to begin A1 drafting.

---

## 0. Why this review exists

Mr. Zhang asked (2026-05-09):

> "我首先需要你重新回顾 seatloom 的 MVP 交付目标，然后评估 MVP 距离完全替代 tmux 协作的模式还有哪些要做的，要细化，需要按照 Seatloom coordinate 协作流程来完成，不要都你自己完成"

Three directives:

1. **Re-anchor** to the canonical MVP delivery goal (not Aegis's private interpretation).
2. **Itemise** the delta between current v0.1 state and the workflow that fully replaces tmux-based multi-agent coordination.
3. **Follow SeatLoom coordination rules** — Aegis produces review authority; Lyra authors sequenced T3 task packets; Nimbus / Mira / Flux deliver and verify per their role contracts. Aegis does **not** land all packets directly.

This review is the authoritative artifact that Phase-next T3 packets must cite in `depends_on`.

---

## 1. MVP baseline recap

### 1.1 Canonical contract set (PRODUCT_TRUTH.md §1.2)

| Topic | Source |
|---|---|
| Product value, scope, P0 boundary | `docs/prd-v0.5.md` |
| User trigger / flow / success-failure behaviour | `docs/interaction-spec-v1.1.md` |
| Screen structure, fields, layout | `docs/ux-spec-v1.1.md` |
| Pass/fail criteria | `docs/acceptance-spec-v1.1.md` |
| System constraints | `docs/architecture-decisions.md`, `docs/architecture-design.md` |

### 1.2 P0 story map (PRD v0.5 §5)

The MVP delivery target is the **15 P0 stories US-P0-01 through US-P0-15**. Stories are classified by surface layer (chan-06 / priorities.md):

| Layer | Stories | Layer definition |
|---|---|---|
| **L1 — Supervisor IM primary surface** | US-P0-01 (start-of-day action queue), US-P0-02 (intent → structured proposal), US-P0-11 (prompt-state classification surface) | Highest-frequency shortest-path interaction. |
| **L2 — Typed data surfaces** | US-P0-03 (seat identity), US-P0-04 (delegation), US-P0-05 (review-fail + reissue), US-P0-06 (continuity pack), US-P0-07 (typed artifact review), US-P0-08 (Seat Card capability truth), US-P0-09 (tiered continuity preview), US-P0-10 (evidence search) | Necessary but lookup / audit / batch-review frequency. |
| **Mobile companion** | US-P0-12 (mobile health), US-P0-13 (mobile approve / reject / escalate), US-P0-14 (mobile short feedback), US-P0-15 (mobile interrupt triage) | Per PRD §3.1, mobile exists for monitoring + approvals + short feedback only. |

### 1.3 v0.1 concrete scope (Mr. Zhang directive, 2026-05-08)

> "全量 + 实时 session" — end-to-end with PTY-wrapped seat sessions, migrating the current 271+ coordination document corpus into the running system.

### 1.4 v0.1 ship-gate definition

v0.1 ships when:

1. Every **L1 P0 story** (US-P0-01, US-P0-02, US-P0-11) is demonstrably load-bearing on real PostgreSQL data.
2. A **complete supervisor → seat → supervisor round-trip** runs inside SeatLoom without touching tmux.
3. Every new coordination artifact authored during a v0.1 session is persisted back to `docs/coordination/` and re-ingested via reconcile without manual recovery steps.
4. L2 stories **US-P0-07, US-P0-08, US-P0-10** reach read-only parity (user can inspect, cannot yet author).
5. L2 stories **US-P0-03, US-P0-04, US-P0-05, US-P0-06, US-P0-09** are architecturally unblocked (backend commands + schema ready) but may surface in UI post-v0.1.
6. Mobile stories **US-P0-12 – US-P0-15** remain deferred to a dedicated mobile packet after v0.1 ships.

---

## 2. Operational definition — what "replace tmux" actually means

The current tmux workflow produces this loop. v0.1 must reproduce all steps without tmux:

```
  ┌─ Mr. Zhang opens laptop
  │
  ├─ Reads overnight activity across 4-5 seats
  │   ← today: scrolls multiple tmux panes
  │   ← v0.1: Supervisor IM global feed shows yesterday's events
  │
  ├─ Picks today's next action (who to ping, what to approve)
  │   ← today: memory + file scan
  │   ← v0.1: action queue / inbox derived from real workitem + handoff state
  │
  ├─ Launches a seat's CLI (claude / gemini / codex)
  │   ← today: tmux split, exec 'claude' in the workspace dir
  │   ← v0.1: click seat in SeatDetail → Launch session → embedded xterm
  │
  ├─ Types instructions to the seat
  │   ← today: types in tmux pane
  │   ← v0.1: IM message targeted at seat → persisted to canonical_events → also injected into the live PTY
  │
  ├─ Seat replies in its CLI
  │   ← today: output scrolls in tmux pane
  │   ← v0.1: PTY bytes stream to xterm AND structured replies bubble in IM
  │
  ├─ Seat hits plan mode (claude-code ExitPlanMode, etc.)
  │   ← today: user reads the printed plan, presses '1' to accept
  │   ← v0.1: IM shows a Plan card with preview + Approve button
  │
  ├─ Seat writes new coordination markdown
  │   ← today: file appears on disk, never tracked until reconcile
  │   ← v0.1: reconcile can be triggered from UI; new doc appears in ArtifactsView
  │
  ├─ Supervisor reviews a delivered artifact
  │   ← today: opens markdown file, reads, comments in chat
  │   ← v0.1: typed artifact detail pane, inline comment threads
  │
  ├─ Supervisor issues verdict (accept / reject / reissue)
  │   ← today: writes acceptance markdown by hand
  │   ← v0.1: structured verdict + evidence_refs → persisted canonical_events
  │
  └─ Session ends
      ← today: tmux pane lingers until killed
      ← v0.1: Session completed event + checkpoint row + transcript log
```

Every row above is a discrete capability. The current state (commit `935e77a`) implements some fully, some partially, some not at all. Parts 3–5 enumerate each.

---

## 3. Coverage assessment (as of commit `935e77a`, 2026-05-09)

Legend: ✅ done · 🟡 partial · ❌ not done · ⛔ blocked · N/A deferred

### 3.1 L1 stories

| Story | Backend | Frontend | Verdict | Evidence / Gap |
|---|---|---|---|---|
| **US-P0-01** open SeatLoom → see what needs action | ✅ `list_workitems` + `list_handoffs` + derived inbox in `hydrateFromBackend` | 🟡 V1 `InboxView` hydrates from derived inbox (Blocked / InReview / Drifted workitems + Sent / Received / Working handoffs); seed is all-done so shows empty — expected; Dashboard counters fed from same derivation | 🟡 | Works for non-empty state; the **action-queue ranking logic** (recommend-next) is not yet built. |
| **US-P0-02** intent → structured proposal → confirm | ✅ `cmd_append_supervisor_message` persists intent | ❌ V1 `SupervisorCommandBar` (create / explain / evidence) still mock-only: builds fake proposal, never writes to canonical_events, never routes to a seat. Rebuilding the `/create` flow against a real proposal synthesiser is unstarted. | ❌ | Proposal pipeline: intent → backend extraction → structured proposal DTO → confirm → `workitems` INSERT + Handoff. |
| **US-P0-11** prompt-state surface | 🟡 schema 005 `prompt_instances` / `prompt_actions` tables; `cmd_list_active_prompts` + `cmd_append_prompt_action` | ❌ no UI detects or renders plan-mode / wizard prompts | ❌ | chan-10 Layer A transcript tail (`ClaudeAdapter::parse_transcript`) unstarted. Plan approval card UI unstarted. |

### 3.2 L2 read-only parity (v0.1 targets)

| Story | Backend | Frontend | Verdict | Gap |
|---|---|---|---|---|
| **US-P0-07** typed artifact review | ✅ `list_documents` + `get_document` + `list_document_sections` + `list_document_associations` | 🟡 V1 `ArtifactsView` now merges 291 documents with 15 runtime artifacts (commit `52dfad6`); typed detail pane exists in V1 for template/subtype but **inline comment thread** not wired to `review_threads` / `review_comments` | 🟡 | Section-level commenting → `cmd_create_review_thread` + `cmd_append_review_comment` needed. |
| **US-P0-08** Seat Card capability truth | ✅ `list_seats` + `list_role_bindings` + `list_delegations` | 🟡 V1 `SeatDetail` shows seat + role + bindings but **budget fields / accepted_input / constraints / attached_skills** all empty (schema 001 seats table has only capability_tags; other fields need schema extension or playbook join) | 🟡 | Gap is partly data-model (fields not in schema) + partly UI. |
| **US-P0-10** evidence search | ✅ schema 002 GIN tsvector on `documents.body_text` + tags; **no search command yet** | ❌ no UI search bar | ❌ | Needs `cmd_search_documents(query, template?, subtype?)` + a global ⌘F search overlay. |

### 3.3 L2 architecturally-unblocked (post-v0.1)

| Story | State |
|---|---|
| **US-P0-03** seat identity across projects | Backend ready (SeatIdentity + ProjectRoleBind); multi-project UI deferred until schema adds project_id FKs across workitems/artifacts/etc. |
| **US-P0-04** scoped delegation | Backend ready (`seat_delegations` table, `list_delegations`); UI to issue / close not built. |
| **US-P0-05** review-fail + reissue | Schema ready (review_threads + ReviewVerdictIssued event, AD-010); UI for verdict + evidence-linked reissue not built. |
| **US-P0-06** continuity pack | Checkpoint schema ready (schema 004); `ContinuityPreview` assembler not implemented. |
| **US-P0-09** tiered continuity preview | Depends on US-P0-06; deferred. |

### 3.4 Mobile (deferred)

US-P0-12 through US-P0-15 — dedicated mobile-companion packet post-v0.1. Not blocking tmux replacement.

### 3.5 tmux-workflow row coverage

From §2:

| Workflow row | Status |
|---|---|
| Read overnight activity feed | 🟡 IM global feed wired; ranking absent |
| Pick today's next action | 🟡 inbox derivation works; ranking / digest absent (US-P0-01 partial) |
| **Launch seat CLI from UI** | ❌ `cmd_launch_session` + `SessionTerminal.tsx` exist; **not mounted inside V1 SeatDetail** |
| Type instructions to seat | 🟡 IM → `cmd_append_supervisor_message` done; **PTY inject path exists but depends on live-session registry populated by launch UI → blocked by row above** |
| **Seat replies in PTY** | ❌ PTY bytes stream as Tauri events; **no `SeatResponse` event bridge**: IM shows only user-sent bubbles, agent output only visible in raw xterm |
| **Plan-mode approval** | ❌ chan-10 design issued; no impl |
| **Reconcile from UI** | ❌ `cmd_reconcile` command present; no button / shortcut / watcher |
| Typed artifact review | 🟡 reads work; inline commenting not wired |
| Issue verdict / reissue | ❌ UI missing |
| Session end → checkpoint | ❌ unwired |

### 3.6 Current P0 blocker

**White-screen crash on V1 main-shell render** (post commit `935e77a`). `TopErrorBoundary` pushed; awaiting Mr. Zhang's next run to capture the stack. Every other packet below is blocked on this fix.

---

## 4. Remaining packet list

Each entry below is a T3 task packet Lyra must author and dispatch. **Aegis does not author these.** Owner seats deliver; Flux verifies; Lyra accepts.

### Stage Gate A — **unblock SeatLoom itself**

| Packet | Owner | Deliverable | Acceptance |
|---|---|---|---|
| **A1**: diagnose + fix V1 main-shell white-screen (commit `935e77a`) | Nimbus | Root-cause the crash revealed by `TopErrorBoundary`; deliver a fix commit + test coverage for the failing path. | Tauri dev window renders V1 AppShell with real data for > 10 s without crash; ⌘K opens IM without crash; 3 tab switches (Inbox / WorkItems / Artifacts) without crash. Flux verifies with browser smoke. |

### Stage Gate B — **tmux replacement minimum**

All three packets below are required before any user can realistically do a SeatLoom-only seat session. Each one is small enough to deliver in one commit.

| Packet | Owner | Scope | Reuses |
|---|---|---|---|
| **B1**: mount `SessionTerminal` inside V1 `SeatDetail` | Mira | Add a "Sessions" tab within `components/SeatDetail.tsx`; list previous + live sessions; Launch button calls `cmd_launch_session` with seat's default runtime; selected session shows embedded xterm via the existing `app-v2/panel/SessionTerminal.tsx`; Kill button calls `cmd_kill_session`. Register seat→live-session mapping into `useLiveSessionsStore`. | `SessionTerminal.tsx` (153 lines), `useLiveSessionsStore.ts`, `api.launchSession/ptyWrite/ptyResize/killSession`. |
| **B2**: PTY output → `SeatResponse` canonical event bridge | Nimbus | In `src-tauri/src/commands/session_cmds.rs`, the output tokio task accumulates bytes per session and on a **silence window (default 3 s)** emits a `cmd_append_supervisor_message(actor_ref='seat:<seat_id>', event_type='SeatResponse', target_seat_id=null, content=<flushed chunk>)`. Skip if flushed chunk is empty or only ANSI control bytes. Add unit test for flush boundary + ANSI-only skip. | Existing `PtySession::subscribe`, `append_supervisor_message` repo method. |
| **B3**: reconcile trigger from UI | Mira | Add a "🔄 Reconcile docs" button at the top of V1 `ArtifactsView`; calls `api.reconcile()`; shows result toast (`scanned / inserted / updated / unchanged / failed / conflicted`); triggers `hydrateFromBackend` on success so the 291-doc list refreshes. | `cmd_reconcile` already exposed. |

Exit criterion for SG-B: A full round-trip — `Mr. Zhang opens SeatLoom → launches a claude session for Lyra → types 'summarize the prd' in IM targeted at Lyra → sees the message in xterm → sees Lyra's reply stream back into xterm AND appear in IM as a `SeatResponse` bubble → asks Lyra to write a test doc → reconciles from UI → sees new doc in ArtifactsView` — runs **without touching tmux**.

### Stage Gate C — **plan-mode + L1 complete**

| Packet | Owner | Scope | Authority |
|---|---|---|---|
| **C1**: ClaudeAdapter Layer A transcript tail | Nimbus | Implement per `docs/coordination/reviews/2026-05-08-aegis-cli-plan-mode-integration-design.md` §6.1: `ClaudeAdapter::detect_native_session_id`, `parse_transcript` (JSONL parser producing `TranscriptEntry`), background tokio task tailing `~/.claude/projects/<slug>/<sid>.jsonl` when a ClaudeCode-runtime session launches. On `ExitPlanMode` tool call → INSERT `prompt_instances(kind='plan_approval', evidence_ref=<plan_body>, evidence_preview=first 400 chars, available_actions=['approve','reject'])` + emit Tauri event `prompt:detected`. | chan-10 design doc (binding). |
| **C2**: Plan approval card in IM | Mira | When `prompt:detected` fires with a plan_approval prompt, render a Plan card bubble inside the IM (inline, scrolls with messages). Approve button calls `cmd_append_prompt_action(kind='approve')` → backend injects `'1\n'` into the PTY + emits `PromptResolved`. Reject injects `'2\n'`. | C1 delivered; schema 005 command surface. |
| **C3**: schema 005 CHECK extension + subtype allow-list | Nimbus | Add `'plan_approval'` to `prompt_kind` CHECK in schema 005 (new migration `006_plan_mode_authority.sql`). Add `'cli_plan'` subtype to `validate_subtype` T3 allow-list in `db/document_parser.rs`. | chan-10 §5. |

Exit criterion for SG-C: start a claude-code session for Lyra from SeatLoom; ask her to "plan a refactor"; observe the Plan card appear in the IM with the real plan body; click Approve; observe Lyra resume execution. Flux verifies via scripted plan-mode trigger.

### Stage Gate D — **L2 read-only parity + ship-readiness**

| Packet | Owner | Scope |
|---|---|---|
| **D1**: evidence-search overlay (US-P0-10) | Mira + Nimbus | Nimbus: add `cmd_search_documents(query, template?, subtype?, tags?)` using schema 002 tsvector + tag array. Mira: ⌘F global overlay with template/subtype filters; results show as a list; click opens artifact detail. |
| **D2**: section-level artifact commenting (US-P0-07 completion) | Mira + Nimbus | Nimbus: `cmd_create_review_thread(target_kind, target_id, anchor, title)` + `cmd_append_review_comment(thread_id, body)`. Mira: inline "Add comment" affordance on each document section in ArtifactsView detail; right-rail thread list. |
| **D3**: Seat Card budget + constraints fields (US-P0-08 completion) | Nimbus + Mira | Nimbus: schema 007 extending `seats` with budget / accepted_input_types / output_types / attached_skills. Backfill from `project_role_bindings.constraints` where possible. Mira: Seat Card surface expands to show these fields with per-project overlay. |
| **D4**: session-end → checkpoint + `CheckpointCreated` event | Nimbus | When `cmd_kill_session` / natural PTY exit fires, assemble a Checkpoint row (summary fields filled with last-N-lines tail excerpt, tier0 basic session metadata, branch / last_commit captured via `git` helper in `seatloom-core::git`). |
| **D5**: header polish — remove mock dots, counters from real state | Mira | V1 top-bar "阻塞 / 待决 / 进行" counters and project switcher today read from mock-like sources; wire them to derived state from hydrated store. |

### Stage Gate E — **v0.1 ship acceptance (Flux + Lyra)**

| Packet | Owner | Scope |
|---|---|---|
| **E1**: commit-pinned full v0.1 acceptance | Flux | Follow `docs/acceptance-spec-v1.1.md`; verify every SG-A / SG-B / SG-C / SG-D packet against its pinned commit; produce a T5 acceptance_review spanning the whole stage-gate set. |
| **E2**: v0.1 release note artifact | Lyra | T1 authority subtype `release_note` documenting the v0.1 shipped surface, known gaps, and deferred-to-v0.2 items (mobile, P1 stories, US-P0-03/04/05/06/09 UI, ranking logic, semantic retrieval). |

---

## 5. Ownership matrix

| Seat | Packets | Role contract (from COORDINATION_RULES.md + seed bindings) |
|---|---|---|
| **Nimbus** | A1 diagnose, B2 PTY bridge, C1 ClaudeAdapter, C3 schema migration, D1/D2/D3 backend, D4 checkpoint | Rust / Tauri / infra — authority docs `architecture-design.md`, `architecture-decisions.md`; constraint `no-runtime-widening, infrastructure-only`. |
| **Mira** | B1 SessionTerminal mount, B3 reconcile button, C2 plan card UI, D1/D2/D3 UI, D5 header polish | UX / React / TypeScript — authority `ux-spec-v1.1.md`, `interaction-spec-v1.1.md`; constraint `no-IA-redesign, ui-contract-only`. |
| **Flux** | verification of every delivered packet | QA / verification — authority `acceptance-spec-v1.1.md`; constraint `read-only-by-default, no-ui-redesign`. Flux may apply bounded UI-wiring fixes (missing asset / config) per COORDINATION_RULES.md §11. |
| **Lyra** | T3 packet authorship for every entry above, T5 acceptance per packet, E2 release note | PO / coordination / acceptance — authority `prd-v0.5.md`; constraint `file-first-coordination`. |
| **Aegis** | this review + any subsequent supervision review; no direct packet implementation in this cycle | supervision / stage-gate — authority `PRODUCT_TRUTH.md`. |

---

## 6. Sequencing DAG

```
SG-A (blocker)
  A1 diagnose white-screen                   [Nimbus]
      │
      ▼
SG-B (tmux min)     ──── can run in parallel ────
  B1 SessionTerminal mount        [Mira]
  B2 PTY→SeatResponse bridge      [Nimbus]
  B3 Reconcile trigger            [Mira]
      │
      ▼
SG-C (L1 complete)
  C3 schema/subtype migration     [Nimbus]  ← prerequisite
      │
      ▼
  C1 ClaudeAdapter Layer A        [Nimbus]
      │
      ▼
  C2 Plan approval card           [Mira]
      │
      ▼
SG-D (L2 parity)   ──── can run in parallel ────
  D1 evidence search              [Nimbus + Mira]
  D2 section commenting           [Nimbus + Mira]
  D3 Seat Card fields             [Nimbus + Mira]
  D4 checkpoint on session end    [Nimbus]
  D5 header polish                [Mira]
      │
      ▼
SG-E (ship)
  E1 Flux acceptance               [Flux]
  E2 Lyra release note             [Lyra]
```

Each ▼ edge is a **delivery + verification + acceptance** cycle before the downstream packet may start.

---

## 7. Coordination flow (mandatory per COORDINATION_RULES.md)

For **every** packet A1 – E2:

1. **Lyra drafts** a T3 task packet under `docs/coordination/tasks/<owner-seat>/`. The packet MUST cite this review id in `depends_on`: `2026-05-09-aegis-mvp-gap-to-tmux-replacement`. Packet id follows `<OWNER>-2026-05-09-<slug>-v1`.
2. **Owner seat delivers** per the reporting format in their packet. Delivery produces a `*-delivery-v1.md` sibling doc pinning the commit hash.
3. **Flux verifies** commit-pinned: runs build, runs the acceptance checklist from the delivery doc, writes either a `HOLD` or `PASS` verification packet (T3 subtype `verification`).
4. **Lyra accepts** with a T5 `acceptance_review` citing the verified commit. Acceptance file goes under `docs/coordination/acceptance/`.
5. **Lyra updates** `docs/coordination/MEMORY.md` with the one-line closure + updates `docs/coordination/memory/YYYY-MM-DD.md`.
6. **Only after acceptance**, the next packet in the DAG becomes dispatchable.

Stage-gate-level decisions (SG-A clear / SG-B clear / etc.) are made by **Aegis** once all packets in a gate reach accepted status, recorded as a T5 `gate_decision`.

---

## 8. Immediate actions for Lyra

On receipt of this review:

1. **Acknowledge** in terminal: `[Lyra -> Aegis] MVP gap review 2026-05-09 received.`
2. **Draft packet A1** first (highest priority — blocks everything else). Dispatch to Nimbus.
3. **Queue packets B1 / B2 / B3** as drafts, ready to dispatch the moment A1 reaches Flux PASS.
4. **Sequence packets C / D / E** internally; they do NOT need to be drafted yet but the ownership matrix in §5 is binding.
5. Propose any **refinement questions** on this review to Aegis before starting authorship — once Lyra accepts, the review is the authority and packets must remain consistent with it.

---

## 9. Out of scope for this review (flagged, not re-argued)

- Mobile companion stories US-P0-12 – US-P0-15 — deferred post-v0.1 per §1.4.
- P1 / P2 / P3 stories in PRD §5 — explicit non-goal of v0.1.
- chan-04 (Session field ownership) and chan-05 (Playbook binding scope) — design-debt items in `AEGIS-2026-04-30-pending-changes-register.md`; unrelated to tmux replacement.
- Semantic retrieval (L3, pgvector) — post-v0.1 per chan-10 §8.
- Claude-Code hooks (chan-10 Layer B, `seatloom install-claude-hooks`) — post-v0.1; Layer A alone is sufficient for plan-mode.

---

## 10. Done Definition for this review

- [x] MVP baseline re-anchored to PRODUCT_TRUTH + PRD v0.5 + v0.1 scope directive.
- [x] Operational tmux-replacement workflow itemised (§2).
- [x] Current state assessed per-story and per-workflow-row (§3).
- [x] Every remaining packet enumerated with owner + scope + authority citation (§4, §5).
- [x] Sequencing DAG with stage gates (§6).
- [x] Coordination flow pinned to COORDINATION_RULES.md §3 writeback rule + Flux commit-pinning (§7).
- [x] Immediate hand-off to Lyra defined (§8).

---

*MVP Gap Review by Aegis · 2026-05-09 · Status: issued. Lyra to acknowledge + begin A1 drafting.*
