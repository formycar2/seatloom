# Delivery: Supervisor IM shows the newest events, not the oldest

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation-delivery |
| id | NIMBUS-2026-10-11-supervisor-im-window-fix-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-10-11 |
| version | v1 |
| tags | nimbus, supervisor-im, read-model, ordering, bugfix |
| governing dispatch | `disp-supervisor-im-shows-oldest-not-newest` |
| base branch | `track/infra-foundation` |
| delivery branch | `packet/nimbus/supervisor-im-window-fix` |
| scratch database | `seatloom_supim3` (001–012 + seed, fresh; NEVER live `seatloom`) |

---

## 0. The bug, and the one-line shape of the fix

`list_supervisor_messages` selected `ORDER BY occurred_at ASC LIMIT n` — the
**oldest** n events. With 515 events in the ledger and the UI asking for 400, the
operator's window covered 04-27 → 10-10 18:45 and hid the newest 115, including
the message he sent himself at 10-11 17:30. His "no valid messages" was an
accurate description of what he could see: five months of dispatch-lifecycle
noise.

The UI does no client-side sorting and appends live events at the tail, so it
expects ascending order with the newest last. The fix is therefore not `ASC` →
`DESC` (which would invert the reading order) but: **select the newest `limit`
rows, then reverse**.

```sql
SELECT ... FROM (
  SELECT ... FROM canonical_events
   ORDER BY occurred_at DESC, id DESC LIMIT $1
) recent
ORDER BY occurred_at ASC, id ASC
```

Both branches (seat-filtered and unfiltered) got the same shape.

`occurred_at` alone is not a total order — events can share a timestamp — so `id`
is the tiebreaker in **both** directions. Without it the reversal could
reshuffle equal-timestamp rows differently on each call, and the window would not
be stable. (The tiebreak is lexical because `id` is TEXT; the test uses
zero-padded ids and says so.)

---

## 1. The two judgments the dispatch asked for

### 1.1 What `limit` now means

After the fix the UI's `limit: 400` returns **the newest 400 events**, ascending.
That is the reading the chat wants and what the operator expected.

Whether the supervisor feed should show only `SupervisorMessage` events rather
than all 515 is a **different decision** — it changes the filter, not the
ordering — and this delivery does not make it. Stated for the record: today the
feed mixes the operator's messages with seat activity events (that mixing is
documented as intentional in the method's own doc comment: a seat-targeted feed
shows the seat's own events too). Narrowing it is a product call, and it would
also change what the seat-filtered branch returns. Recommendation if it is taken:
filter by `event_type = 'SupervisorMessage'` for the unfiltered (human) view and
keep the seat-targeted branch as is — but that is a separate dispatch.

### 1.2 Indexing `ORDER BY occurred_at`

**No new index is needed.** `canonical_events` already carries
`idx_events_occurred ON canonical_events(occurred_at DESC)` (001) and
`idx_canonical_events_project_occurred ON canonical_events(project_id,
occurred_at DESC)` (008). The new shape is a `LIMIT` over that index followed by a
sort of at most `limit` rows — the inner scan is index-driven, the outer sort is
bounded by the window, so it stays cheap as the ledger grows. Adding another
index on the same column would only add write cost. The tiebreaker `id` is not
indexed, but it is applied to at most `limit` rows, not to the table.

---

## 2. Tests

New file `crates/seatloom-core/tests/supervisor_im_window_integration.rs`
(`#[ignore]`, scratch DB, refuses the live database):

| Test | What it pins |
|---|---|
| `a_window_larger_than_the_data_returns_everything_ascending` | 120 events, limit 400 → all 120, ascending, oldest and newest both present |
| `a_window_smaller_than_the_data_returns_the_newest_ascending` | 121 events, limit 50 → the newest 50, ascending, **the operator's own message is the last row** (the regression) |
| `equal_timestamps_keep_a_stable_deterministic_order` | 10 same-timestamp events → the window is identical across two calls, ordered by the id tiebreak |
| `the_seat_filtered_branch_returns_the_newest_window_ascending` | the seat-filtered branch gets the same treatment |

Gates (Rust 1.95.0; fresh scratch DB `seatloom_supim3`, `seatloom-cli reconcile`
run first as CI does):

| Gate | Command | Result |
|---|---|---|
| fmt | `cargo fmt --all --check` | PASS |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | PASS (0 findings) |
| compile | `cargo check --workspace --all-targets` | PASS |
| core tests | `cargo test -p seatloom-core -- --include-ignored --test-threads=1` | PASS (116 + 3 lineage + 6 D2 + 27 baseline + 3 seed + 4 supervisor-IM) |
| mcp-emit tests | `cargo test -p seatloom-mcp-emit -- --include-ignored --test-threads=1` | PASS (67) |

Not run: the UI gates (`pnpm typecheck` / `pnpm test`) — this delivery touches no
`ui/**` file; the fix is entirely in the query the UI already calls. (The
pre-existing `db_baseline_integration` fixed-id collisions appear only when the
suite is run twice against one database; a single fresh-DB run is green, as
above, and CI always runs against a fresh database.)

---

## 3. What this does NOT fix — stated plainly

**The operator's message still gets no reply.** `cmd_append_supervisor_message`
writes the event and nothing reads it: no agent is attached to that surface. The
original plan was to run the `Assistant_Aegis` runtime (outside this repository)
on pi + models-proxy with the SeatLoom MCP tools attached; that was never done.
After this delivery the operator will **see** his own message and the seat
activity, and still get no answer. The interface is not usable as a conversation
yet — it is a readable log. Making it answer is a separate piece of work with its
own owner.

---

## 4. Notes

- The change is a pure read-model ordering fix: no schema change, no event
  change, no write-path change, `infra/**` untouched.
- Forward recording (per AGENTS.md): `remember` is not callable from this session
  — the seat's MCP process is still the pre-plan image (reported to Aegis in
  `ev--dbmkxtmH7DTTh3X2uZf0`; this seat's agent process was not included in the
  fleet swap). Recorded here instead of skipped.
- No whole-branch merge; no live-DB write.
