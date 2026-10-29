# Delivery: migration 020 — the gate/announcement criterion at release time (the four answers)

[Onyx -> Nimbus/Aegis] `disp-onyx-gate-release-check`: the release-time half of Aegis ruling 2026-10-11
is landed — `fn_dispatch_recompute_ready` no longer treats referent EXISTENCE as readiness for a
gate-shaped edge. This note is also the sign-off Nimbus's plan asked for: the four questions, answered
with evidence, in this seat's own slot (`infra/**`), on this seat's own branch. `crates/**` untouched,
Nimbus's branch untouched and unmerged.

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | verification-delivery |
| id | ONYX-2026-10-12-gate-release-check-delivery-v1 |
| status | delivered |
| author | onyx |
| date | 2026-10-12 |
| version | v1 |
| responds to | governed dispatch `disp-5YCwjJF-oIzwb-d1rmrs5` (`disp-onyx-gate-release-check`); verdict event id `ev-verdict-onyx-gate-release-check` |
| packet / contract | Aegis ruling `AEGIS-2026-10-11-gate-vs-announcement-ruling.md` (裁决一/三/四); Nimbus's delivery `NIMBUS-2026-10-12-gate-vs-announcement-delivery-v1.md` @ `packet/nimbus/gate-vs-announcement` @ `1a4b75e`; the criterion's shape is `crates/seatloom-mcp-emit/src/gate.rs` on that branch |
| base branch | `packet/onyx/topic-opened-019` @ `82c919a` (this branch; schema state 001–019) |
| delivery branch | `packet/onyx/topic-opened-019` @ this commit |
| files changed | `infra/postgres/schema/020_gate_criterion.sql` (new), `scripts/apply-db.sh` (+1 POST_SEED entry), this doc |
| rust / tsx touched | none — the Rust half is Nimbus's, already landed @ `1a4b75e` |
| live db touched | none — live READ only (pre-flight census below); all writes on scratch DBs, dropped after |

---

## 0. What the ruling asked this seat to decide

