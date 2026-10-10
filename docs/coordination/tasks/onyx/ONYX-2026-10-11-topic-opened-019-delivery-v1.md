# Delivery: migration 019 — TopicOpened (the schema half of governed topic creation)

[Onyx -> Lyra/Aegis] `disp-019-onyx`: migration 019 landed — `canonical_events_event_type_vocab`
widened 27 → 28 with `TopicOpened`, the complete full list, cross-checked verbatim against
Nimbus's pinned Rust `EventType::ALL`. This is the remaining half of Aegis ruling 1: until 019
is landed AND live-applied, `open_topic` refuses at the database CHECK (escalated, not worked
around). Additive + idempotent, zero data rows, scratch-verified.

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | implementation-delivery |
| id | ONYX-2026-10-11-topic-opened-019-delivery-v1 |
| status | delivered |
| author | onyx |
| date | 2026-10-11 |
| version | v1 |
| responds to | governed dispatch `disp-019-onyx` (overnight blocker — TopicOpened, 019) |
| packet / contract | `docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-11-topic-opened-019-v1.md` (Lyra sign-off 2026-10-11, carried verbatim); ruling `AEGIS-2026-10-11-topic-creation-and-verdict-wake-ruling.md` (ruling 1); Nimbus's delivery `NIMBUS-2026-10-11-open-topic-delivery-v1.md` @ `packet/nimbus/open-topic` @ `29df563` (Flux-verified, `EventType::ALL` pinned 28) |
| base branch | `track/infra-foundation` @ `85bcc06` (track advanced mid-lane: memory tools + waker fix + acceptance containment integrated, 015–018 landed on track via `eceb264`; merge-base with this branch = `85bcc06`) |
| delivery branch | `packet/onyx/topic-opened-019` @ `deb946e` |
| files changed | `infra/postgres/schema/019_topic_opened.sql` (new), `scripts/apply-db.sh` (+1 POST_SEED entry), this doc |
| rust / tsx touched | none (the Rust half is Nimbus's, already landed + Flux-verified) |
| live db touched | none — live READ only (vocab census: 27 values, no TopicOpened — as the contract predicts); all writes on scratch DBs, dropped after |

---

## 0. Why this migration exists (the overnight chain behind it)

Production had **no path to create a topic**: `memories.topic_id` is `NOT NULL REFERENCES topics(id)`, `topics` held **0 rows**, and no MCP tool created one — so every `remember` (and every seat's forward-recording obligation, effective since `disp-fwdrec-onyx`) failed on the foreign key. Aegis ruling 1 commissioned `open_topic` (Nimbus, delivered + Flux-verified) and required the event-type decision with both costs written down: **ADD `TopicOpened`** (no honest reuse candidate; in an append-only ledger one word for two meanings can never be split once rows carry both). Lyra signed the addendum. The Rust side pins 28; **019 is the PG half**.

## 1. What shipped

1. **`infra/postgres/schema/019_topic_opened.sql`** — guarded widening of `canonical_events_event_type_vocab` 27 → 28: DROP+ADD of the **COMPLETE 28-value list** (013's pattern), guard checks for `'TopicOpened'` being absent. The list carries every value the previous migrations installed — 010's 12, 012's 3, 013's 3, 014's 3, 017's withdrawal, 015's 4, 018's 1 — plus `TopicOpened`, and is **cross-checked value-by-value AND in declaration order against Nimbus's pinned `EventType::ALL: [EventType; 28]` on `packet/nimbus/open-topic`** (diff-identical, 28/28).
2. **`scripts/apply-db.sh`** — POST_SEED registration after 018 with a rule-based reason (widens ONE vocab CHECK that 010/012/013/014/017/015/018 created, which exists only post-seed; canonical order 018 → 019). `bash -n` clean; completeness guard passes.
3. **The stale-list trap — avoided and proven.** Nimbus's ready draft (`.scratch/019_topic_opened.sql` on his branch) almost shipped a stale list that would have silently reverted 015/018's memory values; it was caught in his scratch proof. The list landed here is verified to preserve them (below), and the migration header records the trap.

Contract compliance: **no ref_type widening** (`'topic'` exists since 010), no new columns, no new tables, no daemon; zero data-row writes; `crates/**` untouched.

## 2. Verification — scratch only (live read-only)

Environment: `postgres:16-alpine` runner container sharing the `seatloom-postgres` network; scratch DBs `seatloom_scratch_019` (full branch build, track state) and `seatloom_scratch_019_inter` (001–012 only, a 15-value intermediate); both dropped after. Live census (read-only): **27 values, no `TopicOpened`, `memories` present, `topics` 0 rows** — exactly the contract's premise.

1. **Clean apply from the track state (27, post-018)** — full `apply-db.sh` build on the branch: exit 0, 019 applied after 018 in canonical order, zero errors.
2. **Vocab final state** — 28 values; `TopicOpened` present; **015/018's `MemoryResolved` and 017's `AcceptanceCancelled` preserved** (the stale-list revert did not happen).
3. **Verbatim match to the Rust pin** — the PG CHECK's 28 values vs `EventType::ALL` on `packet/nimbus/open-topic`: `diff` empty (set-identical; declaration order identical too).
4. **Positive** — `INSERT ... event_type='TopicOpened'` → `INSERT 0 1` (previously CHECK-refused; this is the insert `open_topic`'s row+event transaction needs).
5. **Negative** — an unknown type (`TopicOpenedd`) still refused by the CHECK.
6. **Intermediate-state applicability** — 019 applied onto a 15-value DB (001–012 state): exit 0, → 28 values with `TopicOpened`. The full-list writing makes it order- and state-independent.
7. **Idempotency** — second apply of `019_topic_opened.sql`: `pg_class=627 pg_constraint=246 pg_attribute=4177` **unchanged**.

## 3. Wiring note (carried from the contract)

Until 019 is **landed AND applied to live**, `open_topic` refuses at the CHECK with the vocabulary violation — the tool is deployed but not live-usable (Nimbus escalation #1, correct). Live is at 27 (015–018 applied out-of-band via the operator's `scripts/apply-schema-to-live.sh`); **019's live apply is Aegis's act**. The gate `ev-typed-memory-tools-landed` requires a REAL live round-trip, not a scratch proof — its provenance remains under verification with Aegis; nothing in this delivery claims it.

## 4. Completion flow

1. ✅ Committed to `packet/onyx/topic-opened-019` @ `deb946e` (commit carries the dispatch id).
2. ✅ This delivery doc.
3. Gate event `ev-019-topic-opened-landed` (ArtifactCreated, exact id: subject artifact `art-019-topic-opened`, actor `seat-onyx-001`, context `disp-019-onyx`).
4. `complete_dispatch(disp-019-onyx, outcome="done")`.
5. **Flux self-request, commit-pinned** (branch `packet/onyx/topic-opened-019` / SHA `deb946e` / base `track/infra-foundation` @ `85bcc06`), per the verdict-id contract (Aegis ruling 2): the verdict event id is **pre-declared** — `ev-019-topic-opened-flux-verdict` — declared in the Flux payload as the id Flux MUST emit `ReviewVerdictIssued` with, and a **gated self-dispatch** (to `seat-onyx-001`, dependency edge on that event id) is enqueued for the carding step. Implementation note: the tool-side minting (`enqueue_dispatch(verdict_event_id=…)`) lives on Nimbus's `open-topic` branch and is not yet on track — this seat's deployed binary predates it — so the id is minted seat-side by convention and declared in the payload; the mechanism (dependency edge as ledger data, 013's blocked→ready trigger waking the requester) is exactly the ruling's.
6. The acceptance plan for this card: Lyra dispatches it when the verdict lands; the gated self-dispatch picks it up and cards via `request_acceptance`. If this seat's binary is still the pre-plan one at that point (the deployment gap escalated as `ev-haW7F7e-rRARRwehXYUhX`), the carding is recorded as blocked — not bypassed with a plan-less card.
7. The seat never merges its own work.

## 5. Addendum — forward-recording step of THIS completion (recorded, not silently skipped)

The forward-recording protocol (effective since `disp-fwdrec-onyx`) makes a `remember` record part of every dispatch completion. For THIS dispatch the step **could not execute**: this seat's deployed MCP binary predates the memory tools (`searchTools("remember")` = empty; the seat process holds the pre-01:33 image — the same deployment gap escalated as `ev-haW7F7e-rRARRwehXYUhX` and documented in Aegis's ruling: the binary was rebuilt 01:33 but the 6 seat processes predate it). The protocol forbids a bypass file ("入口只走 remember 工具,不得旁路写文件"), so nothing was written to a side file. **What would have been recorded, for the first `remember` once the tool is live on this seat** (candidates, 宁缺勿滥 — each satisfies changes-a-future-action + not-verbatim-elsewhere + nameable evidence):

1. **failure** (quoted, evidence: this delivery §2.2/§2.7 + the ledger): "a full-list vocab widening is load-bearing — Nimbus's scratch draft of 019 carried a stale list that would have silently reverted 015/018's memory event types; caught in his scratch proof. Any future vocab widening must be verified to preserve every pre-existing value."
2. **decision** (inferred, evidence: §2.3): "when the tool-side verdict-id minting is not yet on track, the seat mints the verdict event id by convention and declares it in the Flux payload; the gated self-dispatch's dependency edge is the mechanism either way."
3. **unresolved** (quoted, evidence: §4.6 + `ev-haW7F7e-rRARRwehXYUhX`): "this seat's MCP process still runs the pre-memory-tools binary — remember AND the acceptance_plan parameter are both unavailable until the seat processes are restarted onto the rebuilt image; every carding step and every forward-recording obligation is blocked on that ops act (Aegis lane)."
