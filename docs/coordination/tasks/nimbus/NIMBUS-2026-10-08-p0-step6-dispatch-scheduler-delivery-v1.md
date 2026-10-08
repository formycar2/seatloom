# Delivery: P0 step 6 (Nimbus half) — ready-aware scheduler + MCP pull delivery

[Nimbus -> Lyra / Flux] Delivery for `NIMBUS-2026-10-08-p0-step6-dispatch-scheduler-v1`.

> **Status: PARTIAL — scheduler core delivered and green; DB/MCP wiring BLOCKED on
> two sequenced prerequisites outside this packet's write set. This is a STOP-and-report
> per packet §4, not a completed packet.** Read §7 first.

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation-delivery (partial / blocked) |
| id | NIMBUS-2026-10-08-p0-step6-dispatch-scheduler-delivery-v1 |
| author | nimbus |
| date | 2026-10-08 |
| version | v1 |
| responds to | `docs/coordination/tasks/nimbus/NIMBUS-2026-10-08-p0-step6-dispatch-scheduler-v1.md` |
| shared contract | `docs/coordination/tasks/lyra/VALUE-DOMAIN-2026-10-08-governed-dispatch-v1.md` (verbatim) |
| parallel packet | `docs/coordination/tasks/onyx/ONYX-2026-10-08-p0-step6-dispatch-schema-v1.md` |
| base branch | `track/infra-foundation` |
| base commit | `b200cd3b894d792292188c6de9c7ca3140006ccf` |
| delivery branch | `packet/nimbus/NIMBUS-2026-10-08-p0-step6-dispatch-scheduler-v1` |
| delivery commit (core) | `d7aab53ea7a15977005120bd11ef599e7bc1dd2d` |
| worktree | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/nimbus` |
| scratch database | (none run yet — proof blocked, see §6/§7) |
| status | **PARTIAL / BLOCKED** — awaiting enum sequencing + landed `012` |

---

## 1. What is delivered and green now

`crates/seatloom-mcp-emit/src/scheduler.rs` — the **pure, I/O-free scheduler core**:
the algorithmic heart of governed dispatch (G19), built verbatim against the shared
value-domain contract, with 13 unit tests. It is deliberately **database- and
vocabulary-independent** so it is provable *before* its two blocked prerequisites
(§7) land. Added as a module in `main.rs`; no other file changed.

| item | contract rule | what it does |
|---|---|---|
| `DispatchStatus` (ready/blocked/in_flight/done) | status domain | explicit `parse`, no silent fallback |
| `DepKind` (event/artifact/dispatch) | dependency graph | explicit `parse` |
| `select_next(seat, dispatches)` | rules 1 + 4, pull ordering | existing in_flight wins; else highest-priority **ready**, ties by `issued_at` then `id`; else `Nothing{stall}` |
| `classify_stall(seat, dispatches)` | rule 5 | blocked≥1 ∧ ready=0 ∧ in_flight=0 (idle with unreachable work) |
| `detect_cycle(dispatches)` | rule 5 / §1.3 | 3-colour iterative DFS over `dispatch`→`dispatch` edges only, O(V+E), returns the offending ring |

### Design calls the core bakes in (all from the contract)

- **Skip-blocked is by filtering to `ready`, so a blocked item can never
  head-of-line-block** (rule 4). The test `pull_skips_blocked_and_returns_ready`
  asserts the blocked item was *not* chosen, not merely that a ready one was
  (packet §2.2).
- **One-in-flight: the scheduler returns the existing in_flight before any ready
  item** (rule 1). The database partial unique index
  (`uq_dispatches_one_in_flight_per_seat`, Onyx's `012`) is the actual
  enforcement; this is the scheduler half.
- **Cycle detection follows only `dep_kind='dispatch'` edges** — event/artifact
  deps cannot form a dispatch cycle, and a `dispatch` dep on an id the scheduler
  cannot see is an *unsatisfied/blocked* dependency (readiness), not a cycle.
- **Stall vs normal:** "blocked, and something else is ready" is normal;
  "blocked, and nothing is ready/in_flight" is a stall. Both tested.
- **Cost, stated per §1.3:** a full graph walk per pull. The graphs are small (a
  handful of hops); an incremental check is a later refinement, not needed to
  make the loop safe. Accepted explicitly.

### Verification — Layer A (verbatim, on the core commit `d7aab53`)

```
cargo check --workspace            -> Finished (ok)
cargo test --workspace             -> seatloom-core 104; seatloom-mcp-emit 32 (19 step-4 + 13 scheduler);
                                      postgres_seed_consistency 3; others 0; all ok
