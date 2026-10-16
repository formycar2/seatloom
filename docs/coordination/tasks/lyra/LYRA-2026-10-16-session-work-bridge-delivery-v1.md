# Delivery — the session-to-work bridge contract, its five slices, and the finding that reshaped it

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-16-session-work-bridge-delivery-v1 |
| status | delivered |
| author | lyra |
| date | 2026-10-16 |
| version | v1 |
| depends_on | LYRA-2026-10-16-session-work-bridge-contract-v1.md, docs/coordination/acceptance/2026-10-16-session-work-bridge-acceptance-v1.zhs.md |
| tags | delivery, lyra, session-work-bridge, contract, decomposition |

## What was delivered

`docs/coordination/tasks/lyra/LYRA-2026-10-16-session-work-bridge-contract-v1.md`
(`packet/lyra/milestone-summary` @ `8601167`), the operator's acceptance card at
`docs/coordination/acceptance/2026-10-16-session-work-bridge-acceptance-v1.zhs.md`, five
dispatches enqueued into the governed graph, and one ledger event.

| PRD hard rule | Where it is satisfied |
|---|---|
| A user story | Contract §6 — who, what he wants, how he is stuck today |
| An interaction flow | §7, including §7.2's every empty and missing state with its reason |
| A screen / UI surface definition | §8 — one added section inside the existing session detail |
| Pain / omission cost / behaviour change | §1's three consequences and §4.2's four repair requirements |
| Out of scope, with reasons | §10 |

## The judgment call, answered with a measurement

Aegis asked whether this key should be produced by the washing pipeline, and told me not to
write the contract along his assumption. The measurement says the question is already
answered in fact: **the washing pipeline has been writing this key since 2026-10-11**, and
migration 021 was written *specifically to make it resolve*.

| Measurement | Value |
|---|---|
| Live memories total | 151 |
| Carrying `source_kind='archived_session'` | **90** |
| Distinct sessions covered | **10** (b0100cff 30, adb08b3c 15, 01565715 12, 5c09cdda 9, ca5f59be 8, 621ae382 7, ccec8b39 3, 019dda18 3, 019dce05 2, 019ddc3e 1) |
| Earliest event in the whole ledger | `2026-10-09T16:53:28 SeatDispatchIssued` |
| Events carrying a `session` ref | **0** |

So the contract does not design a key. It pins the two that exist, makes the drift
measurable, and declares the third of the question empty.

## The three candidate keys, ranked by what breaks them

**A1 (primary) — the declared path pair.** `memories.l2_file_path` with the corpus-root
prefix stripped, against `archived_sessions.raw_path`. Both sides *declare* the path;
neither derives it. `s5_writein.py`'s own comment records why it normalises rather than
leaving the bare relative form — *"a bare `raw/...` path would look repo-relative while
resolving to nothing here, i.e. an unresolvable pointer — which red line 2 forbids"* — so
the write-in and the migration were both written against the same declared path. Decisive
property: **it is immune to id re-keying.** `021` §3 already uses `(harness, raw_path)` as
its own match key and says why: *"stable across the re-key"*.

**A2 (secondary) — `source_ref` split on the first colon, against
`archived_sessions (harness, id)`.** Exact, and already written by 90 memories — but free
text with no constraint, and **it already broke once**: before 021, 3 of 6 codex
references resolved against no row at all, and nothing in the product reported it. Those
sessions looked like they had produced nothing.

**A3 (never a key) — the topic id.** `wash-sess-cc-621ae382` is `sid[:8]` formatted into a
string by `tools/wash/s5_writein.py:296-309`. A truncation plus a segmentation label is a
guess, and a topic is a revisable label by design (`2026-09-30-aegis-object-model…` §3.4).

## The red line that matters most

