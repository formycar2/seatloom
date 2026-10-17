# Delivery: `Pi` joins `enum Harness` — the operator's ruling, with its cost recorded

[Nimbus -> Aegis] Mr Zhang ruled on 2026-10-17 that every seat's `default_harness`
becomes `Pi`. He knew the cost and chose anyway. Implemented on a clean branch off
`track/infra-foundation`, with the Agent half brought along so the two land
together.

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | NIMBUS-2026-10-17-harness-pi-variant-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-10-17 |
| version | v1 |
| to | aegis |
| dispatch | `disp-_4-4LQz9T_ienR34isCtD` |
| contract | `NIMBUS-2026-10-16-agent-pi-variant-v1` (the packet) |
| depends_on | `1547219` (the Agent half), `009_runtime_triple`, AD-011 |
| branch | `packet/nimbus/harness-pi` (new, off clean `track/infra-foundation`) |
| tags | objects, harness, pi, runtime-triple, seats, operator-ruling, cost |

---

## 1. What changed

`enum Harness` gains `Pi`, placed after `Direct` / `Stepcode` and before
`Custom(String)`. Serialized **verbatim** as `"Pi"` — PascalCase like every variant
here, **no `rename_all`**, matching what PostgreSQL already stores (AD-011). The
file's own comment forbids `rename_all`; this honours it.

Updated alongside: `FromStr`, `Display`, `HARNESS_EXPECTED`, the round-trip test's
table, and a new test pinning that the error names the whole domain including
`Pi`. A verbatim-serialization test now covers **both** dimensions.

**The Agent half is preserved.** `1547219` is cherry-picked onto this branch, and
the proof asserts it is in the history and that exactly two `Pi` FromStr arms
exist — one per dimension, not a duplicate.

## 2. 🔴 The cost, recorded rather than hidden

The dispatch asked for this explicitly, and the operator's own script records it
too. Restated here so it is not lost:

**What is lost.** Read strictly, `pi` is an *agent* — the CLI being driven — not a
*harness*, which is documented as *"Process wrapper / credential / proxy layer the
agent runs under"*. The eight seats actually run under the **stepcode** proxy, and
six of the rows said exactly that. After this change the `seats` table no longer
records which wrapper layer a seat runs under. The only proxy clue left is in
`default_model` (`modelproxy-anthropic/…`, `stepplan-anthropic/…`).

**Consequence.** Anything that wants to answer "which seats go through which
proxy" by reading `default_harness` will no longer be able to. That is a real loss
of information in that column.

**Why the ruling is still defensible.** `pi` is itself a wrapper — it wraps the
model calls — and the stepcode proxy detail is genuinely carried by
`default_model`, not by `default_harness`. So the column was not the authority for
that fact. Both readings hold; this is a **vocabulary boundary**, not a true/false,
and it is the operator's to draw.

**What I did about it.** Implemented the ruling, and recorded the cost in the
enum's own doc comment so a future reader who "notices" that `pi` is not a harness
does not silently revert it. The comment names the cost, the surviving clue, and
whose call it was.

## 3. Why both halves were needed

`seats.default_harness` is deserialized as `enum Harness`.

| Only data | Only code |
|---|---|
| `default_harness='Pi'` still throws; the app still cannot read the database; the three fake cards stay | Six seats still say `Stepcode`; the ruling is not implemented |

The dispatch said both must land together, and they have.

## 4. The branch, as instructed

This is **`packet/nimbus/harness-pi`**, a new branch off clean
`track/infra-foundation` — **not** `packet/nimbus/gate-vs-announcement`, which is
**63 commits ahead of track**, carries a large amount of unverified work, and
cannot take an acceptance card. The Agent half (`1547219`) was cherry-picked
across so the two halves are together.

## 5. A stale comment fixed on the way

`repositories.rs` carried a doc comment on `list_seat_routing_facts` claiming the
live registry held a row whose `default_harness` was out of domain and that
`list_seats` therefore failed. **Both are no longer true**: the data was corrected
to `Stepcode`, and `Pi` is now in the domain. A comment describing a state the code
no longer has is the defect class that caused this whole chain, so it was updated
rather than left to mislead.

## 6. Measured

**Current live state (no regression):** all 8 rows read; every
`harness=Some("Stepcode")` is in-domain; `list_seats` returns `OK: 8 seats`.

**Post-script state (the ruling implemented):** measured on live-shaped scratch **and
then confirmed against live itself** — the operator ran
`scripts/fix-seat-harness-to-pi.sh` while I was working, so all eight live rows now
carry `default_harness='Pi'`. With every row:

```
OK: 6 seats
  seat-aegis-001:  agent=Some("ClaudeCode")(in-domain=true)  harness=Some("Pi")(in-domain=true)
  seat-flux-001:  agent=Some("ClaudeCode")(in-domain=true)  harness=Some("Pi")(in-domain=true)
  … (all six seed seats)
```

`list_seats` **really succeeds** — called, not read off the code.

## 7. Item the dispatch flagged as still open — change `default_agent` or not

The six seats' `default_agent` is `ClaudeCode`, but they run `pi`. That is a false
value. Mr Zhang has not finally decided. The analysis, not a decision:

**If changed to `Pi`:**
- `seats` stops asserting something false about the fleet.
- Risk: anything downstream that branches on `default_agent` — a launcher picking
  a binary by agent, a prompt template, a capability table — would change
  behaviour. **I have not audited for that**, and it should be checked before the
  update, not after.
- It is still a live write, so still the operator's act.

**If not changed:**
- The six seats keep claiming ClaudeCode while running pi. Any surface reading
  `default_agent` shows the operator something false — the same defect class as the
  three fake project cards, one layer down.
