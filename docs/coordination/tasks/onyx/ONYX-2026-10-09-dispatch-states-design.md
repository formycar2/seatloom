# Design proposal: governed-dispatch terminal / failure states (schema evolution)

[Onyx -> Lyra] Schema-evolution design for the governed-dispatch (G19) status
domain, in response to governed dispatch `disp-onyx-states-design`. **Docs only**:
this proposes the shape of a future migration `013` and carries its exact SQL, but
**creates and applies nothing** and **touches no live database**. The vocabulary
widenings below extend the Lyra-owned value-domain contract and therefore require
Lyra sign-off before any migration is written — reported here, not silently
reconciled.

| Field | Value |
|-------|-------|
| template | T4 |
| subtype | design_proposal |
| id | ONYX-2026-10-09-dispatch-states-design |
| status | pending_review |
| author | onyx |
| date | 2026-10-09 |
| version | v1 |
| reviewer | lyra |
| scope | `infra/postgres/schema/012_governed_dispatch.sql`, `docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-08-governed-dispatch-v1.md`, `crates/seatloom-mcp-emit/src/dispatch.rs`, `crates/seatloom-mcp-emit/src/scheduler.rs` |
| governed dispatch | `disp-onyx-states-design` |
| tags | onyx, p0, g19, governed-dispatch, schema, status, retry, failure, design |

---

## 0. Scope and hard constraint

This is a **design proposal**, not an implementation. Per the governing dispatch:

- **No migration is created.** `infra/postgres/schema/013_*.sql` does **not** exist
  after this packet; the exact SQL is carried inline in §4 for a future packet.
- **No migration is applied** and **the live `seatloom` database is never touched.**
  Everything here is derived from the committed text of `012` and the merged Rust,
  read on this branch — no `psql` was run against any database to produce it.
- The three vocabulary/column/trigger changes below **extend the value-domain
  contract** (`VALUE-DOMAIN-2026-10-08-governed-dispatch-v1.md`). Contract rule 6
  already binds every widening to be "additive and idempotent … and ships with a
  verification query"; the event-type additions additionally need **Lyra's explicit
  sign-off**, exactly as the original three dispatch event types did. This packet
  proposes; it does not decide the vocabulary.

---

## 1. Current state (from `012`) and the three gaps

`012` shipped `dispatches.status` as a 4-value domain, lowercase snake_case to match
every existing status column:

```
ready | blocked | in_flight | done
```

enforced by `dispatches_status_vocab`, with the load-bearing partial unique index
`uq_dispatches_one_in_flight_per_seat (target_seat_id) WHERE status='in_flight'`, and
`dispatch_dependencies (dispatch_id, dep_kind, dep_value)` keyed
`(dispatch_id, dep_kind, dep_value)` with **no FK** on `dep_value` (readiness is an
existence query, by design).

The merged Rust (`crates/seatloom-mcp-emit/src/dispatch.rs`) turns that into three
tools. Reading it in full surfaces three concrete gaps the dispatch names:

**Gap (a) — no failure / cancellation terminal states.** `complete_dispatch`
(`dispatch.rs::complete`) transitions `in_flight → done` and there is no other
outcome. A seat that *cannot* finish a dispatch has no way to say so: it either lies
(`done`) or abandons the row `in_flight` forever, which pins the seat's one in_flight
slot (rule 1) and silently wedges its inbox. There is no `failed` (attempted, did not
succeed) and no `cancelled` (withdrawn by the issuer before completion).

**Gap (b) — no retry semantics.** A transient failure (a flaky dependency, a seat
crash mid-task) has no recovery path. Either the whole dispatch is lost, or a human
re-issues a brand-new dispatch by hand and the retry lineage is invisible. There are
no `retry_count` / `max_retries` columns, so a retry budget cannot be expressed.

**Gap (c) — `blocked → ready` is neither persisted nor notified.** Readiness is
authoritatively computed *live* on every pull: `load_views` → `live_status`
(`dispatch.rs`) re-runs the existence query for `ready`/`blocked` rows and returns
the freshest answer, so a pull *does* see a now-unblocked dispatch as `ready`. But:
1. the **stored** `status` column stays `'blocked'` until some pull happens to touch
   that row, so the projection — and the Scenario-3 dispatch-inbox view that orders by
   stored status — shows a stale `blocked`; and