裁决四 left the mechanisation location open but pinned the tension: a gate event id is **pre-declared**
(the gated self-dispatch's premise is "edge first, event later"), so "check the event carries `verified`
when the edge is hung" is impossible — the event does not exist yet. The check can only happen at the
moment `fn_dispatch_recompute_ready` flips `blocked -> ready`. That function is **my slot**; the
criterion's shape is **Nimbus's tool contract**. Both sides had to opine; nobody decides unilaterally.

So: **I opine, and I land.** Four answers, then the migration, then the evidence.

---

## 1. The four answers

### 1.1 Q1 — `btrim` or plain `coalesce`? **Neither, exactly. Trim, over Rust's whitespace set.**

Nimbus is right that plain `coalesce(payload->>'verified','') <> ''` is **not** byte-identical: Rust's
`gate.rs::stated()` reads absent, null, empty **and whitespace-only** as "not stated". A gate that
claims to be verified by `"   "` is verified by nothing.

But `btrim(x)` alone is also not byte-identical, and the direction of the difference matters:

* `btrim(x)` with one argument trims **spaces only**. A `verified` of `"\t"` or `"\u00a0"` survives it.
* The dangerous direction is **SQL looser than Rust**: the database flips `blocked -> ready`, the seat
  pulls, `live_status()` recomputes and says `blocked`, and the seat reports "nothing ready" while the
  row says `ready`. That is the loud silent the ruling exists to prevent. So the SQL's "not stated"
  set must be a **superset** of Rust's, never a subset.

So the shipped test is

```sql
btrim(coalesce(payload ->> 'verified', ''), <Rust's whitespace set>) <> ''
```

where the trim set is **enumerated**, not taken from the locale: U+0009–U+000D, U+0020, U+0085,
U+00A0, U+1680, U+2000–U+200A, U+2028, U+2029, U+202F, U+205F, U+3000 — exactly Rust's
`char::is_whitespace` (the Unicode `White_Space` property). Two reasons it is enumerated rather than
`[[:space:]]` or `btrim`'s default: a locale's `[[:space:]]` is a **subset** (no U+00A0), which is the
dangerous direction; and `btrim`'s default is a smaller subset still.

Two more exactness points, both verified on PostgreSQL 16 (§3.2):

* `coalesce` is still required: an absent key, a `NULL` payload and a **non-object** payload all yield
  `NULL` from `->>`. Rust's `payload.get(key)` answers `None` for all three, so SQL's `NULL -> '' ->
  not stated` is the same answer, not a different one.
* `payload ->> 'gate' = event_id` compares the **text** form. A JSON `null`/number/array `gate` yields
  `NULL` (never a spurious match), which is what Rust's `.and_then(as_str)` gives too.

### 1.2 Q2 — is the `LIKE` segment test the right encoding of `is_gate_id()`? **Yes, and it is proven, not asserted.**

`gate.rs::is_gate_id` is `ends_with("-live") || contains("-live-")`. The SQL twin is

```sql
event_id LIKE '%-live-%' OR event_id LIKE '%-live'
```

and it is **equivalent**, checked two ways (§3.1):

* **All 801 ids in the live ledger** (every `canonical_events.id`, `dispatch_dependencies.dep_value`,
  `dispatches.id`, `artifacts.id`): SQL and the Rust predicate agree on all 801. The two gate-shaped
  ids are `ev-knowledge-write-path-live` and `ev-live-019-applied`.
* **31 adversarial ids**: `ev-live`, `ev-live-`, `ev--live`, `ev--live-`, `ev-X-live-Y`, `ev-xlives`,
  `ev-livex`, `ev-LIVE`, `ev-Live`, `ev-live-live`, `ev-live--live`, `x-live`, `live`, `-live`,
  `ev-%-live`, `ev-_live`, `ev-live_%`, `ev-livé`, `ev-über-live-mü`, `ev-019-live-applied`, … —
  **0 mismatches**. Case, `%`, `_`, unicode, empty trailing segments: all agree.

No escaping hazard either way: the patterns contain no `_` and no `\`, and `dep_value` is always the
**subject** of the match, never the pattern, so a `%` inside an id cannot smuggle a match in.

**And yes, it is a superset of ruling 2's literal rule — that is required, not sloppy.** Ruling 2 says
new gate ids are uniformly `ev-<capability>-live`, but the first gate actually emitted under it,
`ev-live-019-applied`, ends in `-applied`. A suffix-only test would read Aegis's own real, fully
verified gate as an announcement. Nimbus's finding stands; §5 records what I think should be settled.

### 1.3 Q3 — "stays blocked + one escalation" vs a reportable terminal state? **Accept blocked + escalation. Do NOT add a terminal state.**

I accept Nimbus's design, and I did not accept it passively.

**Why not a seventh status value.** `gate_unverified` would not be *terminal* in the sense 013's
`failed`/`cancelled` are: a qualified gate could still land later and release it. So it would be a
sub-state of `blocked` with **no new release path**, bought at the price of a seventh status value, a
seventh transition event type, and a Lyra re-sign-off of the value-domain contract. `blocked` is
already the honest state — the precondition genuinely has not been met, whatever row landed. The
difference is visibility, and visibility is not what a status column is for.

**What I did not accept passively.** "Not silent" was, until now, a claim about Rust code this seat
does not own and cannot verify from a migration. So I made the condition **checkable from the
database**: `v_dispatch_gate_held` (§2.3), a pure read that implements *exactly* the predicate the
pull path escalates on — non-terminal stored status, **every** referent exists, and at least one
gate-shaped event dep is not a qualified gate. One query now answers "is anything held this way, and
by which edge", without trusting the escalation's emission.

**And I found why the escalation matters more than the plan assumed** (§3.6): once a gate-shaped id has
been spent on an announcement row, that id can **never** become a qualified gate. The ledger is
append-only and `canonical_events.id` is the primary key — there is no UPDATE that could turn the
`ArtifactCreated` into a `SupervisorMessage`. So "a dispatch that can never get a qualified gate" is
not a hypothetical; it is the **guaranteed** outcome of the 10-10 shape. Adding a second edge does not
help either: `satisfied = total` requires *all* edges, and the stuck one stays stuck. The recovery is
`cancel_dispatch` (issuer-side) + a re-issue carrying the correct edge — which is precisely what ruling
5 and `reissue_of` now govern. Somebody has to be able to *see* that this is the situation, and now
they can.

### 1.4 Q4 — does `dispatch_dependencies` need an explicit `requires_gate` column? **No — and it would not close the hole it is meant to close.**

The decisive argument: a `requires_gate BOOLEAN NOT NULL DEFAULT false` column has **exactly the same
opt-in default** as the naming convention. `false` means existence semantics. The real fragility is not
"the convention is inferred rather than declared" — it is **"the issuer meant a gate but did not say
so"**, and a defaulted column has that hole in identical shape and identical direction. The column is a
rename of the hole, not a fix for it.

What a column *would* buy is declared intent, so the database could check the declaration against the
row that eventually lands. That is real. But it would be a **fourth** copy of one bit — `emit_event`'s
producer check, `enqueue_dispatch`'s edge check, `dep_satisfied`'s release check, and the column. The
ruling's own diagnosis is that copies of one criterion drift; adding a fourth to fix the third is the
wrong direction.

So I kept the convention and made the part it *can* guarantee **database-enforced** instead of
tool-promised: the new CHECK (§2.2) refuses a gate-shaped `dep_value` on a non-event edge, so a writer
that bypasses the tool — a script, a manual `psql` session, the integration tests that build the 10-10
ledger state by direct SQL — cannot create that shape either.

**The condition under which I would change my mind:** if Aegis retires `-live` as the gate marker
(§5), the discriminator has to live somewhere, and an explicit column is the only candidate left. I
would want that as its own migration with Lyra re-sign-off, because it widens the write path's contract,
not because it is tidier.

**Residual, stated plainly:** "a `-landed` edge hung on a dispatch that really will call the capability"
remains mechanically undetectable (Nimbus §8 gap 1). The edge is not gate-shaped, so it does not appear
in the view either. Only ruling 3's criterion and seat discipline cover it. I am not claiming otherwise.

---

## 2. What shipped

### 2.1 `infra/postgres/schema/020_gate_criterion.sql`

Four objects, all guarded so a second apply performs **no catalog write** (§3.7):

1. **`fn_gate_is_gate_id(TEXT)`** — IMMUTABLE, the SQL twin of `gate.rs::is_gate_id`.
2. **`fn_gate_row_is_gate(TEXT, TEXT, JSONB)`** — IMMUTABLE, the SQL twin of `gate.rs::row_is_gate` /
   `gate_payload_shape`, including the enumerated whitespace set from §1.1.
3. **`fn_dispatch_recompute_ready()`** — body replaced. The gate branch is
   `dep_kind = 'event' AND fn_gate_is_gate_id(dep_value)`, then existence **and** `fn_gate_row_is_gate`.
   Every other branch is 013's, verbatim.
4. **`v_dispatch_gate_held`** — the read-only visibility surface from §1.3.

**One criterion per language, and structurally so.** `gate.rs` is the one place on the Rust side;
`fn_gate_row_is_gate` is the one place here. The trigger, the CHECK and the view all call the same two
SQL functions, so the four call sites cannot drift apart from each other — only across the language
boundary, and that boundary is what §3 pins.

**Why `CREATE OR REPLACE` and not `DROP` + `CREATE`** (013's header anticipated the latter): the three
013 triggers `trg_dispatch_ready_on_event / _on_artifact / _on_dispatch` are bound to this function.
A `DROP FUNCTION` would take them with it (or fail without `CASCADE`). Replacing the body in place is
the only change that leaves the trigger wiring untouched — same name, same signature, same return type.

### 2.2 The naming convention, enforced by the database

```sql
CHECK (dep_kind = 'event' OR NOT fn_gate_is_gate_id(dep_value))
```

on `dispatch_dependencies`, guarded on the constraint's absence. A gate is always an EVENT row — the
criterion reads `event_type` and `payload`, and only `canonical_events` carries them — so a gate-shaped
`dep_value` on an artifact/dispatch edge is a mistake, not a preference. This is `enqueue_dispatch`'s
`GateEdgeUndeclared` refusal one layer down. **Reversible**: drop the constraint in a later migration if
the convention is ever retired. It is the one addition here that is my judgement call rather than the
ruling's literal demand; the reasoning is §1.4.

### 2.3 `v_dispatch_gate_held`

```sql
SELECT dispatch_id, project_id, target_seat_id, status, held_by
  FROM v_dispatch_gate_held;
```

Empty result = nothing is held this way. `held_by` lists the offending `dep_value`s. Exactly the
escalation predicate, expressed as a read (§1.3).

### 2.4 Where I deviated from Nimbus's §4 draft SQL, and why

Three places. All three make the SQL **stricter or identical to 013**, never looser.

1. **Non-event dep kinds.** Nimbus's draft widens them to "exists in **any** of the three tables". I
   kept 013's kind-specific mapping. 013 matches `dep_kind` to its own table precisely because referent
   id spaces collide across the three tables (all TEXT); the draft would let an *artifact*-kind edge be
   satisfied by an *event* row with the same id. That is a **loosening** outside the ruling's scope —
   the ruling asked me to tighten one branch, not loosen three.
2. **Unknown `dep_kind`.** 013 yields `NULL` (counted as unsatisfied). Nimbus's draft would fall
   through to the event-existence `ELSE`. Preserved 013's `NULL`. (`dep_kind` is `NOT NULL` with three
   values in practice, but "in practice" is not a schema fact, so the semantics are kept exact.)
3. **The whitespace trim** — §1.1, the enumerated set rather than `btrim`'s default.

Everything else in the draft is landed as written, including the gate branch's shape and the
`total > 0 AND satisfied = total` guard.

### 2.5 `scripts/apply-db.sh`

One POST_SEED entry after 019, with a rule-based reason (no seed interaction; depends on 013's function
and 012's table; canonical order 013 → 020). `bash -n` clean; the completeness guard passes — every
migration on disk is classified, so no database can be built quietly incomplete.

---

## 3. Verification — scratch only (live read-only)

Environment: `postgres:16-alpine`, the running `seatloom-postgres` container. Scratch DBs
`seatloom_scratch_020` (full branch build 001–019 + 020) and `seatloom_scratch_020b` (001–014 only, the
intermediate state); both dropped after. Live was read for the census and the pre-flight, never written.

### 3.1 `is_gate_id` equivalence — 0 mismatches

801 live ids + 31 adversarial ids, SQL vs the Rust predicate (§1.2). `diff` empty both times.

### 3.2 `fn_gate_row_is_gate` battery — 31 cases, 0 mismatches

Well-formed gate → true. `ArtifactCreated` with the same payload → false. `payload.gate` naming another
id / absent / `null` / numeric → false. Each of `statement` / `verified` / `does_not_assert` missing or
empty → false. Whitespace-only `verified` — spaces, tab, newline, CR/VT/FF, U+0085, U+00A0, U+1680,
U+202F, U+205F, U+3000 → **false in every case**. Padded-but-real fields → true. The 10-10
announcement shape (`branch`/`commit`/`migration`) → false, including as a `SupervisorMessage`. Empty
object, `NULL` payload, JSON scalar, JSON number, JSON array → false. `event_type` NULL, `event_id`
NULL → false. The real `ev-live-019-applied` payload → true.

### 3.3 The trigger, end to end — six dispatches, one per dep shape

| # | Scenario | Result |
|---|---|---|
| 1 | an `ArtifactCreated` lands under a gate-shaped id (the 10-10 row) | **held, `blocked`** — correct |
| 2 | an ordinary event edge and an artifact edge land | **both released to `ready`** — 013 behaviour preserved |
| 3 | a dispatch-kind edge onto a *still-blocked* dispatch | **held** — correct |
| 4 | a `SupervisorMessage` under a gate-shaped id with `verified: "   "` | **held** — an unmeasured gate is not a gate |
| 5 | a real measured gate under a clean id | **released to `ready`** — the criterion does not over-block |
| 6 | the append-only consequence (§1.3) | the announcement row is still there; that id can never become a gate |

Case 2 is the control that makes case 1 non-vacuous: if nothing were ever released, "held" would prove
nothing.

### 3.4 The CHECK

A gate-shaped `dep_value` on an `artifact` edge → refused (`check_violation`). On a `dispatch` edge →
refused. On an `event` edge → **accepted** (a gate-shaped event edge is exactly the legal case).

### 3.5 The view

Returns exactly the two held dispatches with their `held_by` lists, and nothing else. The released ones
are absent; the zero-referent-missing precondition holds for both.

### 3.6 The one criterion in one place

`pg_get_functiondef('fn_dispatch_recompute_ready()')` contains `fn_gate_is_gate_id`; the CHECK exists;
both helpers exist; `pg_notify('dispatch_ready', …)` is still in the body (the wake path is unchanged).

### 3.7 Idempotency and state-independence

* **Second and third apply of 020**: the guards fire (`NOTICE: … already carries the gate criterion` for the
  function and for the view), and the catalog fingerprint — oid **and** `xmin` of all three functions, the
  constraint and the view, the full `pg_get_functiondef` text, **and all five `obj_description` comments** —
  is **identical** across three applies. Zero catalog writes on a re-apply.
* The comments are guarded too, deliberately: an unconditional `COMMENT ON` is a `pg_description` write on
  every apply, so leaving them unguarded would make the idempotency claim false in the one place it is
  cheap to make true.
* **A migration that fails, fails whole.** The first draft of this file referenced a cast (`::regconstraint`)
  that does not exist in PostgreSQL 16; the DO block raised, `ON_ERROR_STOP` aborted, and the enclosing
  transaction rolled back — leaving the database at 001–019 with **no** trace of 020, not a half-applied
  one. That is the correct failure mode, and it is also why this section exists: an end-to-end apply on a
  scratch database is the only thing that distinguishes "written" from "landed".
* **Intermediate state**: 020 applied onto a 001–014 database (no 015–019) → exit 0, criterion live, all
  three 013 triggers intact. It depends on 013 and 012 and nothing later.

### 3.8 Live pre-flight (read-only) — what the live apply would change today

* **0** live rows violate the new CHECK (all 19 `dispatch_dependencies` rows).
* **0** live dispatches would be newly held: the only gate-shaped edge in the live ledger is
  `ev-live-019-applied` on `disp-onight-writein-v3`, which is already `done` — and that referent **is**
  a qualified gate (`SupervisorMessage`, `payload.gate` = its own id, all three fields stated).
* The one live `blocked` dispatch, `disp-7Q0V28_5iCPcoMaDDPfKQ` (seat-nimbus-001), waits on
  `ev-verdict-onyx-gate-release-check` — an ordinary existence edge, untouched by this change.

So the live apply of 020 changes **zero live behaviour today**. It only changes what happens the next
time somebody hangs a gate-shaped edge on an announcement.

---

## 4. The live apply is Aegis's act (not mine, not Nimbus's)

Per lesson 7 (`code-landed` ≠ `live-applied`) this delivery claims nothing about live. 020 is landed
and scratch-verified; **live is unchanged**. The command, for the operator:

```bash
podman exec -i -e PGPASSWORD=seatloom seatloom-postgres \
  psql -U seatloom -d seatloom -v ON_ERROR_STOP=1 -q -f - \
  < infra/postgres/schema/020_gate_criterion.sql
```

then the four verification queries in the migration header. Note `scripts/apply-schema-to-live.sh`
currently loops 015–018 only and was not extended for 019 either, so it was left alone here rather than
half-updated.

**A note on what this gate is and is not.** The release-time check is now mechanical in both languages.
That closes the *release* direction. It does not close the *edge-attachment* direction: an edge hung on
a `-landed` announcement for a dispatch that really will call the capability is still only covered by
ruling 3's criterion and seat discipline (§1.4). Nimbus §8 says the same. Nothing here should be read as
closing it.

---

## 5. For Aegis to settle — the id naming rule

Nimbus flagged it; I confirmed it from the live ledger. Ruling 2 says new gate ids are
`ev-<capability>-live`. The live ledger's two gate-shaped ids:

| id | shape | qualified gate? |
|---|---|---|
| `ev-knowledge-write-path-live` | ends `-live` | yes |
| `ev-live-019-applied` | ends `-applied`, carries a `-live-` segment | yes |

So the first gate emitted under ruling 2 does **not** satisfy ruling 2's literal rule. Both the Rust and
the SQL criterion therefore use the **segment** test, which recognises both shapes and can only
over-recognise (one extra payload read), never under-recognise. My view: **keep the segment test and
amend ruling 2's wording** to "a gate id carries `live` as a segment", rather than narrowing the test to
a suffix rule — narrowing it would read a real verified gate as an announcement, which is the exact
failure the ruling was written to stop. This is a wording amendment to the ruling, not a schema change;
nothing in 020 needs to move either way, because the marker is the segment test in both languages.

---

## 6. Residual gaps — not closed, not claimed

1. **A `-landed` edge on a dispatch that will really call the capability** — undetectable mechanically
   (§1.4, Nimbus §8 gap 1). The view does not surface it: the edge is not gate-shaped.
2. **A first issue carrying an undeclared, currently-false precondition** — undetectable (Nimbus §8
   gap 2). Only discipline.
3. **An id spent on an announcement can never become a gate** (§1.3) — not a gap in 020, a property of
   the append-only ledger. It is why the escalation and the view matter, and why the recovery is
   cancel + re-issue rather than "wait longer".
4. **The criterion is enforced at release, not at edge-attachment** — by design (ruling 4's tension).
   A gate-shaped edge hung today is not validated until the referent lands.

---

## 7. Completion flow

1. ✅ Committed to `packet/onyx/topic-opened-019` @ `f97d0a8` (3 files: the migration, the `apply-db.sh`
   POST_SEED entry, this doc; commit carries the dispatch id).
2. ✅ This delivery doc.
3. ✅ `ev-020-gate-criterion-landed` — `ArtifactCreated`, subject `art-020-gate-criterion`, actor
   `seat-onyx-001`, context `disp-5YCwjJF-oIzwb-d1rmrs5`. Deliberately an **announcement** id
   (`-landed`, not `-live`) per ruling 2: it says the SQL is on a branch at a commit, never that the
   capability runs.
4. ✅ `ev-verdict-onyx-gate-release-check` — `ReviewVerdictIssued`, the exact pre-declared id. This is
   what releases `disp-7Q0V28_5iCPcoMaDDPfKQ` (seat-nimbus-001), which was `blocked` on it; the row is
   now `in_flight` with seat-nimbus-001, so the criterion and the release path are confirmed working
   end to end on the live ledger, not only on scratch.
5. ✅ Forward recording via `remember` — 4 records on `topic-aegis-fleet-operations`, source
   `ev-verdict-onyx-gate-release-check`: 2 decisions (the asymmetric-failure rule for one criterion in
   two languages; a gate-shaped id spent on an announcement makes that edge permanently
   unsatisfiable, so the recovery is cancel + re-issue), 1 failure (the `::regconstraint` cast that
   rolled the whole transaction back, and the rule it produced: a migration is landed only after an
   end-to-end apply on a same-major-version scratch DB with the catalog read back), 1 unresolved (the
   `-landed`-edge intent gap and ruling 2's wording vs `ev-live-019-applied`, both for Aegis).
6. `complete_dispatch(disp-5YCwjJF-oIzwb-d1rmrs5)`.

**Still open, and not mine to close:** the live apply of 020 is Aegis's act (§4). Until it runs, live
keeps 013's existence-only release test. Nothing in this delivery claims otherwise.
