# Reconciliation Note: Nimbus arch supplement × Lyra product supplement — cross-point map for Aegis joint review

| Field | Value |
|---|---|
| template | T4 |
| subtype | design_proposal |
| id | 2026-05-09-nimbus-arch-x-lyra-product-supplement-reconciliation-v1 |
| status | issued |
| author | nimbus |
| date | 2026-05-09 |
| version | v1 |
| to | aegis (joint review), lyra (FYI) |
| priority | P0 |
| depends_on | `docs/coordination/reviews/2026-05-09-nimbus-seatloom-full-arch-design-v1.md` (Nimbus arch supplement, commit `dd8758c`), `docs/coordination/reviews/2026-05-09-lyra-seatloom-full-product-design-v1.md` (Lyra product supplement, commit `3ccbd6d`), `docs/prd-v0.5.md`, `docs/architecture-decisions.md` (AD-007, AD-010 — joint supersession candidates), `infra/postgres/schema/001..005_*.sql` |
| tags | reconciliation, cross-link, joint-review, schema-numbering, event-first, realtime-tiers, multi-project, historical-backfill, AD-007-supersession |
| acceptance owner | Aegis |
| concurrency rule | Reconciliation only — no new product or architecture content. Captures where the two 2026-05-09 supplements converge, where they differ, and (§2.1) one factual correction Nimbus owes Lyra. Aegis uses this as the joint-review index; a full merged design doc is not proposed. |

---

## 0. Why this document exists

Lyra's A3 acceptance (2026-05-09 UNCONDITIONAL PASS, commit `b150089`) named three explicit cross-points that Aegis needs aligned before B1 dispatches:

> "B1 派发前请先把 arch supplement (docs/coordination/reviews/2026-05-09-nimbus-seatloom-full-arch-design-v1.md @ dd8758c) 与 Lyra 产品 supplement (@ 3ccbd6d) 的交叉点 (schema 008 vs 006 编号, realtime tier 切分, 多项目 project_id 时机) 等 Aegis 联审。"

I re-read Lyra's supplement after A3 closed and spotted two additional cross-points (event-first projection vs hybrid two-channel; AD-007 joint supersession wording). Filing this note now so Aegis can review both parent docs + this cross-map together, rather than re-derive the intersections from raw text.

This note explicitly does NOT redesign either parent. Each parent document stands on its own; this is a map.

---

## 1. Where the two supplements converge independently

These six points were reached independently in both docs. No reconciliation needed — they reinforce each other.

| # | Convergence point | Nimbus anchor | Lyra anchor |
|---|---|---|---|
| 1 | **Automatic file watcher on `docs/coordination/**` supersedes AD-007's "no daemon in MVP" stance** | §2 (notify + 500 ms debounce + per-file partial reconcile + journaled queue) | §1.1 ("AD-007's original 'no file watcher in MVP' is **superseded for v0.1** by tmux-mirror R4") |
| 2 | **Watcher granularity: `docs/coordination/**/*.md` only, not the whole repo tree** | §2 recommendation (glob filter) | §2.3 Watcher granularity ("Watching the whole repo causes false reconcile triggers") |
| 3 | **Historical back-fill is required for v0.1 to ship with non-empty Inbox / WorkItem views** | §5 Option C (bootstrap + steady-state, both idempotent) | §4.4 Option B (full retroactive with `synthetic=true` flag) |
| 4 | **Back-fill is keyed by `document.id` for idempotency** | §5 recommendation ("each upsert keys on document.id") | §4.4 implementation note ("Idempotency: synthetic events are keyed by (document_id, event_type)") |
| 5 | **Multi-project `project_id` must become a first-class column on `workitems` / `sessions` / `handoffs` / `canonical_events`** | §3 Option A | §3.3.1 Option B |
| 6 | **NOT NULL with default/backfill, not nullable** | §3 recommendation | §3.3.1 Option B recommendation |

These six are the solid bedrock. Aegis can ratify them as single joint decisions without re-deriving from either supplement.

---

## 2. Where the two supplements differ — correction + conflicts + alignment candidates

### 2.1 CORRECTION — schema migration numbering: Lyra is correct, mine is wrong

**Fact**: current schema on disk is `001..005`:

```
infra/postgres/schema/
  001_seatloom_core.sql
  002_document_authority.sql
  003_write_ingest_reconcile.sql
  004_operational_review_and_continuity.sql
  005_prompt_and_channel_action_authority.sql
```

My arch supplement §3 proposed `008_project_isolation.sql`. That was a miscount — I skipped 006 and 007. Lyra's §3.3.1 correctly proposes `006_multiproject_scope.sql`.

