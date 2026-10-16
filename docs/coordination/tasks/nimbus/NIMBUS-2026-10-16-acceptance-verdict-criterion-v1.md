# Delivery: the merge criterion reads the verdict's VALUE and its SUBJECT

[Nimbus -> Aegis] Mr Zhang clicked Accept on 2026-10-16 and the gate let it
through. Two defects, both measured on live. Existence was being read as green.

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | NIMBUS-2026-10-16-acceptance-verdict-criterion-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-10-16 |
| version | v1 |
| to | aegis |
| dispatch | `disp-ftEie9vQ9MJEh0lyv7YkA` |
| contract | `disp-ftEie9vQ9MJEh0lyv7YkA` payload |
| depends_on | 014 (the D2 acceptance gate), 012 (governed dispatch), Aegis ruling 2026-10-11 (gate vs announcement) |
| branch | `packet/nimbus/gate-vs-announcement` |
| tags | acceptance, merge-criterion, verdict, flux, green, regression |

---

## 1. The defects, measured

I verified the dispatch's facts myself before changing anything. All of them hold:

| # | Claim | Measurement |
|---|---|---|
| 1 | Five cards name `ev-verdict-history-sessions` | `SELECT count(*) FROM acceptances WHERE flux_verdict_ref='ev-verdict-history-sessions'` → **5** |
| 2 | That event is a **fail** verdict | `payload->>'verdict'` → `fail` |
| 3 | It is about a *different* dispatch | `payload->>'dispatch_id'` → `disp-history-sessions-verify` |
| 4 | Flux never verified the five | `ReviewVerdictIssued` naming any of their dispatches → **0** for each |
| 5 | A click on Accept would merge | `event_or_artifact_exists('ev-verdict-history-sessions')` → **t** |

**Defect 1 — the merge criterion does not read the value.**
`cmd_accept_delivery` called `event_or_artifact_exists`, whose entire body is
`SELECT EXISTS(canonical_events WHERE id=$1) OR EXISTS(artifacts WHERE id=$1)`.
The comment directly above it said *"the Flux **green** verdict must exist NOW"*.
The comment said green; the code never read green. Existence was being read as a
verdict.

**Defect 2 — opening a card does not check the subject.** `request_acceptance`
required only that `flux_verdict_ref` name an existing `ReviewVerdictIssued`, so
any verdict at all could stand in as the merge criterion — another unit of work's,
or a red one.

## 2. What was built

| File | What |
|---|---|
| `crates/seatloom-core/src/db/models.rs` | `VerdictFacts` + `is_merge_criterion()` |
| `crates/seatloom-core/src/db/repositories.rs` | `flux_verdict_facts(ref_id, dispatch_id)` — the one place the two questions are asked |
| `crates/seatloom-core/examples/verdict_facts_probe.rs` | new — drives the real read, so the proof measures live instead of asserting |
| `src-tauri/src/commands/acceptance_cmds.rs` | the merge criterion reads the value and the subject; four distinct refusals |
| `crates/seatloom-mcp-emit/src/dispatch.rs` | `request_acceptance`'s guard: exists ∧ is a verdict ∧ `pass` ∧ about this dispatch; two new typed rejections; 3 new DB tests |
| `crates/seatloom-mcp-emit/src/protocol.rs` | the tool's schema states the criterion |
| `crates/seatloom-mcp-emit/proof/acceptance_verdict_gate_proof.sh` | new — 6 points against live and a scratch DB |

## 3. The criterion, and why it is not a guess

**A verdict counts for a dispatch iff it NAMES that dispatch.** It names it in one
of two ways Flux actually writes:

| Channel | Measured on live |
|---|---|
| `payload->>'dispatch_id' == <dispatch_id>` | **27 of 30** `ReviewVerdictIssued` events. The 3 that do not are `ev-030`/`ev-031` (pre-012 seed rows about a workitem) and one scenario fixture |
| an `event_object_refs` row with `ref_type='dispatch'`, `ref_id=<dispatch_id>` | **5 of 30** |

Either channel counts, so a verdict written the other way is not refused for using
the wrong spelling. Neither channel is invented: both are things Flux already
produces, and the counts are measured rather than assumed.

