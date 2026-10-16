# Decision — the acceptance card must say which verdict it cites, and the merge gate must read it

| Field | Value |
|-------|-------|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-16-acceptance-card-verdict-contract-v1 |
| status | issued |
| author | lyra |
| date | 2026-10-16 |
| version | v1 |
| to | aegis, nimbus, mira |
| depends_on | LYRA-2026-10-16-acceptance-build-attestation-decision-v1.md, NIMBUS-2026-10-16-acceptance-verdict-criterion-v1, FLUX-2026-10-16-mira-five-slices-verdict-v1 |
| tags | decision, acceptance-gate, product-truth, heartbeat, contract |

Raised by the LYRA heartbeat protocol (§3.1 and §3.2), round 2, no dispatch waiting. Facts,
findings and decisions are in separate sections. Every fact below was measured by me on
`track/infra-foundation` @ its current tip, or read from the live ledger; nothing is taken on
trust from another seat's report.

---

## 1. Facts

**F1 — The merge guard the operator's Accept button runs is existence-only, and its own error
message promises something it does not check.** `src-tauri/src/commands/acceptance_cmds.rs`,
`cmd_accept_delivery`, on `track/infra-foundation`:

```rust
// Merge criterion, first half: the Flux green verdict must exist NOW. …
let verified = state.db.event_or_artifact_exists(&row.acceptance.flux_verdict_ref).await?;
if !verified {
    return Err(format!("<cannot-merge-without-a-green-verdict>: flux_verdict_ref {:?} names no existing event or artifact", …));
}
```

The comment says *green verdict*. The code asks *is the referent there*. The refusal message says
*"cannot merge without a green verdict"* — for a check that never reads the verdict's
value. **The product states a guarantee in its own words that its code does not implement.**

**F2 — It already happened, twice, and the operator clicked.** Two cards were created citing
`ev-verdict-history-sessions`, which is `verdict = "fail"` and `payload.dispatch_id =
"disp-history-sessions-verify"` — a different dispatch from either card's:

| Card | Requested | Cited verdict | Granted | Merged in |
|---|---|---|---|---|
| `acc-disp-history-sessions-ui` | 10-16 05:53 | `ev-verdict-history-sessions` (**fail**, about `disp-history-sessions-verify`) | 06:54:29 | `4cfd2302` |
| `acc-disp-ol420Y586tNIsbbeh5nlU` (app icon) | 10-16 06:45 | same | 06:54:27 | `4cfd2302` |

Two seconds apart, one operator action. A third card (`acc-disp-cEyS0pTJRZaC3b64vFUdR`, requested
07:23) cited the same fail verdict and was withdrawn at 07:49 before a decision.

**F3 — The card showed him a red badge, and he accepted anyway.** `ui/src/lib/acceptance.ts:154`,
`fluxVerdictStyle("fail")` returns a rose-styled label reading **Flux-verified-red**. So both cards
rendered a red verdict badge. **He was not shown a green light.** I am not claiming he was
misled, and I have no evidence he did not know what he was doing — the 10-14 fail verdict's own
text says the data migration itself is correct and lists a fix path that was completed before he
clicked. His override may well have been informed.

**F4 — But the card could not have told him the one thing that would have made it informed.**
`src-tauri/src/dto.rs:362`, `FluxVerdictDto`:

```rust
pub struct FluxVerdictDto {
    pub ref_id: String,
    pub verdict: Option<String>,
    pub occurred_at: Option<String>,
}
```

**No subject.** The DTO carries the verdict's id, its value and its time — and nothing about
*which dispatch it was about*. So the card renders a red badge, an opaque event id
(`ev-verdict-history-sessions`) and a timestamp, and the operator has no way to tell that the
verdict concerned the **data** slice, whose defects had been fixed, rather than the **UI** slice he
was accepting. The information is not in the DTO, so the UI cannot show it even if it wanted to.

**F5 — Accept records nothing; reject requires a written reason.** `cmd_accept_delivery` takes
`acceptance_id` and nothing else. The UI's reject button is `disabled={busy !== null || !trimmedFeedback}`
with the tooltip that a reject needs one sentence of reason so the seat can redo the work, and a
required textarea. The accept button is
`disabled={busy !== null}`. So the ledger holds a bare `AcceptanceGranted` for two red-verdict cards,
and nothing distinguishes "he knew the fail was stale" from "he clicked through".

**F6 — The fix is written, correct, and not integrated.** On
`packet/nimbus/gate-vs-announcement`, `cmd_accept_delivery` calls
`state.db.flux_verdict_facts(&row.acceptance.flux_verdict_ref, &row.acceptance.dispatch_id)` and
refuses via `verdict_gate_message`, with a comment that names the exact measured harm: *"three
pending cards named `ev-verdict-history-sessions`: a fail verdict about
`disp-history-sessions-verify`, for three dispatches Flux had never verified. Existence was being
read as a verdict, and a red verdict about someone else's work would have merged on the operator's
click."* That branch is not merged. `track/infra-foundation` still carries F1.