cargo clippy --workspace --all-targets -- -D warnings  -> Finished (no warnings)
cargo fmt --all --check            -> clean
```

The core is `#[allow(dead_code)]` with an explicit rationale comment, because its
consumer (the DB/MCP layer) is blocked (§7); the allow is a one-line deletion the
moment the layer wires it. The alternative — shipping a non-compiling tree or
forking the vocabulary into string literals — would violate either Layer A or the
single-source-of-truth boundary, so it was rejected.

## 2. What is designed and ready to wire, but NOT committed (blocked)

The three MCP tools and their atomic row+event writes. Not committed because they
cannot compile without the enum values in §7-A, and cannot be proven without the
landed `012` in §7-B. The design below is grounded in Onyx's delivered `012`
(`97cf9bf`, read in full), so wiring is mechanical once unblocked.

### 2.1 `enqueue_dispatch` (issuer: Aegis / Lyra)

- One transaction: `INSERT INTO dispatches (...) VALUES (...)`, then one
  `INSERT INTO dispatch_dependencies (...)` per dependency, then
  `append_event` of **`SeatDispatchIssued`** — **all in the same `tokio_postgres`
  transaction**, via the A1 pattern (not A1's `append_event` directly, which opens
  its *own* transaction; see §4 — this is the one real integration question).
- Event refs: `subject = dispatch:<id>`, `actor = seat:<issued_by>`.
  `schema_version = 1` explicit; `role` explicit; exactly one subject; `turn_seq = None`.
- `status` computed from readiness at enqueue (ready if every dep referent exists,
  else blocked) — see §2.4.

### 2.2 `get_next_dispatch` (seat, when idle)

1. Resolve the calling seat from server env (§3) — never a caller parameter.
2. Load that seat's dispatches + their dependency rows; compute each row's live
   readiness by **existence query** per dep (§2.4); build `DispatchView`s.
3. `scheduler::select_next(seat, &views)`:
   - `AlreadyInFlight{id}` → return that dispatch, no write.
   - `Deliver{id}` → in **one transaction**: `UPDATE dispatches SET status='in_flight',
     started_at=NOW() WHERE id=$1 AND status='ready'`, then `append_event`
     **`SeatDispatchDelivered`**; commit. (The `AND status='ready'` guard + the DB
     partial unique index make a lost race safe: a second concurrent pull fails the
     unique index and rolls back.)
   - `Nothing{stall:true}` → run cycle/stall escalation (§2.5); return "nothing ready".
   - `Nothing{stall:false}` → return "nothing ready" (idle, not an error).

### 2.3 `complete_dispatch` (seat, when done)

- In one transaction: `UPDATE dispatches SET status='done', completed_at=NOW()
  WHERE id=$1 AND target_seat_id=<resolved seat> AND status='in_flight'`, then
  `append_event` **`SeatDispatchCompleted`**; commit. The seat can only complete its
  own in_flight dispatch (identity from env, not parameter).

### 2.4 Readiness by existence query (contract: READY iff every dep referent EXISTS)

Per `dep_kind`, `SELECT 1 FROM <table> WHERE id=$1`:
`event → canonical_events`, `artifact → artifacts`, `dispatch → dispatches`. A
missing referent → that dep unsatisfied → the dispatch is `blocked`. No FK is used
(Onyx's `012` has none on `dep_value` by design); a dangling dep reads as `blocked`,
never an error.

### 2.5 Cycle / stall escalation (rule 5 — never hang)

- `scheduler::detect_cycle` over the project's dispatch graph; `classify_stall` for
  the calling seat. On a cycle **or** a stall: `append_event` a **`SupervisorMessage`**
  with `actor_ref = the scheduler`, `refs = [subject=<offending dispatch>,
  actor=<scheduler>]`, **and** append a line to `docs/coordination/memory/YYYY-MM-DD.md`.
  The pull then returns "nothing ready" rather than blocking — detection and
  escalation are side effects, never a wait.

## 3. Non-forgeable seat identity (packet §1.1)

`get_next_dispatch` and `complete_dispatch` resolve the calling seat from the
**server's own environment** — `SEATLOOM_ACTOR_REF` (already used by `emit_event`
in step 4), normalised to a seat id — **not** from any caller-supplied `seat_id`
parameter. The tools' JSON Schema will expose **no** `seat_id`/`target_seat_id`
input at all, so there is nothing to spoof: a seat physically cannot ask for
another seat's queue, because the identity never crosses the wire.

