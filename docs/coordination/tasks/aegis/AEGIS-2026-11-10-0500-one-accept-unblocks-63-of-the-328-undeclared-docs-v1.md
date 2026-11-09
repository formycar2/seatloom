| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-10-0500-one-accept-unblocks-63-of-the-328-undeclared-docs |
| status | delivered |
| author | aegis |
| date | 2026-11-10 |
| version | v1 |
| tags | aegis, docs, measurement, backlog, per-rule, acceptance-leverage |

# One Accept unblocks 63 of the 328 undeclared docs, and I nearly reported the opposite

## What I set out to check

The standing backlog says there are 328 undeclared nonconforming docs, classified on 2026-11-08 as 63
knowledge-convention / 221 prose / 28 wrong-key. That classification is two days old and I have learned
twice this week that a list nobody re-measures becomes a false report. So I re-measured.

## The re-measurement

| | measured now |
|---|---|
| docs failing `--audit` | **535** |
| already declared as debt | **207** |
| **undeclared** | **328** — the backlog's number holds |

By directory: mira 85, aegis 73, knowledge 63, flux 46, nimbus 29, onyx 20, lyra 4, acceptance 4.

**So the 328 is still accurate.** But one detail inside it is stale in a way that matters.

## The mistake I nearly made

The doc-gate exemption for the knowledge convention is on `packet/vela/washing-pipeline` — `KNOWLEDGE_DIR`
appears 3 times there and **0 times on track**. It is one of the three cards waiting for you. So I assumed
accepting it clears 63 failures, and set out to confirm.

I applied Vela's gate to a fixture of the 90 knowledge files and got **63 failures before, 63 after**. On
that measurement the exemption does nothing, and I was one step from reporting "the pending card does not
reduce the count".

**The measurement was wrong because it counted files, not rules.** Measured per rule:

| | header failures | CJK failures |
|---|---|---|
| track (no exemption) | **63** | **0** |
| with Vela's exemption | **37** | **26** |

The exemption moves 26 files from the header rule to the CJK rule. It does not make them pass — it makes
them **reach a rule they were previously short-circuited past**, because `check_file` returns early on a
header failure. So the total stays 63 and the composition changes.

That is exactly what Flux's verdict said, in the payload I had already read: *"the gate has not gone quiet —
26 files moved from one rule to another, they did not become invisible."* I had the correct answer
available and re-derived a wrong one by measuring the aggregate instead of the breakdown.

## What this actually means for the decision in front of you

The honest statement is narrower than "accepting clears 63":

- **26 files** stop failing the header rule and start failing the CJK rule — a real improvement, and an
  honest one, because the CJK failure was already there and was being masked
- **37 files** still fail the header rule for reasons the exemption deliberately does not cover (26 of
  them carry Chinese, 11 are genuinely missing frontmatter keys)
- the **remaining 265** undeclared docs are untouched by this card

So one Accept reduces the undeclared count by **zero** and moves 26 files from a masked failure to a
visible one. **That is worth doing and it is not worth overselling.**

## The backlog item I can now close

The backlog's item 4 says "26 Mira heartbeat docs lack a header, whole-repo state is red" and records 26 as
the number. Measured now: **114** docs under `tasks/mira/` lack a header, of which **33** are declared and
**81 are declared nowhere**. The 26 was the 2026-11-02 figure. Mira has kept working since.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0, plus `check-unclaimed-work` at 0.
- Three pending cards awaiting you.
- Push: local and remote both at `7e162df`, confirmed. No new commits this hour.
- DB reliability: 10/10 this hour.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **Three cards are waiting.** For Vela's specifically: accepting it moves 26 files from a masked failure to
  a visible one and reduces the undeclared count by zero. That is the honest framing.
- **Option 1, 2 or 3 for the live-seats count** — my recommendation is 1, unchanged.
- **The sandbox decision unblocks five dispatches at once**, not one.
- **`documents` has zero rows**, so `l2_ref_kind=document` can never succeed.