**`lane` was rejected as a criterion.** It is free text — measured, a whole
sentence (`"Historical session corpus, slices 1+2 (Onyx 33f456b data, Nimbus
d3e53ea read path), judged against the live database per
AEGIS-2026-10-14-flux-live-shape-correction"`), not a key. Matching on it would be
a heuristic, and a heuristic is a guess wearing a criterion's clothes.

**Green means exactly `pass`.** Measured, the values in use on live are `pass`
(27), `fail` (2), `reject` (1). So a `reject` is not green either, and a verdict
that states no value is not a pass — an absence is not an affirmation.

## 4. The four refusals are four different messages

The messages are in the operator's language because the pane renders them to
him; this note is English per §10 and names them by their meaning.

| State | What the operator is told |
|---|---|
| The referent is gone | "the verification basis \<ref\> is not in the ledger — it may have been deleted, or was never recorded. With no verified-green verdict, it cannot be merged." |
| Not a verdict event | "…is not a verdict event (it is \<type\>) — only Flux's ReviewVerdictIssued can serve as the verification basis." |
| Red, or no value stated | "…the verdict's conclusion is \<value\>, not a pass. Flux judged it not-passing so it cannot be merged — have him re-verify, or deal with what he sent back." |
| Green, but about another dispatch | "…is not about this dispatch — it is about \<other\>. Another unit of work's verdict cannot stand in for this one's." |

They are separate because they call for different actions. Collapsing them into one
sentence — "not verified green, cannot merge" — is what left him with a single
opaque "operation failed", his words, from the dispatch.

## 5. 🔴 The line was not relaxed

- **No default.** A refusal is a refusal; no value is substituted anywhere.
- **The checks are at BOTH points** — card-open and decision-time — so a verdict
  re-issued, or a card re-opened, is re-checked. The dispatch calls these two the
  only mechanical barrier against unverified work reaching the main branch.
- **The normal path still works.** A third DB test asserts a green verdict about
  *this* dispatch still opens a card, so the guards cannot be satisfied by
  refusing everything.

## 6. The proof caught a design flaw in my own fix

I first wrote `VerdictFacts::is_green()`, answering only the value half. Measured
on live, a **green verdict about another dispatch read `green=true`** — the exact
half-criterion this dispatch exists to close, sitting one method call away. It is
now `is_merge_criterion()`, which ANDs in `names_dispatch`, and there is no
half-question left to ask by accident. The two call sites that need the halves
separately — to render two different refusals — read the fields directly, which is
honest about being partial.

The proof also caught two fixture problems in pre-existing tests, both fixed: 11
DB fixtures inserted `{"verdict":"pass"}` with **no `dispatch_id`** (the same
under-specification that let the production bug through), and three tests needed
their own verdict so their request reached the guard they exist for.

## 7. Measured harm — reported, not fixed

**Two of those five cards were ACCEPTED and merged on the fail verdict.**

| Card | Dispatch | Decided | Merge commit |
|---|---|---|---|
| `acc-disp-ol420Y586tNIsbbeh5nlU` | the app-icon work | 2026-10-16 06:54:27Z | `4cfd2302` |
| `acc-disp-history-sessions-ui` | the historical-sessions UI | 2026-10-16 06:54:29Z | `4cfd2302` |

Both in one operator action, two seconds apart, both merged into
`track/infra-foundation` in the same commit. Neither dispatch has a
`ReviewVerdictIssued` of its own — measured 0 for each.

**I did not revert anything and I am not claiming they were wrongly merged.**
Whether `4cfd2302` stands is Aegis's ruling: the work may have been verified
through a channel I have not measured, and un-merging is not mine to do. What I am
reporting is the mechanical fact — under the criterion this dispatch asked for,
neither card qualifies, and they landed anyway. That is the harm the two guards
exist to prevent, and it had already happened once before this fix.