- **Why it cannot be forged:** the MCP server is launched per seat by the operator
  (`pi mcp add … --env SEATLOOM_ACTOR_REF=seat:<id>`), exactly as step 4's rollout
  already does. The identity is pinned to the process the seat talks to, not to the
  message it sends.
- **What that costs (stated honestly):** identity is only as trustworthy as the
  per-seat server launch. If one seat could invoke another seat's MCP server
  instance, it would inherit that identity — so the operator must give each seat its
  own server with its own `SEATLOOM_ACTOR_REF`, and must not share one server across
  seats. This is the same trust boundary step 4 established; step 6 adds no new
  attack surface, but it does make the boundary load-bearing for queue isolation.
  A stronger model (signed seat tokens) is out of the first slice.

## 4. One integration question for the wiring (not an A1 change — a reuse question)

A1's `SeatloomDb::append_event` opens **its own** transaction and commits it. Step 6
rule 2 requires the dispatches row change **and** its event in the **same**
transaction. So the wiring cannot call `append_event` *and* do the row `UPDATE` in
one transaction through the current public A1 surface — they would be two
transactions.

Two ways to honour rule 2 without weakening A1, to be decided with Lyra before
wiring (I did **not** pick one unilaterally, because one touches A1's file):

- **(a) No A1 change.** Replicate A1's event+refs+projection INSERTs inside the
  scheduler's transaction using the **same SQL constants and validation path**.
  Risk: a second copy of the append logic that could drift from A1 — the exact
  defect A1 exists to prevent. I do **not** recommend this.
- **(b) Minimal A1 addition (recommended, sequence it).** Add a
  `append_event_in_tx(&Transaction, &EventDraft)` to `write_api.rs` that does the
  validated event+refs+projection write against a **caller-supplied** transaction,
  and make the existing `append_event` a thin wrapper that opens a tx and calls it.
  This is additive, keeps one implementation of the append, and lets the scheduler
  compose its row change with the event atomically. It touches A1's accepted file,
  so per §4 of the packet and the step-4 F1 ruling it is a **sequenced A1 change**,
  not something I fold in.

**Flagged for Lyra.** This is the same class as the step-4 F1 ruling ("do not fork
validity from A1"): the right fix is a small, sequenced A1 addition, not a second
write path.

## 5. Rebase onto Onyx's `012` (packet §3)

**Not done — `012` has not landed on `track/infra-foundation`.** Onyx has
*delivered* `012_governed_dispatch.sql` on `packet/onyx/ONYX-2026-10-08-p0-step6-dispatch-schema-v1`
(`97cf9bf`), but it is delivered-but-unmerged, not accepted/landed. The packet says
"rebase onto Onyx's **landed** migration before delivery"; there is nothing landed to
rebase onto. Per COORDINATION_RULES §14, I will not pin a rebase to an unaccepted
commit. Reported here as required rather than rebasing onto an in-flight branch.

I **read** Onyx's `012` in full to ground §2 — table/column/index names
(`dispatches`, `dispatch_dependencies`, `uq_dispatches_one_in_flight_per_seat`,
`idx_dispatches_pull`), the status/dep_kind CHECK values, the no-FK-on-dep_value
decision, and `payload JSONB NOT NULL`. My scheduler core's `DispatchStatus`,
`DepKind`, and ordering match it exactly. No conflict is expected (disjoint write
sets: `crates/seatloom-mcp-emit/**` vs `infra/postgres/schema/**` + `scripts/`).

## 6. Behavioural proof — not run (blocked), plan committed

The §2 proof (packet §2 items 1–13) **cannot run** until (A) the enum values exist
so the DB layer compiles and (B) `012` is applied to a scratch DB. The proof plan is
fixed and will run on scratch DB `seatloom_mcp_step6_nimbus` (migrations 001–012 via
`scripts/apply-db.sh`, never live `seatloom`), driving the real MCP binary over
stdio exactly as the step-4 proof did, covering: enqueue ready+blocked → pull skips
blocked → second pull returns same in_flight → second seat gets its own → complete →
dependency lands → previously-blocked pulled → every transition present in
`canonical_events` with refs/roles → atomicity rollback → cross-seat cycle
detected+escalated → stall detected+escalated → one-in-flight rejected by the index
in raw SQL → spoofed seat param cannot reach another queue (no param exists) →
dispatches row rebuilt from the ledger alone equals the live row.

I can run a **PROVISIONAL** version of this against Onyx's unmerged `012` branch to
de-risk ahead of acceptance, but it is PROVISIONAL per §14 and does not close the
gate; say the word and I will and attach the output to a v2.

## 7. BLOCKERS — STOP and report (packet §4)

**7-A. The three new `EventType` values + the `dispatch` `RefType` value are not in
`crates/seatloom-core/src/db/event_domain.rs`.** That is A1's accepted file. Packet
§4: *"that file is A1's accepted file, so flag it to me and I will sequence the
change; do not edit it unilaterally."* The daily log (2026-10-08) records Lyra
already signed off the vocabulary widenings and explicitly noted Nimbus "must flag it
to me for sequencing rather than edit it unilaterally." **I am flagging it now.**
Needed, matching Onyx's `012` CHECK verbatim:
- `EventType`: `SeatDispatchIssued`, `SeatDispatchDelivered`, `SeatDispatchCompleted`
  (added to the enum, `ALL`, `as_str`, `FromStr`, and the `010`/`012` exhaustiveness
  tests → now 15 values).
- `RefType`: `Dispatch` → `"dispatch"` (→ 10 values).
Once sequenced (either Lyra authorises me to make this exact additive change on a
branch, or it lands separately), the DB/MCP layer in §2 compiles.

**7-B. Onyx's `012` is not landed on `track`.** The proof (§6) and the required
rebase (§5) both wait on it.

**7-C. The atomic-append integration question (§4)** needs a Lyra decision —
recommended option (b), a sequenced additive `append_event_in_tx` in A1.

None of these is worked around. The scheduler core (§1) is the maximum correct work
available without crossing the A1 boundary or pinning to unaccepted commits.

## 8. Done-definition status

| # | item | status |
|---|---|---|
| 1.1 | three tools added to the step-4 crate | **blocked** (§7-A/B) — designed §2, not committed |
| 1.2 | ready-aware pull (skip blocked, priority, one-in-flight) | **DONE** as pure core + tests (§1) |
| 1.3 | cycle/stall detection + escalation | detection **DONE** + tested; escalation side-effects designed §2.5, blocked on §7 |
| identity | non-forgeable seat identity | **designed + justified** §3; realised when §7-A unblocks |
| §2 proof | end-to-end first slice on scratch DB | **blocked** §6/§7 |
| §3 | rebase onto landed `012` | **cannot** — not landed (§5) |
| Layer A | green verbatim | **DONE** on `d7aab53` (§1) |
| scope | no `.sql`, no `.tsx`, no A1 files edited, no live DB | **held** |

## 9. Changed files (exact — `git show --stat d7aab53`)

```
 crates/seatloom-mcp-emit/src/main.rs      |   1 +   (mod scheduler;)
 crates/seatloom-mcp-emit/src/scheduler.rs | 523 +   (pure core + 13 unit tests)
 2 files changed, 524 insertions(+)
```

Zero `.sql`, zero `.tsx`, **no change to `write_api.rs` / `event_domain.rs`**
(confirmed `git diff --name-only`). No live seat touched. No tmux change.

---

*Delivered by Nimbus · 2026-10-08 · P0 step 6 scheduler half, PARTIAL · pure
scheduler core delivered + green at `d7aab53` (base `b200cd3`); DB/MCP wiring,
proof, and rebase all blocked on (A) the sequenced A1 enum change, (B) landed
`012`, (C) a Lyra decision on the atomic-append integration — reported, not worked
around, per packet §4.*
