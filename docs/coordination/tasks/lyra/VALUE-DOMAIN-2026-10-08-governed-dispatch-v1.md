# Value-domain contract — SeatLoom governed dispatch (G19 / P0 step 6)

**Owner:** Lyra (PO). **Status:** ACTIVE. **Date:** 2026-10-08.
**Applies to:** `ONYX-2026-10-08-p0-step6-dispatch-schema-v1.md` and
`NIMBUS-2026-10-08-p0-step6-dispatch-scheduler-v1.md`.

This is the single contract both step-6 packets carry **verbatim**, the same way the
runtime-triple packets were joined by one Lyra-owned value-domain contract. Neither seat
re-derives it; a deviation is reported, not silently reconciled.

```
GOVERNED DISPATCH (G19, P0 step 6) — per-seat inbox + ready-aware scheduler

OBJECT: dispatches — the per-seat inbox. One row = one thing a seat must do.
  id              TEXT PRIMARY KEY
  target_seat_id  TEXT NOT NULL        -- the seat that must act
  issued_by       TEXT NOT NULL        -- actor_ref of the issuer (supervisor / PO)
  project_id      TEXT NOT NULL REFERENCES projects(id)
  title           TEXT NOT NULL        -- one-line routing summary
  payload         JSONB NOT NULL       -- the packet path + instructions
  status          TEXT NOT NULL        -- ready | blocked | in_flight | done
  priority        INT  NOT NULL        -- higher pulls first; ties broken by issued_at
  issued_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
  started_at      TIMESTAMPTZ
  completed_at    TIMESTAMPTZ

DEPENDENCY: dispatch_dependencies — the graph AS DATA, machine-checkable.
  dispatch_id  TEXT NOT NULL REFERENCES dispatches(id)
  dep_kind     TEXT NOT NULL        -- event | artifact | dispatch
  dep_value    TEXT NOT NULL        -- canonical_events.id | artifacts.id | dispatches.id
  A dispatch is READY iff every dependency row's referent EXISTS.
  Cross-seat dependencies are expressible by construction, which is what makes a
  cycle detectable rather than merely possible.

STATUS DOMAIN (4 values, lowercase snake_case to match existing status columns):
  ready      -- all dependencies satisfied, pullable now
  blocked    -- at least one dependency not yet present; NOT an error, NOT a stall
  in_flight  -- exactly one seat holds it; the seat is acting on it
  done       -- the seat completed it

EVENT TYPE VOCABULARY — 3 NEW VALUES (Lyra sign-off, this packet):
  SeatDispatchIssued       -- a dispatch entered the inbox
  SeatDispatchDelivered    -- a seat pulled it (ready -> in_flight)
  SeatDispatchCompleted    -- the seat marked it done (in_flight -> done)
  Rationale: the dispatches row is a PROJECTION. It is rebuildable from the ledger
  only if all three transitions are events. Omitting any one makes the projection
  unreplayable, which is the exact defect step 2 (A1) removed.
  Escalation does NOT add a value: a cycle/stall escalation is emitted as the
  existing SupervisorMessage, actor_ref = the scheduler, refs = [subject=the
  offending dispatch, actor=the scheduler]. Minimal widening, reused value.

REF TYPE VOCABULARY — 1 NEW VALUE (Lyra sign-off, this packet):
  dispatch
  Rationale: `dispatches` is a table with its own object, exactly the reasoning 010
  used to admit 'delegation' (which the original contract omitted). Without it an
  event cannot name the dispatch it is about.

RULES (binding on both packets):
  1. ONE in_flight dispatch per seat, enforced IN THE DATABASE by a partial unique
     index on (target_seat_id) WHERE status='in_flight' — not only in scheduler code.
     One-in-flight is what kills context pollution.
  2. Status transitions are EVENTS. The dispatches row is the projection;
     canonical_events is the ledger. No silent UPDATE that is not backed by an
     appended event in the SAME transaction.
  3. Delivery is by MCP PULL ONLY. A seat calls get_next_dispatch() when it is idle
     (end of turn). There is NO mid-turn tmux injection, ever.
  4. Blocked items NEVER head-of-line-block. A pull skips them and returns the next
     READY item for that seat, highest priority first.
  5. A cross-seat dependency CYCLE, or an all-blocked STALL, is DETECTED and
     ESCALATED (emit SupervisorMessage + park in docs/coordination/memory/YYYY-MM-DD.md),
     never hung.
  6. Every vocabulary widening is additive and idempotent, using 010's
     catalog-guarded DO-block pattern, and ships with a verification query.
  7. schema_version is set EXPLICITLY (>= 1); role is stated EXPLICITLY on every ref;
     EXACTLY ONE subject per event; turn_seq stays NULL (not recorded) in P0.
  8. Auto-resolution, preemption, and priority tuning are explicitly OUT of the first
     slice. Decide them as reached.
```