2. `pg_notify('dispatch_ready', seat)` fires **only inside `enqueue`**, and only for a
   dispatch that is *already* ready at insert. When a blocked dispatch's dependency
   referent lands *later* (an event/artifact/dispatch row is inserted), **nothing
   wakes the seat** to come pull the now-ready work. The seat stays idle until its
   next unprompted drain, and the waker — the whole point of the event-driven pickup
   (`cc92714`) — is never told.

These are the three gaps §2 designs against.

---

## 2. Design decisions

### 2.1 (a) Widen the status domain 4 → 6: add `failed`, `cancelled`

Add two **terminal** values, following `012`'s additive / catalog-guarded DROP-and-
re-ADD pattern (guarded on `pg_get_constraintdef … LIKE '%failed%'`, so a re-apply
finds the value present and does nothing — a true no-op, the standard `010`/`012`
met). The exact SQL is in §4.1.

**Semantics.** `done`, `failed`, `cancelled` are the three terminal states; `ready`,
`blocked`, `in_flight` are the non-terminal ones. Terminal is **immutable**: once a
row is `done`/`failed`/`cancelled` it never leaves that state. This matters because
the dispatches row is a projection of the append-only ledger — mutating a terminal
row back to a non-terminal state would be a silent rewrite of a recorded fact, the
exact anti-pattern contract rule 2 forbids. The retry path (§2.2) re-opens from
`in_flight`, **never** from `failed`, so it never violates this.

**The one-in-flight index is unaffected.** It is `WHERE status='in_flight'`; both new
values sit outside that predicate, so a seat may hold one `in_flight` plus any number
of `done`/`failed`/`cancelled` rows, exactly as it already may hold many `done`. No
index change is needed for rule 1. (`cancelled` of an `in_flight` dispatch *releases*
the slot — see §2.4 — which is the desired "abandon the work" behaviour.)

**Rust flag (Nimbus lane, reported not touched).** `scheduler.rs::DispatchStatus`
gains `Failed` / `Cancelled` variants (`as_str` / `parse`, exhaustiveness test kept,
not loosened — the same hand-written-boundary discipline as the runtime-triple work).
Critically, `dispatch.rs::live_status` currently returns `Done | InFlight` "as stored"
and recomputes only `Ready | Blocked`; it **must** add `Failed | Cancelled` to the
"return as stored" arm, or a terminal failed/cancelled row whose deps are all present
would be recomputed back to `ready`. That is a one-line correctness change in Nimbus's
file, flagged here per the "report rather than work around" rule.

### 2.2 (b) `complete_dispatch` failure path + `retry_count` / `max_retries`

**Two new columns (§4.2):**

```
retry_count INTEGER NOT NULL DEFAULT 0   -- failed attempts already re-opened
max_retries INTEGER NOT NULL DEFAULT 0   -- retry budget; 0 = first failure is terminal
```

`NOT NULL DEFAULT 0`, not nullable — following the same reasoning `012` §2.3 used for
`payload JSONB NOT NULL`: `retry_count = 0` ("no attempt has failed yet") and
`max_retries = 0` ("no retry budget") are both *real recorded states*, not "not
recorded", so `NULL` would be a lie. `max_retries` is a **hard ceiling**, never
`NULL`-means-unlimited: an unbounded retry budget on a permanently-failing task would
loop forever, which is the stall the contract forbids. An issuer that wants "a lot"
sets a large integer. Existing rows backfill to `0/0` implicitly via the default
(additive; no seeded dispatch row exists to break, per the `012` delivery).

