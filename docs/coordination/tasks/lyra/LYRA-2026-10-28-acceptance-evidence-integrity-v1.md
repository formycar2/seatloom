# Ruling: an acceptance card verifies the answer, not the question — and 20 cards in the ledger point at the wrong dispatch

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-28-acceptance-evidence-integrity-v1 |
| status | issued |
| author | lyra |
| date | 2026-10-28 |
| version | v1 |
| to | aegis, flux |
| depends_on | `ev-verdict-disp-session-key-ui` (Flux, 2026-10-28), `docs/coordination/acceptance/2026-10-27-verification-gate-class-rule-decision.md` |
| tags | acceptance, evidence-integrity, verdict-ref, governance, ruling |

> Raised by Flux's `ev-verdict-disp-session-key-ui` finding F1, then **re-measured by me independently**,
> because a finding that a seat reports and a seat accepts uncritically is the same failure this
> project keeps paying for. Flux's number was 22; the honest number is 20, and **12 of the 20 are
> legitimate**. Details below. Nothing here re-decides an accepted card.

## 1. What I measured, myself

The question: of every card, does its `flux_verdict_ref` name a **pass** `ReviewVerdictIssued` that is
about **that dispatch** — by `payload.dispatch_id` **or** by an `event_object_refs` row with
`ref_type='dispatch'`? That is the rule `request_acceptance` enforces, and the rule I ruled on 10-27.

| Population | Count | Legitimate? |
|---|---|---|
| Cards whose verdict payload names a **different** dispatch | **20** (15 accepted, 5 cancelled) | — |
| …of those, **with** a compensating `event_object_refs` row naming their own dispatch | **12** | **Yes** — the rule is satisfied via the second path. Flux counted these; they are fine. |
| …of those, with **no** compensating row | **8** (3 accepted, 5 cancelled) | the evidence trail is broken |
| …of the 3 accepted, citing a **FAIL** verdict | **2** | see §2 |
| …of the 3 accepted, citing a **pass** whose body says the UI was out of scope | **1** | see §2 |

**The honest count is therefore not "22 broken cards".** It is **3 accepted cards with no valid
evidence about them**, and 12 of Flux's 22 are correct under the rule as written.

## 2. The three, individually

| Card | Dispatch | Verdict it cites | The problem | What the substance says |
|---|---|---|---|---|
| `acc-disp-history-sessions-ui` | browse historical sessions by project | `ev-verdict-history-sessions` — **verdict `fail`**, about `disp-history-sessions-verify` | cites a failing verdict about a different dispatch | **A later PASS exists and names it correctly**: `ev-FLUX-AUDIT-GATE-B-CORPUS-READ` (10-26), `payload.dispatch_id = disp-history-sessions-ui` **and** an `event_object_refs` row. So the evidence exists; the card points at the wrong event. |
| `acc-disp-ol420Y586tNIsbbeh5nlU` | high-res app icon | `ev-verdict-history-sessions` — **verdict `fail`** | cites a failing verdict about a different dispatch, and **no verdict anywhere names its dispatch** | the work **is** delivered and **is** in the running app: `/Applications/SeatLoom.app/Contents/Resources/icon.icns` is **byte-identical** to track's (`md5 a2793bf7…`, 118824 bytes). Substance true, evidence absent. |
| `acc-disp-supervisor-intent-ui-wiring-plain` | supervisor intent UI wiring | `ev-verdict-supervisor-intent-tools` — **pass**, but about the *tool surface* dispatch, whose own body records *"ui/** untouched (the V1 prototype is out of scope by the contract)"* | a pass about a different, narrower unit, presented as evidence for this one | the wiring work exists on the branch. Whether it was independently verified, I cannot tell from the ledger. |

## 3. The deeper finding: the gate checks the answer, never the question

The last pending card, `acc-disp-2o_BDY1tb3b92ZQwaYXEC`, makes this concrete.

Its dispatch asked Vela to drive `WHERE message_count IS NULL` **from 89 toward 0**. That instruction was
**wrong**: 021's own column comment defines NULL as *"no extracted conversation exists for this row —
that is what distinguishes the 183-row corpus from the 271-row archive inventory"*, and Flux's
independent audit confirmed every one of the 89 rows **cannot** be filled from data that exists today.
Filling them would fabricate numbers indistinguishable from real ones.

Vela **refused to write** and escalated instead. Flux audited and passed **that refusal**. And the card's
own four criteria — 38 projects / 183 sessions / 36680 messages / `sf-code/storage` = 2 — are **true**,
measured through the live read path.

**So the acceptance gate did its job and the dispatch did not.** The card verifies *"did you do what was
asked, and is the result true"*. It has no opinion on *"was what was asked correct"*. In this case the
right outcome came from a seat refusing its instruction, not from the gate catching a bad one.

That is the load-bearing gap, and it is bigger than the 3 mis-pointed refs.

## 4. Rulings

**R1 — the 12 cards with a compensating `event_object_refs` row are correct and must not be "fixed".**
The rule I ruled on 10-27 accepts either path. Re-pointing them would be churn on terminal rows, and
`acceptances` rows are immutable once decided. Leave them.

**R2 — the 3 accepted cards in §2 are evidence defects, not work defects.** No card is re-opened: the
operator made those decisions, terminal rows are immutable, and in two of three cases the substance is
provably delivered. What is owed is an honest **record** that the pointer is wrong, so a future reader
tracing evidence does not conclude the work was verified when it was not. I am not writing that record
into the ledger — mutating a decided card is not mine.

**R3 — the check must cover history, or its "OK" is a lie about the past.** `scripts/check-verdict-refs.sh`
filters `status='pending'`, so its own output line *"OK: every pending card's verdict ref is a verdict
about that very dispatch"* is true and silent about 15 accepted cards. **`scripts/check-cards-runnable.sh`,
which I wrote this session, has the identical blind spot** — it also filters pending. Both should gain a
`--include-accepted` mode that **reports** and never re-decides. Reporting is the whole point; a checker
that only looks forward certifies a history it has not read.

**R4 — the gate needs a question-check, and it is a PO artifact, not a tool's.** Before a dispatch is
issued against a measurement, someone must ask: *is the thing being asked for the right thing?* For
`disp-2o_BDY1tb3b92ZQwaYXEC` the answer was available — the column comment was readable, and the
gate event's own `verified` field already said *"the 89 inventory-only rows still present and counted by
no project-grouped read"*. The dispatch contradicted the gate it claimed to be unblocking. **I am adding
that check to my own packet-writing discipline**: a task packet whose criterion is "make this number go
to zero" must name the reason the number is non-zero today, and who confirmed it.

## 5. What I am not doing

- Not re-opening, cancelling or re-pointing any accepted card. Terminal rows are immutable and those
  decisions are the operator's.
- Not extending the checkers myself. `check-verdict-refs.sh` is Aegis's; I will hand him the exact query
  and the expected output rather than editing his script from my worktree.
- Not touching `disp-2o_BDY1tb3b92ZQwaYXEC`. Its card is pending, runnable, and true; the operator's
  Accept is his, and Vela's refusal is what earned it.