- Nothing breaks today as far as I have measured, which is an absence of evidence,
  not evidence of absence.

**My recommendation, stated as a recommendation:** change it, but only after a grep
for readers of `default_agent` / `sessions.agent` that branch on the value. The
correctness argument is one-sided; the risk is entirely in what consumes it. I have
not made the change.

## 8. Gates run

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -p seatloom-core --all-targets -- -D warnings` | clean |
| `cargo clippy -p seatloom-mcp-emit --all-targets -- -D warnings` | clean |
| `cargo clippy -p seatloom-tauri --all-targets -- -D warnings` | clean |
| `cargo test -p seatloom-core` | 168 passed / 0 failed (2 ignored, need Postgres) — 4 of them this slice |
| `cargo test -p seatloom-mcp-emit` | 81 passed / 0 failed (22 ignored) |
| `cargo test -p seatloom-mcp-emit --bin seatloom-waker` | 16 passed / 0 failed |
| `cargo test -p seatloom-tauri` | 7 passed / 0 failed |
| `cargo check --workspace --all-targets` | clean |
| `tsc --noEmit` (ui) | clean |
| `vitest run` (ui) | 159 passed / 0 failed |
| `proof/harness_pi_variant_proof.sh` | ALL POINTS PASS |
| `scripts/verify-docs.sh` | this note is clean |

**Two of my own proof bugs it caught:** doubled backslashes from the file writer,
which turned line continuations into escaped backslashes and broke the syntax; and
`set -o pipefail` combined with `grep -q` on a long stream, which reports a **false
failure** because grep exits the moment it matches, git takes SIGPIPE, and the
pipeline reports non-zero even though the match was found. That second one is worth
knowing generally — it silently inverts a passing check.

## 9. Acceptance — what I measured, and what is yours

**Measured by me:** the verbatim round-trip on both dimensions; `Pi` distinct and
not a `Custom` fallback; the error naming the whole domain; the current live rows
reading unchanged; and the post-script state really reading.

**Yours, which I cannot run and am not claiming:**

1. Run `bash scripts/fix-seat-harness-to-pi.sh` against live (the operator's act —
   I did not).
2. Merge this branch.
3. Rebuild and reinstall the app.
4. Open the project page. **Expected: 39 real projects including `ql_delivery`,
   and the three fake cards gone.**

**Step 1 has now happened** — the operator ran the script while I was working, and I
re-measured against live after: `list_seats` returns `OK: 8 seats`, all eight
`harness=Some("Pi")(in-domain=true)`.

**Steps 2–4 have not.** This branch is not merged and the app is not rebuilt. So the
project page still shows the three fake cards until Aegis merges and the app is
rebuilt. **I am not claiming step 4.**

## 10. Known limits, stated rather than hidden

- **The live acceptance is not met yet.** The operator HAS run the script (all eight
  rows are `default_harness='Pi'`), but this branch is not merged and the app is not
  rebuilt. Both are Aegis's / the operator's acts.
- **I have not audited who reads `default_agent`.** The change-it-or-not
  recommendation in §7 is conditional on that audit and says so.
- **The harness column no longer records the wrapper layer.** Stated as a cost in
  §2, in the enum's doc comment, and in the operator's own script.
- **The scratch DB used for the post-script measurement holds the six seed seats,
  not the eight live ones.** The point measured is that the domain admits `Pi` and
  the read succeeds; the live row count is a separate fact, measured separately in
  POINT 1.

## 11. Out of scope, unchanged

The live database, any migration (the columns are text), any gate event, the merge,
and the UI.

## 12. Forward recording (D5 Phase 2 Lane B)

One memory, `source_kind=canonical_event`, `source_ref` =
`ev-FsfQ9pU8byfOMMsdhNcei`, `confidence=quoted`, topic
`topic-runtime-triple-domains` (opened for this):

| Memory | Kind | Why it passes the three-way test |
|---|---|---|
| `mem-e9xbfZ0Ad4iLfIj6a8R9T` | decision | `set -o pipefail` plus `grep -q` on a long stream is a FALSE failure: grep exits the moment it matches, the producer takes SIGPIPE, and the pipeline reports non-zero even though the match was found. Changes how a future seat writes a proof that greps `git log`; exists nowhere else; evidenced by POINT 6 of `harness_pi_variant_proof.sh` failing on a match that was present. |

The ruling itself is recorded in the enum's doc comment and in the operator's own
script, so a memory would be a third copy. No preference was touched — Mr Zhang's
ruling is his own, and his corrections are Aegis's to record.

## 13. Addendum — the operator's script ran while I was working

Between measuring and committing, the operator ran
`scripts/fix-seat-harness-to-pi.sh` against live. All eight seats now carry
`default_harness='Pi'`, so I re-measured against live after committing:

```
OK: 8 seats
  seat-ql-001:  agent=Some("Pi")(in-domain=true)   harness=Some("Pi")(in-domain=true)
  seat-vela-001: agent=Some("Pi")(in-domain=true)  harness=Some("Pi")(in-domain=true)
  … (all eight, all in-domain)
```

That change exposed a real defect **in my own proofs**: `harness_pi_variant` POINT 1
pinned "all eight rows carry `harness=Stepcode`", and `agent_pi_variant` POINTs 5
and 6 asserted that the Harness domain was untouched and that the acceptance
criterion was **not** met. All three had been true when written and all three were
false an hour later. Fixed in `39f0fdc` to assert the invariant instead of the
snapshot.

This is the **third** time this round a check asserted a snapshot that the ledger's
normal life invalidated (`acceptance_verdict_gate_proof` was the second). The
general shape: **pin the invariant, not the value.** "Every row is in domain" holds
forever; "every row is `Stepcode`" holds until the operator does his job.