**`complete_dispatch` gains an `outcome` argument** (Nimbus lane — the tool signature
and event append): `"done"` (the **default**, fully backward compatible — the current
three-tool loop and Aegis's live proof are unchanged) or `"failed"` with an optional
`reason`. On `outcome="failed"`:

- if `retry_count < max_retries` → transition `in_flight → ready`,
  `retry_count := retry_count + 1`, append **`SeatDispatchRetried`** (payload carries
  `reason`, `retry_count`, `max_retries`). The dispatch is immediately re-pullable —
  the transition *releases* the one-in-flight slot, so the same or another seat can
  pull it again. The failed attempt is durably recorded **in the ledger event** even
  though the row is `ready`, so the retry lineage is auditable.
- else (budget exhausted, or `max_retries = 0`) → transition `in_flight → failed`
  (terminal), append **`SeatDispatchFailed`** (payload `reason`, `retry_count`,
  `max_retries`, `retries_exhausted = true`).

**Why two event types, not one.** Each lifecycle event maps to *exactly one*
`(from → to)` transition — the property that makes the projection replayable, and the
whole reason the contract insisted on all three of Issued/Delivered/Completed
("omitting any one makes the projection unreplayable"). A single `SeatDispatchFailed`
that *sometimes* means "re-opened for retry" and *sometimes* "terminal failure" would
make the ledger ambiguous to replay. So `SeatDispatchRetried` (→ ready) and
`SeatDispatchFailed` (→ failed) are distinct, each with one transition. This keeps the
event vocabulary a clean transition table (see §2.5).

**Rejected alternative — `failed` always terminal, retry = re-enqueue.** I considered
making `failed` unconditionally terminal and modelling a retry as a *fresh*
`enqueue_dispatch` (new id, optionally with a `dispatch` dependency on the failed one).
Rejected: it forks one logical task across N rows and makes the `retry_count` /
`max_retries` *columns* the dispatch explicitly asks for inexpressible — a re-enqueue
carries no single-row retry budget. In-place retry keeps one row = one logical task,
with the budget visible on the row. Stated so the trade-off is on the record.

### 2.3 (c) `blocked → ready`: persist the flip and wake the seat — via triggers

The recompute already exists as *live* logic; what is missing is (1) persisting the
flip into the stored column and (2) notifying the seat. I propose an
**`AFTER INSERT` trigger on each referent table** (`canonical_events`, `artifacts`,
`dispatches`) that, for the just-inserted referent, finds every *currently-blocked*
dispatch that names it as a dependency, re-evaluates **all** of that dispatch's deps
by existence query (the same query the app runs), and — only if the set is now
complete — persists `blocked → ready` and `pg_notify('dispatch_ready', target_seat_id)`.
Exact SQL in §4.3.

**Why this composes with what already exists.** The notify reuses the *exact* channel
`enqueue_dispatch` already fires (`pg_notify('dispatch_ready', seat)`), so the
already-built `seatloom-waker` (LISTENs on `dispatch_ready`, nudges a seat iff idle)
wakes the seat with **zero new machinery**. The flip runs *inside the referent's insert
transaction*, so it is atomic with the dependency landing, and the notify is
transactional (delivered on commit, after the row is visible). This is the event-driven
pickup done right — no polling.

**Design points inside the trigger:**

- **`dep_kind` is matched to the firing table.** Referent id spaces collide
  (`canonical_events.id`, `artifacts.id`, `dispatches.id` are all TEXT and can hold
  equal strings), so the events-trigger satisfies only `dep_kind='event'`, etc. The
  function branches on `TG_TABLE_NAME`.
- **A referent landing re-checks the *whole* dep set**, not just the one that landed —
  one new referent may complete the set or may not. Correct under concurrency: each
  insert re-evaluates independently, so the *last* dep to commit is the one that
  flips the row.
- **The flip UPDATE is guarded** (`WHERE id = cand AND status = 'blocked'`), so it
  commutes safely with the app's transitions: if `get_next_dispatch` already moved the
  row `blocked → in_flight`, the trigger matches 0 rows and no-ops; if the trigger
  wins, `get_next`'s `WHERE status IN ('ready','blocked')` still matches. No
  corruption, no double-notify (the trigger only touches *other* blocked dispatches
  that depend on the referent; `enqueue`'s own notify is for the newly-created row).
- **Reverse-lookup index is required.** `dispatch_dependencies`' PK leads with
  `dispatch_id`, so the trigger's `WHERE dep_kind = $ AND dep_value = $` would
  otherwise seq-scan on every referent insert. `013` adds
  `idx_dispatch_dependencies_by_referent (dep_kind, dep_value)` (§4.2). At governed
  dispatch's tiny scale this is belt-and-braces, but it is the correct primitive and
  also serves any future "what is blocked on X?" query.
- **Cost, stated.** The events-trigger fires on *every* `canonical_events` insert (a
  hot table). The work per insert is one indexed reverse-probe that returns ~0 rows
  for the overwhelming majority of events, so the steady-state overhead is negligible
  at SeatLoom's current event rate. If `canonical_events` ever becomes hot enough for
  that probe to matter, the recompute could be narrowed or made asynchronous — noted
  as a scale refinement, not needed now (the same "graphs are small, a full walk is
  fine" justification the scheduler's `detect_cycle` already records).
- **Existence semantics preserved, including the documented subtlety.** A `dispatch`
  dependency is satisfied by the referent *row existing*, not by it completing — so
  "wait for B to finish" is modelled by depending on B's `SeatDispatchCompleted`
  **event**, not on B's dispatch row. The trigger honours this exactly (the
  dispatches-trigger fires on B's insert and satisfies any dep on B's *existence*).
  The `blocked → ready` direction is the common case (referents get created); the
  reverse (`ready → blocked` if a referent is ever *deleted*) stays the app's
  live-read concern and is out of scope — referents are append-only in practice.

**Rejected alternative — a waker-side scan / reaper.** I considered having the
`seatloom-waker` periodically scan for blocked-but-now-satisfiable dispatches and
notify. Rejected: that is polling on a timer, which the architecture explicitly
rejected when it chose the notify+waker design ("NOT polling — old-fashioned + burns
idle tokens", `cc92714`). The trigger pushes the recompute through the *existing*
notify channel instead of reintroducing a poll.

### 2.4 Cancellation: a terminal status, set by the issuer (cross-lane)

`cancelled` is terminal like `failed`, but its **actor is the issuer**, not the seat:
a seat either does the work (`done`) or fails it (`failed`); *withdrawal* is the
issuer's act. `complete_dispatch` resolves the calling identity from
`SEATLOOM_ACTOR_REF` = the **seat**, so cancellation cannot live cleanly inside
`complete_dispatch`.

Recommendation (Nimbus lane, flagged): a separate **`cancel_dispatch`** tool, called
from the *issuer's* server, guarded so only the issuer may cancel
(`WHERE issued_by = <resolved actor_ref>`), transitioning any non-terminal
(`ready | blocked | in_flight`) → `cancelled` and appending **`SeatDispatchCancelled`**
(§2.5). Cancelling an `in_flight` dispatch releases the one-in-flight slot — the
seat's in-progress work is abandoned, which is the intended semantics. The **schema
half** (the `cancelled` value, its event type, terminal immutability) is fully
specified in §4; the tool is Nimbus's to build and is **not** a blocker for the
schema. If scope must shrink, `cancelled` + `cancel_dispatch` + `SeatDispatchCancelled`
are the cleanly-severable piece — `failed` + retry + the blocked→ready trigger stand
on their own.

### 2.5 Event vocabulary: 3 new types, each one transition (needs Lyra sign-off)

The dispatches row is replayable from the ledger only if **every** transition is an
event (contract rule 2). The current three cover `→ready/blocked`, `→in_flight`,
`→done`. The new transitions need matching events:

| Event type | Transition | Terminal? | Actor |
|---|---|---|---|
| `SeatDispatchIssued` (exists) | — → `ready` \| `blocked` | no | issuer |
| `SeatDispatchDelivered` (exists) | `ready`/`blocked` → `in_flight` | no | seat |
| `SeatDispatchCompleted` (exists) | `in_flight` → `done` | **yes** | seat |
| **`SeatDispatchFailed`** (new) | `in_flight` → `failed` | **yes** | seat |
| **`SeatDispatchRetried`** (new) | `in_flight` → `ready` | no | seat |
| **`SeatDispatchCancelled`** (new) | `ready`/`blocked`/`in_flight` → `cancelled` | **yes** | issuer |

Three new `canonical_events.event_type` values, widening the vocabulary 15 → 18 via
`012`'s `pg_get_constraintdef`-guarded DROP/ADD (§4.1). This is a **minimal** widening
(every transition gets exactly one event, none reused across transitions — the same
"minimal widening, reused value" discipline the contract applied) but it is still a
**vocabulary change to the Lyra-owned contract and needs explicit Lyra sign-off**
before `013` is written. Escalation adds no new value (a stall/cycle stays
`SupervisorMessage`), consistent with the contract. Reported here, not assumed.

---

## 3. Cross-lane flags (report, do not work around)

1. **Lyra — value-domain sign-off (§2.5).** Three new event types
   (`SeatDispatchFailed` / `SeatDispatchRetried` / `SeatDispatchCancelled`) extend the
   contract's event vocabulary. Required for replayability; needs Lyra's ruling. If
   Lyra wants fewer, `cancelled` is the severable one (§2.4).
2. **Nimbus — `DispatchStatus` enum + `live_status` (§2.1).** Add `Failed`/`Cancelled`
   variants; add them to `live_status`'s "return as stored" arm so a terminal row is
   never recomputed to `ready`. Hand-written `FromStr`/`Display` kept; exhaustiveness
   test pinned, not loosened.
3. **Nimbus — `complete_dispatch` `outcome` arg + `cancel_dispatch` tool (§2.2, §2.4).**
   Failure path, retry columns write, and the issuer-side cancel tool are Rust/MCP;
   the event append routes through the existing `append_event_in_tx` in the same
   transaction (rule 2), no second write path.
4. **Onyx (future packet) — migration `013` + `apply-db.sh`.** When Lyra signs off, a
   real Onyx packet creates `infra/postgres/schema/013_dispatch_states.sql` from §4,
   registers it in `scripts/apply-db.sh` **POST-SEED with a reason** (it widens CHECKs
   `010`/`012` created and adds triggers — same classification logic as `011`/`012`),
   and verifies on a scratch DB. That packet is out of scope here.

---

## 4. Proposed migration `013` — EXACT SQL (not created, not applied)

> **PROPOSED ONLY.** This is the SQL a future `013` packet would ship once Lyra signs
> off on §2.5. This packet does **not** write this file, does **not** register it in
> `apply-db.sh`, and does **not** apply it to any database. It is 010/012 house style:
> additive, idempotent (a re-apply performs no catalog write), one transaction.

### 4.1 Vocabulary widenings (catalog-guarded DROP/ADD, `012` pattern)

```sql
-- 013_dispatch_states.sql
-- =============================================================================
-- Governed-dispatch terminal / failure states (G19 follow-on to 012).
-- Authority: docs/coordination/tasks/onyx/ONYX-2026-10-09-dispatch-states-design.md
-- (Lyra value-domain sign-off REQUIRED for the 3 event-type widenings — see §2.5).
-- Slot 013: next after 012. POST-SEED (widens CHECKs 010/012 created; adds triggers).
-- Additive and idempotent, 010/012 house style: every vocab change is a
-- pg_get_constraintdef-guarded DO block, so a second apply performs NO catalog write.
-- =============================================================================

BEGIN;

-- Widen dispatches_status_vocab: 4 -> 6 (+failed, +cancelled). Guarded on the new
-- value being absent, so a re-apply finds it present and does nothing.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'dispatches_status_vocab'
       AND pg_get_constraintdef(oid) LIKE '%failed%'
  ) THEN
    ALTER TABLE dispatches
      DROP CONSTRAINT IF EXISTS dispatches_status_vocab;
    ALTER TABLE dispatches
      ADD CONSTRAINT dispatches_status_vocab
      CHECK (status IN ('ready', 'blocked', 'in_flight', 'done', 'failed', 'cancelled'));
  END IF;
END $$;

-- Widen canonical_events_event_type_vocab: 15 -> 18 (+3 dispatch lifecycle). 012
-- widened it 12 -> 15 with the same guard; this adds the failure/retry/cancel
-- transitions so the dispatches projection stays replayable (contract rule 2).
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint
     WHERE conname = 'canonical_events_event_type_vocab'
       AND pg_get_constraintdef(oid) LIKE '%SeatDispatchFailed%'
  ) THEN
    ALTER TABLE canonical_events
      DROP CONSTRAINT IF EXISTS canonical_events_event_type_vocab;
    ALTER TABLE canonical_events
      ADD CONSTRAINT canonical_events_event_type_vocab
      CHECK (event_type IN (
        'SessionStarted', 'WorkItemCreated', 'WorkItemStatusChanged',
        'ArtifactCreated', 'HandoffSent', 'HandoffAccepted', 'HandoffCompleted',
        'ReviewVerdictIssued', 'SeatDelegationIssued', 'SeatDelegationClosed',
        'SupervisorMessage', 'SeatResponse',
        'SeatDispatchIssued', 'SeatDispatchDelivered', 'SeatDispatchCompleted',
        -- 013 (G19): terminal / retry / cancel transitions.
        'SeatDispatchFailed',      -- in_flight -> failed (terminal)
        'SeatDispatchRetried',     -- in_flight -> ready  (retry re-open)
        'SeatDispatchCancelled'    -- ready|blocked|in_flight -> cancelled (terminal)
      ));
  END IF;
END $$;
```

### 4.2 Retry columns + reverse-lookup index

```sql
-- Retry budget, per dispatch. NOT NULL DEFAULT 0 (a real recorded state, not
-- "not recorded"); additive — existing rows backfill to 0 implicitly. max_retries
-- is a hard ceiling, never NULL-means-unlimited (an unbounded retry loops forever).
ALTER TABLE dispatches ADD COLUMN IF NOT EXISTS retry_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE dispatches ADD COLUMN IF NOT EXISTS max_retries INTEGER NOT NULL DEFAULT 0;

COMMENT ON COLUMN dispatches.retry_count IS
  'Failed attempts already re-opened for retry (013). 0 = no attempt has failed yet. Incremented by the in_flight -> ready SeatDispatchRetried transition; never mutated on a terminal row.';
COMMENT ON COLUMN dispatches.max_retries IS
  'Retry budget set by the issuer (013). 0 = the first failure is terminal (SeatDispatchFailed). A hard ceiling, never NULL: an unbounded retry on a permanently-failing task would loop forever, the stall the contract forbids.';

-- Reverse lookup for the blocked->ready trigger (and any future "what depends on
-- X?"): the PK leads with dispatch_id, so dep_kind+dep_value would otherwise
-- seq-scan on every referent insert.
CREATE INDEX IF NOT EXISTS idx_dispatch_dependencies_by_referent
    ON dispatch_dependencies (dep_kind, dep_value);
```

### 4.3 The `blocked -> ready` recompute function + three triggers

```sql
-- One shared function; three thin triggers (one per referent table) call it.
-- FOR EACH ROW: for the just-inserted referent, find every CURRENTLY-blocked
-- dispatch that names it, re-check ALL that dispatch's deps by existence (same
-- query the app runs on pull), and — only if the set is now complete — persist
-- blocked -> ready (guarded) and pg_notify('dispatch_ready', seat) so the waker
-- wakes the seat. Idempotent: guarded CREATE (no DROP), so a re-apply is a no-op;
-- a future body change ships as its own DROP+CREATE migration.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace
     WHERE p.proname = 'fn_dispatch_recompute_ready' AND n.nspname = current_schema()
  ) THEN
    CREATE FUNCTION fn_dispatch_recompute_ready() RETURNS trigger
    LANGUAGE plpgsql AS $fn$
    DECLARE
      ref_kind TEXT;
      cand      RECORD;
      total     INT;
      satisfied INT;
    BEGIN
      -- Which dep_kind does the table this trigger fired on satisfy? Referent id
      -- spaces collide across the three tables, so match kind to firing table.
      IF TG_TABLE_NAME = 'canonical_events' THEN ref_kind := 'event';
      ELSIF TG_TABLE_NAME = 'artifacts'  THEN ref_kind := 'artifact';
      ELSIF TG_TABLE_NAME = 'dispatches' THEN ref_kind := 'dispatch';
      ELSE RETURN NEW; END IF;  -- defensive: unknown table, no-op

      FOR cand IN
        SELECT d.id, d.target_seat_id
          FROM dispatches d
          JOIN dispatch_dependencies dd
            ON dd.dispatch_id = d.id
           AND dd.dep_kind     = ref_kind
           AND dd.dep_value    = NEW.id
         WHERE d.status = 'blocked'
      LOOP
        -- Re-evaluate ALL of this dispatch's deps by existence. One referent
        -- landing may complete the set, or may not.
        SELECT count(*), count(*) FILTER (WHERE present)
          INTO total, satisfied
          FROM (
            CASE dd2.dep_kind
              WHEN 'event'    THEN EXISTS(SELECT 1 FROM canonical_events e WHERE e.id = dd2.dep_value)
              WHEN 'artifact' THEN EXISTS(SELECT 1 FROM artifacts a       WHERE a.id = dd2.dep_value)
              WHEN 'dispatch' THEN EXISTS(SELECT 1 FROM dispatches x      WHERE x.id = dd2.dep_value)
            END AS present
          ) deps
          FROM dispatch_dependencies dd2
         WHERE dd2.dispatch_id = cand.id;

        IF total > 0 AND satisfied = total THEN
          -- Persist the flip. GUARDED: only blocked -> ready; a row that raced to
          -- in_flight/done/failed/cancelled/ready is left untouched (0 rows).
          UPDATE dispatches SET status = 'ready'
           WHERE id = cand.id AND status = 'blocked';
          -- Wake the seat through the SAME channel enqueue_dispatch uses.
          PERFORM pg_notify('dispatch_ready', cand.target_seat_id);
        END IF;
      END LOOP;

      RETURN NEW;
    END;
    $fn$;
  END IF;
END $$;

-- Three triggers, each guarded so a re-apply creates nothing.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'trg_dispatch_ready_on_event'
                 AND tgrelid = 'canonical_events'::regclass) THEN
    CREATE TRIGGER trg_dispatch_ready_on_event
      AFTER INSERT ON canonical_events
      FOR EACH ROW EXECUTE FUNCTION fn_dispatch_recompute_ready();
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'trg_dispatch_ready_on_artifact'
                 AND tgrelid = 'artifacts'::regclass) THEN
    CREATE TRIGGER trg_dispatch_ready_on_artifact
      AFTER INSERT ON artifacts
      FOR EACH ROW EXECUTE FUNCTION fn_dispatch_recompute_ready();
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'trg_dispatch_ready_on_dispatch'
                 AND tgrelid = 'dispatches'::regclass) THEN
    CREATE TRIGGER trg_dispatch_ready_on_dispatch
      AFTER INSERT ON dispatches
      FOR EACH ROW EXECUTE FUNCTION fn_dispatch_recompute_ready();
  END IF;
END $$;

COMMENT ON FUNCTION fn_dispatch_recompute_ready() IS
  'Governed-dispatch (G19, 013): on insert of an event/artifact/dispatch referent, flip any blocked dispatch whose dep set is now fully satisfied to ready (persisted) and pg_notify its seat on the dispatch_ready channel the waker already LISTENs on. Readiness stays authoritatively an existence query; this is the projection-updater + notifier for the blocked->ready direction.';

COMMIT;
```

> Note on the inner `SELECT … FROM ( CASE … END AS present ) deps FROM
> dispatch_dependencies dd2`: the subquery form is illustrative; the shipped form
> would compute the existence booleans directly over `dd2` (e.g. a `SELECT …
> FROM dispatch_dependencies dd2, LATERAL (SELECT CASE … END) p`). The intent — count
> total deps and satisfied deps for the candidate dispatch — is the load-bearing part
> and is what §5 verifies.

### 4.4 Verification query (for the future packet's file header)

```sql
-- Status domain now 6 values, event vocabulary now 18:
SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'dispatches_status_vocab';
SELECT pg_get_constraintdef(oid) FROM pg_constraint WHERE conname = 'canonical_events_event_type_vocab';
-- Retry columns exist, NOT NULL, default 0:
SELECT column_name, data_type, is_nullable, column_default
  FROM information_schema.columns WHERE table_name='dispatches'
   AND column_name IN ('retry_count','max_retries');
-- Reverse index + function + three triggers exist:
SELECT indexdef FROM pg_indexes WHERE indexname='idx_dispatch_dependencies_by_referent';
SELECT proname FROM pg_proc WHERE proname='fn_dispatch_recompute_ready';
SELECT tgname, tgrelid::regclass FROM pg_trigger
 WHERE tgname IN ('trg_dispatch_ready_on_event','trg_dispatch_ready_on_artifact','trg_dispatch_ready_on_dispatch');
```

---

## 5. Verification plan for the future `013` packet (scratch DB only)

Not run here (no migration created, no DB touched). The packet that ships `013` runs,
on a **scratch** database via `scripts/apply-db.sh` (live `seatloom` never touched):

1. **Idempotency** — apply `013` twice; snapshot the two widened CHECKs, the two new
   columns, the new index, the function, and the three triggers is byte-identical
   before/after the re-apply (zero catalog write — the `012` standard).
2. **Domains** — `status` CHECK rejects `queued`; accepts `failed`/`cancelled`.
   `event_type` CHECK accepts the three new values, rejects an unknown one.
3. **Retry columns** — existing dispatch rows read `retry_count=0, max_retries=0`;
   a negative or `NULL` insert is rejected (`NOT NULL`).
4. **`blocked → ready` via trigger** — insert a `blocked` dispatch depending on an
   event id that does not exist; insert that event; confirm the dispatch's **stored**
   status flips to `ready` (persisted, not just live-computed) and that a LISTENing
   session on `dispatch_ready` receives the seat id. Repeat for `artifact` and
   `dispatch` referent kinds. Confirm a dep set that is *still* incomplete does **not**
   flip.
5. **Trigger does not touch terminal/active rows** — an `in_flight`/`done`/`failed`
   dispatch that happens to name the referent is left unchanged (guarded `WHERE
   status='blocked'`).
6. **One-in-flight unchanged** — a second `in_flight` for the same seat is still
   rejected by `uq_dispatches_one_in_flight_per_seat` (new terminal values sit outside
   its predicate).

---

## 6. Done-definition for THIS design dispatch

1. ✅ Design doc at `docs/coordination/tasks/onyx/ONYX-2026-10-09-dispatch-states-design.md`.
2. ✅ Gap (a): `failed`/`cancelled` proposed with the `012`-style additive/idempotent
   CHECK widening; one-in-flight index shown unaffected; Rust `live_status` flag raised.
3. ✅ Gap (b): `complete_dispatch` failure path (backward-compatible `outcome` arg)
   and `retry_count`/`max_retries` columns proposed, with the rejected re-enqueue
   alternative stated.
4. ✅ Gap (c): `blocked → ready` persist + notify designed via guarded triggers on the
   three referent tables, reusing the existing `dispatch_ready` waker channel; the
   polling alternative rejected with its recorded reasoning.
5. ✅ Exact `013` SQL attached in §4, clearly labelled proposed-only.
6. ✅ Vocabulary sign-off + all cross-lane flags reported, not worked around.
7. ✅ **No migration created, none applied, no live database touched, zero
   Rust/`.tsx`/A1 files edited** (this packet is `.md` only; `git status` shows only
   this doc + the coordination writeback).

---

## 7. Blockers / follow-ups

- **Lyra sign-off on the 3 new event types (§2.5)** is the gate for a real `013`
  packet. Until then this stays a proposal. If scope must shrink, sever `cancelled` +
  `cancel_dispatch` (§2.4).
- **Onyx follow-up** (post-sign-off): create `013_dispatch_states.sql` from §4,
  register it POST-SEED in `scripts/apply-db.sh` with a reason, verify per §5 on a
  scratch DB, deliver under `packet/onyx/…`.
- **Nimbus follow-up**: `DispatchStatus` variants + `live_status` arm, the
  `complete_dispatch` `outcome` arg, and the issuer-side `cancel_dispatch` tool.
- **Open question for Lyra (not blocking the schema):** should a retry re-open to the
  *same* seat (sticky) or to any eligible seat (current `ready` behaviour)? Sticky
  retry would need a "preferred seat" notion the contract does not have; I left it as
  plain `ready` (re-pullable by anyone) to avoid inventing contract surface. Flagged,
  not decided.

---

*Proposed by Onyx · 2026-10-09 · governed dispatch `disp-onyx-states-design` · design
only — migration 013 NOT created, NOT applied, live DB untouched · awaiting Lyra
value-domain sign-off on the 3 new event types (§2.5)*