## 8. Gates run

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -p seatloom-core --all-targets -- -D warnings` | clean |
| `cargo clippy -p seatloom-mcp-emit --all-targets -- -D warnings` | clean |
| `cargo test -p seatloom-core` | 174 passed / 0 failed (2 ignored, need Postgres) |
| `cargo test -p seatloom-mcp-emit` | 85 passed / 0 failed (22 ignored) |
| `cargo test -p seatloom-mcp-emit --bin seatloom-waker` | 9 passed / 0 failed |
| `cargo check -p seatloom-tauri` | clean |
| `cargo check --workspace --all-targets` | clean |
| DB tests on a scratch DB (canonical order + 015-019) | **22 passed / 0 failed** |
| `proof/acceptance_verdict_gate_proof.sh` | ALL POINTS PASS |
| the three earlier proofs | ALL POINTS PASS (re-run, unchanged) |
| `scripts/verify-docs.sh` | this note is clean |

The proof's six points: the live shape and harm; the old criterion's verdict; the
new criterion refusing all five cards; a green verdict being green **only** for its
own dispatch; all three required DB tests present and passing on a scratch DB; and
the read being read-only on live (ledger 1127 events / 2762 refs / 11 acceptances,
identical before and after).

## 9. Acceptance — what I measured, and what is yours to click

**Measured by me:** every case above, against the live database through the real
function, plus the three DB tests on a scratch database.

**Yours to click, which I cannot run and am not claiming:**

1. A card whose verdict is a **fail** → **Expected: refused, naming the verdict and
   its value.** (There are none pending on live right now — the three were
   cancelled — so this needs a fixture or a new card.)
2. A card referencing **another unit of work's** verdict, even a passing one →
   **Expected: refused, saying whose verdict it actually is.**
3. A card with the correct verdict, passing → **Expected: accepted and merged,
   exactly as before.** That is the no-regression criterion.

**What this build will not show:** the running app is the previously installed
binary. It must be rebuilt and reinstalled
(`scripts/prepare-verification-build.sh`) to carry this change, and **the card and
the build must arrive together.** I did not rebuild, reinstall or restart
anything.

## 10. Known limits, stated rather than hidden

- **A verdict that names the dispatch only in prose is refused.** If Flux ever
  writes a verdict with no `dispatch_id` and no dispatch ref — as the three
  pre-012 seed rows do — it cannot be a dispatch's merge criterion. That is the
  intended strictness, and it is why the schema documents it.
- **`reject` is treated as not green**, along with `fail` and an absent value. If
  the vocabulary gains a third passing word, `is_merge_criterion` is the one place
  to change.
- **The artifact branch of the polymorphic ref still exists** and an artifact
  verdict cannot state a value or a subject, so it can never satisfy the criterion.
  Measured: **0 of 11** cards on live reference an artifact, so this is a
  documented dead branch rather than a live path.
- **The two accepted cards in §7 are untouched.** Reverting them is not mine.

## 11. Out of scope, unchanged

The merge logic itself, the dispatch state machine, the live database, and the
acceptance card's state machine. The only behavioural changes are where the
operator identity's verdict comes from and what the refusal says.

## 12. Forward recording (D5 Phase 2 Lane B)

Two memories, both `source_kind=canonical_event`, `source_ref` =
`ev-nimbus-unverified-merges-on-track`, both `confidence=quoted`, in topic
`topic-acceptance-verdict-criterion` (opened for this):

| Memory | Kind | Why it passes the three-way test |
|---|---|---|
| `mem-fLY1IufMt0G5N91D5Q-Zr` | decision | The four-part merge criterion, and the two lessons: a comment promising green above code checking only existence is how it survived, and a half-criterion must not be a method. Changes what a future seat does when writing a gate; exists nowhere else; evidenced by the five live cards and the measured `green=true` on a foreign verdict. |

Nothing else was recorded. The `reject`-is-not-green ruling and the artifact branch
being a measured dead end are consequences of the one decision, and no preference
was touched — Mr Zhang's corrections are Aegis's to record.

**Reported to Aegis, outside this dispatch's boundary:** the two unverified merges
in §7, as `ev-nimbus-unverified-merges-on-track` (a `SupervisorMessage` naming the
cards, the verdict, the merge commit, what I did, what I did not do, and the one
question for him: does `4cfd2302` stand). Nothing was reverted and the live
database was not touched.