**F7 — The two merges' work is genuinely verified, but only after the fact.** Flux's
`ev-0idwakl17abEU75LNED24` (10-16 08:31, verdict `pass`) covers Mira's five 2026-10-16 slices, and
its re-issue `ev-NFPr-zN36by-i9SE-0FPH` (08:46) names all five dispatches by
`event_object_refs` rows, explicitly listing `disp-history-sessions-ui` (51bccc5) and
`disp-ol420Y586tNIsbbeh5nlU` (d04f373) as PASS and noting both merged at `4cfd230`. The re-issue
states plainly that it is *not* a second verification and nothing was re-measured. So the merge
preceded its green verdict by **1 hour 37 minutes**, and the record is coherent only because Flux
and Aegis repaired it afterwards.

## 2. Findings (inference, labelled)

**2.1 The harm was real even though the outcome was fine.** The gate failed *open*. Had the 08:31
verdict come back fail, two merges would already be on `track/infra-foundation` with no green
verdict behind them, and a revert would have been the only remedy. Nothing in the system would have
flagged it. "It turned out all right" is not a control.

**2.2 This is the fourth instance today of the same shape, and the first one that reached the
operator's click.** Earlier today: the acceptance card checked that a build was *prepared* rather
than that it is the one *running*; a `-live` gate checked that a capability *runs somewhere* rather
than that it is *integrated*; `prepare-verification-build.sh` step 4 grepped a compressed binary.
Now: the merge guard checks that a verdict *exists* rather than what it *says* or *about whom*. Each
check was written by someone who understood the risk. Each passes while the thing it stands for is
absent. Three of the four are now fixed or fixed-in-branch; the fourth is the one the operator's
finger is on.

**2.3 The deeper product defect is not the gate, it is the card.** Even with F6 integrated, a card
whose cited verdict is red is still *offered* with an enabled merge button, and the operator still
cannot see which dispatch that verdict was about. F6 makes the merge *refuse*; it does not make the
card *explain*. A refusal message is a worse experience than a card that never offered the button,
and Aegis has already said as much about a sibling error string: that the message he saw was a dead
end for someone who cannot read code, and that the operator has already pushed back on exactly that
(he clicked merge and got one line saying the operation failed).

## 3. Decisions (mine, as PO)

**D1. The next round's milestone: before clicking Accept-and-merge, you can see whether the verdict
this card cites is about this work, and what it said.** The card tells you which dispatch the verdict
was about, what it concluded, and when — and if it is not about this work, the card says so instead
of offering the button. User-perceivable by construction: it is on the surface he decides on.

**D2. The card carries the verdict's subject, and it is a DTO contract, so it is mine to write.**
`FluxVerdictDto` gains the verdict's subject dispatch id (from `payload->>'dispatch_id'` or an
`event_object_refs` row with `ref_type='dispatch'`), the verdict's evidence reference, and a
one-line summary of what it found. Without the subject in the DTO, no UI can be honest about this,
so this is the load-bearing half.

**D3. The Accept button is gated on the verdict being about this work and green.** Not a refusal
after the click — the button is not offered, and the card states why in plain language. This
mirrors what the merge guard will do, moved earlier, where he can read it.

**D4. Accepting over a red verdict records a reason; accepting a green one does not.** Reject
already requires one. An override of a visible failure is a decision worth recording, and six weeks
later "he knew" and "he clicked through" must not look identical in the ledger. The *policy* is
mine; the signature change is Nimbus's.

**D5. What I am not deciding.** Whether `4cfd2302` stands — Aegis's ruling, and my reading is that
it does, because F7 shows the work is verified, but the inversion is his to accept or revert.
Whether to integrate `packet/nimbus/gate-vs-announcement` — Aegis's action. I am not reverting
anything and not touching the live database.

## 4. The acceptance criteria

The operator-facing card is `docs/coordination/acceptance/2026-10-16-acceptance-card-verdict-zhs.md`.
In English, for whoever implements it:

| # | Step | What passes | What gets rejected |
|---|---|---|---|
| 1 | Open a pending card whose verdict is green **and** about this work | the card names the dispatch the verdict was about, what it concluded, and when — in words, not an event id | a bare event id; a badge with no subject; a "verified" label with nothing saying what was checked |
| 2 | Open a card whose verdict is about a **different** dispatch | the card says so plainly, names which dispatch the verdict *was* about, and does not offer the merge button | an enabled merge button; a refusal only after clicking; an error string in code |
| 3 | Open a card whose verdict is red | the card shows the red verdict, says what it found, and requires a written reason before Accept is offered | Accept offered with no reason; the red badge as the only signal |
| 4 | Accept a green card | no reason required, the decision recorded | a mandatory reason on the happy path — that trains people to type noise |
| 5 | Read the ledger afterwards | a grant over a red verdict carries its reason; a grant on green does not need one | two grants that look identical when one was an informed override |

## 5. Out of scope

Reverting or ruling on `4cfd2302` (Aegis's). Integrating Nimbus's branch (Aegis's action). Changing
`cmd_accept_delivery`'s signature or `dto.rs` (Nimbus's). Any UI re-layout (forbidden by the
operator). Touching the live database.