**Resolution (binding)**: the multi-project migration ships as **`006_multiproject_scope.sql`**. My arch supplement §3 and Appendix A should be read with `006` substituted for `008`. A formal errata edit to the arch supplement can follow if Aegis wants the file cleaned up — or the correction can just stay captured here; Aegis picks.

If event-first projection (§2.3 below) also lands in v0.1, it would be `007_*.sql` in the same window. This is what Lyra forecasted in §8 "PostgreSQL schema 006 (multi-project) and 007 (event-first projection if Option C accepted)".

### 2.2 Multi-project `project_id` timing: both docs say v0.0.1/v0.0.2, slight scheduling gap

| Source | Timing |
|---|---|
| Nimbus arch §3 + §7 sequencing | "§3 schema migration 008 [read: 006] — cheap; unlocks everything else. First in the sequence." |
| Lyra product §7 open question #3 | "ship `006_multiproject_scope.sql` in v0.0.1, v0.0.2, or v0.1? My recommendation is v0.0.1 because every subsequent table interaction assumes `project_id`, but the SG-A packet list (gap review §4) does not currently include this migration." |

**Status**: no conflict. Both want it first. Lyra flags that it wasn't in the SG-A packet list; A3 is now closed and SG-A is done, so the next dispatch window (post-A4-β, pre-B1) is the natural slot. Recommendation: dispatch `006` between A4-β landing and B1 dispatch. It is a backend-only, 600–900 LOC packet that blocks nothing the UI needs now and unblocks everything multi-project-facing later. **Aegis decides**.

### 2.3 Data lifecycle — event-first (Lyra) vs hybrid two-channel (Nimbus): orthogonal, not conflicting

Lyra's §1.4 Option C ("reconcile writes only `canonical_events`; `workitems` / `handoffs` / `inbox` are materialized views") and my §1 (hybrid two-channel — byte-stream liveness + adapter-structured events) address **different slices of the same pipeline**:

| Slice | Source of truth | Owned by |
|---|---|---|
| **Doc lifecycle** (T3 issued → delivered → T5 accepted) | `documents` table rows | Lyra §1.4 Option C — reconcile writes events, workitems is a view |
| **Runtime lifecycle** (session active / idle / response emitted / tool called) | PTY byte stream + adapter JSONL tail | Nimbus §1 hybrid two-channel |

Both produce `canonical_events` rows. The event table is the junction. Lyra's §1.4 handles `WorkItemIssued` / `WorkItemDelivered` / `WorkItemAccepted`; my §1 handles `SessionActive` / `IdleWindowEntered` / `SeatResponseEmitted`. Neither proposal preempts the other; they are complementary EventType extensions.

**Where they would conflict**: if `workitems` is a materialized view (Lyra §1.4 Option C) AND `project_id` is a direct column on `workitems` (both docs §3), then the view definition must include the `project_id` derivation rule. Mechanical — `project_id` is carried on the `canonical_events` row per §3, the view propagates it. Not an architectural problem; a view-definition note for Nimbus to write up when the materialized-view machinery lands.

**Recommendation for Aegis**: ratify both. Lyra §1.4 Option C (event-first for doc lifecycle) + Nimbus §1 hybrid two-channel (PTY + adapter for runtime lifecycle). Document the view-definition requirement as a follow-up packet spec, not a parent-doc amendment.

### 2.4 Real-time tiers — how Lyra's T-Live/T-Near/T-Batch maps onto Nimbus's throttle + LISTEN/NOTIFY + batch emit

Lyra §2.3 defines tiers by latency budget:

| Lyra tier | Latency | Mechanism (Lyra) |
|---|---|---|
| T-Live | ≤ 1 s | Tauri event → component subscriber, no DB roundtrip |
| T-Near | ≤ 5 s | DB write + Tauri event → store update → UI re-render |
| T-Batch | manual or scheduled | reconcile run; explicit user trigger |

Nimbus §4 defines mechanisms (not tiers) for delivering realtime signals. The two map cleanly:

| Lyra tier | Surface examples | Nimbus mechanism |
|---|---|---|
| **T-Live** (≤ 1 s) | SessionTerminal xterm output; Supervisor IM own-bubble echo | Nimbus §4 `session:output` throttle policy (4 KiB or 16 ms whichever comes first). Latency bound: worst-case 16 ms + render. Well inside 1 s. |
| **T-Near** (≤ 5 s) | Supervisor IM target-seat keystroke delivery; plan-mode approval card; WorkflowPanorama event update; Inbox row insertion | Nimbus §4 `app.emit(ledger:new_event)` per DB insert. For cross-process origins (watcher, CLI), Nimbus §4 PG `LISTEN/NOTIFY seatloom_events` → Tauri backend re-emit. Burst mode: `ledger:new_events_batch` coalesces ≥ 10 events within 100 ms. Latency bound: watcher debounce (500 ms) + DB write + emit ≈ 600–800 ms steady state. Inside 5 s. |
| **T-Batch** | initial Inbox hydration on app launch; 30-day panorama; search | Nimbus §4 not needed — these are pull-on-mount, no event plumbing |

