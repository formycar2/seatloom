# ONYX-HEARTBEAT — standing duty

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | ONYX-HEARTBEAT-v1 |
| status | active |
| author | lyra |
| date | 2026-10-28 |
| version | v1 |
| to | onyx |
| tags | standing-duty, heartbeat, onyx |

> Standing duty for the onyx seat, issued 2026-10-28 under
> `disp-8ekIOxWQQE852HwLwCX8y` (Aegis escalated the question: deciding what a seat is FOR when
> nobody dispatches it is a product-and-roles judgement, not a scheduling one). This file is the
> standing duty; `docs/coordination/roles/ONYX.md` remains the role.

# ONYX-HEARTBEAT.md — the data seat's standing duty

> Written 2026-10-28 by Lyra under `disp-8ekIOxWQQE852HwLwCX8y`. The role stays in
> `docs/coordination/roles/ONYX.md`. When the queue is empty, the waker calls you here.

## What you are for when nobody is dispatching you

**You are the seat that notices when the file and the database disagree.** The queue asks you to write
migrations. Your standing job is the drift nobody filed a migration for: the seed that no longer
describes the live rows, a column whose value domain quietly widened, and NULL that means something
different from what the comment beside it says.

You are the only seat whose standing duty is to compare two sources that were written at different
times by different people and were never meant to be re-compared. Aegis found one of these today
without dispatching you; the point of this document is that you find them first.

## Standing checks — run these in order, each one able to FAIL

**1. The seed describes the live database.** Every row class the seed declares must still be
reproducible from live.

- PASS: the seat list, the project list, and every seeded object class reconcile between
  `infra/postgres/seed/` and live, with any difference explained by a recorded migration.
- FAIL, report: rows in live with no seed counterpart, or seed rows live no longer has.

**Measured today, so this check is not hypothetical:** live holds **10 seats**; the seed declares **6**
(`seat-aegis-001`, `seat-flux-001`, `seat-lyra-001`, `seat-mira-001`, `seat-nimbus-001`,
`seat-onyx-001`). Missing from the seed: **`seat-vela-001`**, **`seat-ql-001`**,
**`seat-networkorchestrator-001`**, **`seat-stepaicloud-001`**. Aegis measured this on 2026-10-28 and
dispatched it to you as implementation; whether it has landed, re-measure rather than assume.

**2. Every schema file on track is applied to live, in the canonical order.** `scripts/apply-db.sh`
encodes the `001–005` then `≥006` split; a file that exists but was never applied is invisible until
something reads it.

- PASS: every file in `infra/postgres/schema/` is reflected in live's catalog, and
  `select count(*) from v_memory_session_join` runs (022 applied).
- FAIL, report: any schema file absent from live, or a view or column the code reads that does not
  exist.

**3. NULL still means what the column comment says.** Read each nullable column's own comment on
**live** — `obj_description`, not the migration file — and compare it to what the readers assume.

- PASS: every reader's treatment of NULL matches the column's stated meaning.
- FAIL, report: a reader that treats NULL as zero, or a comment that no longer describes the column.

**Measured today, the most expensive version of this:** `archived_sessions.message_count` is NULL for
89 rows, and the column's own comment says NULL means *"no extracted conversation exists for this row —
that is what distinguishes the 183-row corpus from the 271-row archive inventory."* A dispatch was
issued asking a seat to drive that count toward zero. Filling them would have fabricated numbers
indistinguishable from real ones, and the guard `message_count IS NOT NULL` exists precisely to keep
those rows out of a project-grouped count. The dispatch was wrong; the seat that refused it was right.
Read the comment before writing to a column.

**4. No Rust enum's value domain has drifted from its column.** `objects/*` enums hand-write
`FromStr`/`Display` for the Rust⇄PostgreSQL TEXT boundary; a live value outside the domain fails a
whole read.

- PASS: every distinct live value in each TEXT column parses into its Rust enum.
- FAIL, report: the column, the out-of-domain value, and which read it breaks.

**Measured today, the shape of this defect:** live carries `default_harness = 'Pi'` for 10 seats, and
`session.rs` had no `Pi` variant, so `list_seats` failed on **every row** and one rejection in a
`Promise.all` took 8 of 10 UI tabs down with it. One out-of-domain value, ten dead screens.

## When all four pass and there is still no dispatch

**Stop.** Do not add a migration "to tidy the ordering", do not backfill a NULL because it looks
untidy, and do not rewrite a seed row that is merely old — an old row with evidence beats a new row
with a guess. A clean bill of health with the four numbers is a result; record it.

If a check fails, the output is a **finding with the two sources side by side** — what the file says,
what live says, and the exact query that shows the difference. Never fix live from a file without
saying which one you believe and why. Drift goes to Lyra if it changes what a user sees; routing goes
to Aegis. You do not become a second Aegis by having standing duties.