The ledger's earliest event is 2026-10-09 and the corpus's latest session is 2026-10-08, so
**every historical session's ledger half is empty by construction** — not because a link is
missing, but because nothing was recording then. The contract forbids filling it: no
backfill, no re-extraction into `canonical_events`. A fabricated event is worse than a
missing one in the one product whose thesis is *"the record, not the harness, holds the
truth."* The product must say *why* there is nothing, and stop talking.

## The dispatch graph, as it stands now

| Dispatch | Seat | Live status | Edges |
|---|---|---|---|
| `disp-session-key-view` | Onyx | **ready** — released by the contract event | `ev-session-work-bridge-contract-landed` (exists) |
| `disp-session-key-readmodel` | Nimbus | blocked | `ev-session-key-view-landed` (absent) |
| `disp-session-key-writein-guard` | Vela | **in_flight** — pulled within seconds, no edges | none |
| `disp-session-key-ui` | Mira | blocked | `ev-session-key-readmodel-live` (absent) **and** `ev-history-sessions-read-live` (exists) |
| `disp-session-key-verify` | Flux | blocked | `ev-session-key-readmodel-landed` (absent); pre-declared verdict `ev-verdict-session-key` |

Verified with `get_dispatch` after emitting the contract event: the slice whose edge now
exists flipped to `ready` on its own, and Vela's edgeless slice went `in_flight`
unprompted. The three waiting on absent referents are `blocked`, not errored.

Slice 4 carries two edges on purpose: it *calls* the read model, so it needs the measured
`-live` gate rather than the announcement — the same rule Mira's own corpus slice 3 was
issued under — and it needs the corpus read gate because its screen does not exist without
the corpus screens.

## One structural change worth noting

The operator's acceptance card is now a **separate artifact**
(`docs/coordination/acceptance/2026-10-16-session-work-bridge-acceptance-v1.zhs.md`) rather
than a section inside the design contract. Two reasons: a card buried in a contract is a
card nobody runs, and the doc gate exempts `*.zhs.md` from its English-only rule, which is
what that suffix exists for. Both new documents pass `scripts/verify-docs.sh` born-clean.

## The debt I found in my own work, and did not sweep

Running the gate on this branch shows **85 red documents**, and a large share are mine. The
systematic cause is not carelessness about content — it is that I have been writing YAML
frontmatter (`template: T5`) where `DOCUMENT_TEMPLATES` §2 and `verify-docs.sh` require a
markdown **table** (`| template | T5 |`), and using T5 for documents that live under
`docs/coordination/tasks/` when T5 must live under `docs/coordination/acceptance/`. The
roadmap already tracks this class of tax as **DV3** ("Doc-header conformance at source"),
and records it as a *recurring* one — every reconcile, Aegis hand-fixes seat docs missing
the `version` header. My branch sharpens that finding: the tax is the whole header *form*,
not one field, and it is large enough that a hand-fix per reconcile cannot keep up.

I fixed the two documents I authored in this dispatch and left the rest, because rewriting
~25 documents' headers is a mechanical sweep that deserves its own commit and its own
review, not a rider on a planning delivery. Recommend it as a follow-up: the transform is
scriptable (YAML key/value → table row), and fixing a grandfathered doc removes its line
from `.gitlab/docs-grandfather.txt`, so the ratchet only tightens.

## Blockers

- None for this turn. Two slices are moving; three are correctly blocked on referents that
  do not exist yet.
- No human decision is blocking. The one thing I flagged last time and am re-flagging
  because it is now sharper: the ⌘K double-binding is still unresolved with Aegis, and it is
  unrelated to this contract's slices.

## Ledger

- `ev-session-work-bridge-contract-landed` — `ArtifactCreated`, the contract announcement.
  Carries `branch`/`commit`/`files`, the acceptance-card path, the judgment call, the three
  keys ranked, the five measurements, the five slices, and an explicit `does_not_assert`
  that says the join is **unmeasured** since 021's re-key. No `verified` field — an
  announcement, not a gate.
- The five `SeatDispatchIssued` events are written by `enqueue_dispatch` itself, in the same
  transaction as the rows.
