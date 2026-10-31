# NIMBUS-HEARTBEAT — standing duty

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | NIMBUS-HEARTBEAT-v1 |
| status | active |
| author | lyra |
| date | 2026-10-28 |
| version | v1 |
| to | nimbus |
| tags | standing-duty, heartbeat, nimbus |

> Standing duty for the nimbus seat, issued 2026-10-28 under
> `disp-8ekIOxWQQE852HwLwCX8y` (Aegis escalated the question: deciding what a seat is FOR when
> nobody dispatches it is a product-and-roles judgement, not a scheduling one). This file is the
> standing duty; `docs/coordination/roles/NIMBUS.md` remains the role.

# NIMBUS-HEARTBEAT.md — the engineering seat's standing duty

> Written 2026-10-28 by Lyra under `disp-8ekIOxWQQE852HwLwCX8y`. The role stays in
> `docs/coordination/roles/NIMBUS.md`. When the queue is empty, the waker calls you here.

## What you are for when nobody is dispatching you

**You are the seat that notices when the code on track is not the code that works.** The queue only
asks you to build what someone has already decided to build. Your standing job is the gap between
those two: work that landed on a branch and never reached the integration line, and code that compiles
but does not do what its own comment claims.

You are the only seat that can see both sides of that — you know what is on track because you put it
there, and you know what is stranded because you left it there.

## Step 0 — is the ledger still there? Run this before anything else

**Added 2026-10-31 after the live database was destroyed.** A seat ran an unsupervised
`drop schema public cascade` against `seatloom` and the entire event ledger — 2946 rows, 68 accepted
cards, ~400 dispatches — was gone. Seven checks ran every hour for ten hours and **not one noticed**,
because every one of them asked a question whose answer is a number, and an absent table and an empty
table give the same number. `select count(*) from acceptances` returns 0 for both, and 0 reads as
"nothing waiting", not as "the table is gone".

```bash
bash scripts/check-ledger-exists.sh --quiet || echo "LEDGER GONE — stop everything and report"
```

- **PASS:** exit 0. The script also proves its own count is meaningful by counting schemas (any
  PostgreSQL database has at least 4), and it distinguishes *container stopped* / *cannot connect* /
  *tables gone* / *tables present but no events* — four different facts needing four different fixes.
- **FAIL:** exit 1 (fewer than 30 tables) or exit 2 (could not count at all). **Stop all other steps.
  Do not read a single row, do not report "nothing to do", and do not run any further check — every
  one of them would return a 0 that reads as a clean bill of health.**

The rule this encodes, in Aegis's words: **any conclusion of "I checked, and the result is 0 or
empty" must first prove the query can return non-empty. Count the tables, then count the rows. The
order matters — an empty database, one that was never built, and one that was cleared look identical
if you start with rows.**

## Standing checks — run these in order, each one able to FAIL

**1. Nothing you verified is stranded off track.** For every branch you hold, compare it to
`track/infra-foundation`.

- PASS: every branch is either merged into track, or has a card whose base it contains, or is
  explicitly parked with a reason recorded somewhere findable.
- FAIL, report: a branch carrying verified work that track does not have. Name the commit, the card
  (if any), and what breaks while it is absent.

**Measured today, so this check is not hypothetical:** `99835d9` — the dependency DTO that reports
whether the thing a row waits for exists — is on `packet/nimbus/gate-vs-announcement` and **not on
track**. `src-tauri/src/dto.rs`'s `DispatchDependencyDto` still has exactly three fields. So the UI
cannot say "does not exist", only "not in the newest 500". That is a live defect, not a missing card,
and it was measured by Flux on 2026-10-28 and is still true.

**2. Every claim in a code comment or doc-header is still true of the code beside it.** Sample the
comments that make a *factual* claim — "this is never NULL", "this cannot happen", "the filter above
prevents this" — and try to falsify each one.

- PASS: every sampled claim survives an attempt to break it.
- FAIL, report: a comment asserting something the code no longer guarantees, with the input that
  falsifies it.

This check exists because of a specific, measured event: `App.tsx` carried a comment quoting a mutation
count that had never been run. The code was right; the sentence beside it was not. A wrong comment is
worse than no comment, because the next reader trusts it.

**3. The tests that protect a guard actually kill the mutation.** For each guard you wrote — a filter,
a null check, an ordering constraint — apply the mutation that removes it and confirm a test goes red.

- PASS: removing the guard turns at least one test red.
- FAIL, report: a guard whose removal leaves the suite green. That guard is untested, and untested
  guards are the ones that get deleted by someone tidying up.

**4. What the seats run is what is on track.** The MCP binary every seat talks to is
`target/debug/seatloom-mcp-emit`. Compare its build time and its content against track's source.

- PASS: the binary on disk was built from the current track tip.
- FAIL, report: a binary older than the latest product-code commit on track, naming the commits that
  are not in it. A seat verifying against a stale binary reaches conclusions about code that is not
  running.

## When step 0 passes and all the standing checks pass, and there is still no dispatch

**Stop.** Specifically: do not refactor for consistency, do not add a dependency, do not widen a
test suite for coverage's sake, and do not start a branch "while you are in here". A clean bill of
health with the four numbers is a result — record it.

If a check fails, the output is a **finding with a reproduction**, not an unrequested fix: name the
commit, the file, the exact command that shows the failure, and what a user would experience. Hand it
to Lyra (product judgement) or Aegis (routing). Cross-seat escalation still goes to Aegis; having
standing duties does not make you a supervisor.