**Aegis check**: do the numeric budgets in Lyra's table (1 s / 5 s) agree with the implementation Nimbus proposes? Yes — Nimbus's throttle is 16 ms (30× tighter than Lyra's T-Live budget) and Nimbus's end-to-end watcher→UI path is ~600–800 ms (6–8× tighter than Lyra's T-Near budget). No re-spec needed. Lyra's Q2 ("are 1 s / 5 s the right cutoff, or 500 ms / 3 s?") is answered by the implementation envelope: Nimbus's design already beats 500 ms / 3 s comfortably, so Lyra can tighten the budgets safely if she prefers.

**My recommendation for Lyra's Q2**: keep Lyra's 1 s / 5 s — they give implementation headroom for less-ideal machines (slow SSD, throttled CPU) without pulling the spec tighter than the typical-case envelope. If Aegis prefers tighter, 500 ms / 3 s is still implementable with the same mechanisms.

### 2.5 Message lifecycle — Lyra §1.5 Option B two-phase matches Nimbus §1 liveness separation

Lyra §1.5 recommends **Option B (two-phase: persist event first, then dispatch to tmux)** for Supervisor IM message delivery. This matches exactly what Nimbus §1 enables — the event persistence path doesn't depend on the PTY byte stream. **No conflict. No amendment needed.**

Nimbus can add one implementation note: Lyra's §1.5 Option C (three-phase: persist → dispatch → confirm via mirror) is feasible in v0.2 using the §1 byte-stream liveness pipeline (`SessionActive` events within N ms of dispatch would count as delivery confirmation). Not proposed for v0.1. Lyra already noted this ("over-engineering for v0.1").

### 2.6 AD-007 supersession — both docs independently state it; suggested joint wording

Both supplements name AD-007's "MVP 不做文件监听 (无 daemon)" as superseded:

- Nimbus §2 anchors: *"contradicts AD-007's 'no daemon' stance"*
- Lyra §1.1: *"AD-007's original 'no file watcher in MVP' is **superseded for v0.1** by tmux-mirror R4"*
- Lyra §6.3: *"AD-007 (reconciliation triggers): explicitly **superseded** by tmux-mirror R4 + this supplement §1 + §2 (file watcher is now P0; manual is fallback)."*

**Proposed joint amendment to AD-007**, for Aegis to apply in `architecture-decisions.md`:

> **2026-05-09 amendment (supersedes "MVP 不做文件监听" clause)**:
> Under the tmux-mirror architecture (commit `a5998c1`) and the joint supplements `2026-05-09-nimbus-seatloom-full-arch-design-v1.md` §2 + `2026-05-09-lyra-seatloom-full-product-design-v1.md` §1, v0.1 ships an automatic file watcher on `docs/coordination/**/*.md` with 500 ms debounce and per-file partial reconcile. The three original trigger points (app launch, pre-pipeline, manual button) are preserved as fallbacks; the watcher is the primary trigger. All other AD-007 clauses (PostgreSQL write-path contract, idempotency via `body_digest`, `reconcile_runs` / `reconcile_items` observability) remain in force.

Aegis owns the actual AD-007 edit; this is the wording suggestion.

### 2.7 AD-010 generalization — Lyra proposes it, Nimbus §5 depends on it

Lyra's §1.4 Option C recommendation notes: *"AD-010 already commits to event-first for review-failure. Generalizing this rule to all object lifecycles avoids two write paths."* My §5 backfill derivation rules do the same generalization implicitly (deriving `ReviewVerdictIssued` from T5, and analogous lifecycle events from T3).

If Aegis accepts Lyra §1.4 Option C, AD-010 should be amended from *"review-failure is event-first"* to *"all object lifecycles are event-first; review-failure is one case"*. Again, Aegis owns the edit.

---

## 3. Answers to Lyra's §7 open questions (Nimbus tech-arch perspective, for Aegis review)

Lyra's supplement §7 lists five questions addressed to Aegis. Here are my tech-arch responses — Aegis makes the final call.

### Q1. Event log backbone: is event-first (§1.4 Option C) acceptable for v0.1?

**Yes**, with a caveat: event-first is architecturally cleaner but requires materialized-view machinery. In Rust, this is `seatloom-core/src/data_engine/views.rs` maintaining denormalized `workitems` / `handoffs` / `inbox` tables from event replay. SQL-side, the cheapest path is trigger functions on `canonical_events` INSERT — no view re-materialization needed, just an UPSERT per event. Complexity delta vs non-event-first: +300–500 LOC for the trigger + view-invariant tests; no additional LOC on reader side (queries stay the same shape).

### Q2. Real-time tier boundaries: 1 s / 5 s, or 500 ms / 3 s?

See §2.4 above. Implementation beats 500 ms / 3 s today. Keep Lyra's 1 s / 5 s as spec for machine-dependent safety margin; tighten if Aegis prefers.

### Q3. Multi-project schema timing: v0.0.1, v0.0.2, or v0.1?

Between A4-β completion and B1 dispatch (see §4 sequencing below). Labeling it as v0.0.2-α or v0.0.1.1 is Aegis's call; what matters is it lands before any code path assumes single-project.

### Q4. Historical back-fill timing: part of v0.1 release, or v0.0.2 utility command?

**Utility command at v0.0.2**. Reason: the derivation logic (Lyra §4.4 mapping table + Nimbus §5 mapping table) needs to be exercised against the real 291-doc corpus before it's baked into the release. Wrapping it as `cmd_backfill_historical_events(dry_run: bool)` lets Lyra/Aegis validate the derivation before v0.1 locks it in. Same code is called from the watcher steady-state path; utility command just iterates it over all existing documents.

### Q5. Plan card placement — does AD-012 "first-class state" formally exclude toasts?

Reading AD-012 literally: *"Prompt 状态扩展 (SessionStatus)... PromptBlocked 变体"* and *"作为一等架构状态建模"* — yes, first-class means structural UI surface, not ephemeral overlay. Toast rejected. Lyra's §5.2.2 Option A (inline IM card) + Option C (PromptInbox back-pocket) is consistent with AD-012. No AD-012 amendment needed.

---

## 4. Proposed sequencing after joint review (combines Nimbus §7 + Lyra §8)

Revised from Nimbus arch §7 in light of Lyra's timing recommendations and this reconciliation:

| Slot | Packet | Scope | Owner |
|---|---|---|---|
| **next** | `006_multiproject_scope.sql` + repo/DTO updates | Backend-only; §3 of Nimbus arch + §3.3.1 of Lyra product | Nimbus |
| **then** | B1 (send-keys / bidirectional write path) | Per existing SG-B plan + §6b `capture-pane` backfill on attach (Nimbus arch §6b-B) | Nimbus (arch), Mira (UI) |
| **parallel to B1** | `cmd_backfill_historical_events` utility + steady-state derivation module | §5 Nimbus + §4.4 Lyra; exercises derivation on real corpus before v0.1 | Nimbus |
| **after B1** | Event-first projection migration `007_event_first_views.sql` + trigger functions | §1.4 Lyra Option C + generalization of AD-010 | Nimbus |
| **after 007** | File watcher `seatloom-core/src/watcher.rs` | Nimbus §2 + Lyra §1.1 (AD-007 supersession lands here) | Nimbus |
| **after watcher** | Realtime throttle + `LISTEN/NOTIFY` | Nimbus §4 + Lyra §2.4 tier mapping | Nimbus |
| **final v0.1 gate** | Joint Lyra/Aegis acceptance against full PRD v0.5 + supplements | — | Lyra, Aegis |

Total forecast: ~3 500–4 800 LOC + two schema migrations + one bootstrap command. Roughly 5–7 A3-sized packets. Matches Lyra's "v0.0.1/v0.0.2/v0.1 progression acceptable" framing from the tmux-mirror architecture.

---

## 5. Explicit non-decisions (to keep joint-review scope tight)

This reconciliation note does NOT:

- Decide the final AD-007 / AD-010 amendment text — it proposes wording; Aegis owns the file edits.
- Decide the final event view schema — materialized vs trigger-based is a Nimbus implementation-packet decision after Aegis ratifies Lyra §1.4 Option C.
- Redesign either parent supplement. Both stand as written (with the §2.1 `008 → 006` correction applied to Nimbus arch §3 + Appendix A).
- Dispatch any implementation packet. Lyra dispatches per usual after Aegis review.

---

*Reconciliation filed by Nimbus · 2026-05-09 · Cross-maps Nimbus arch supplement @ `dd8758c` and Lyra product supplement @ `3ccbd6d`. One factual correction owed (§2.1 schema 008→006). Answers Lyra §7 Q1–Q5. Proposes AD-007 + AD-010 amendment wording for Aegis. Sequencing after joint review: 006 → B1 + backfill utility → 007 → watcher → realtime.*
