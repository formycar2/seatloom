# Acceptance decision: verification gate class — who may open which acceptance card

| Field | Value |
|---|---|
| template | T5 |
| subtype | gate_decision |
| id | LYRA-2026-10-27-verification-gate-class-rule-v1 |
| status | issued |
| author | lyra |
| date | 2026-10-27 |
| version | v1 |
| target | `Acceptance gate class` |
| verdict | GO |
| depends_on | `docs/coordination/tasks/aegis/AEGIS-2026-10-27-flux-self-verification-gap-v1.md` |
| tags | acceptance, verification, gate-class, flux, governance, decision |

## 1. The decision

`request_acceptance`'s guard — `flux_verdict_ref` must name a `ReviewVerdictIssued`
whose `payload.dispatch_id` names the dispatch being accepted — is correct and
stays. What was missing is a rule for the seat that cannot be independently
verified by the mechanism it serves.

The decision, written into `docs/coordination/ENGINEERING_STANDARDS.md` §
`Acceptance gate class` (v2):

1. **A seat never issues the verdict for its own dispatch.** Self-verification is
   not verification; this guard is unchanged.
2. **A dispatch's acceptance class is declared at issue time in its payload**
   (`acceptance_class`: `implementation` | `verification` | `review`) and is a
   property of the work's content, not of which seat ran it.
3. **`implementation` (any diff) opens a card, and the card needs an independent
   GREEN verdict from a seat other than the actor**, plus the operator running
   the PO-authored plan.
4. **`verification` (reads state, issues a verdict, commits no diff) opens a card
   too** — it is a delivery like any other, and it needs its own verdict.
5. **`review` (investigates, reports, commits no diff, opens no card) opens no
   acceptance card**; its durable artifact is the work product. It is auditable
   and may be re-checked, not exempt.
6. **There is no `verdict_exempt`.** No dispatch skips the verdict by declaring
   itself exempt.

Option A (cross-verification) is kept only as the mechanism for `implementation`
work by a verifier seat — it is not a new recursion, because the cross-checker is
not verifying its own output. Option B (Aegis re-checks every verdict) is
rejected: it makes the supervisor a serialization point on the one lane that is
supposed to stay independent. Option C is rejected in its blanket form (a
maintained exempt list is a whitelist whose boundary moves) and adopted in its
narrow form: read-only reporting with no diff opens no card.

## 2. Why this and not the alternatives

| Option | Verdict | Reason |
|---|---|---|
| A. Cross-verify Flux's work | Adopted narrowly | Correct for `implementation` work; does not recurse, because the verifier of the verifier is checking someone else's output |
| B. Aegis re-checks every verdict | Rejected | One bottleneck lane; Aegis would be in the audited set; latency on every card |
| C. Blanket exempt list | Rejected | A whitelist is an invitation to move its boundary; "exempt" hides rather than states |
| C'. Read-only-with-no-diff opens no card | Adopted | Defines the exemption by content (no diff ⇒ no merge candidate), so it cannot be stretched to cover code |
| D. Let Flux self-issue a pass | Rejected | This is the hole being closed; an independent guard reduced to one tool call |

## 3. Applying it to the eight blocked dispatches

Aegis asked for a rule and will then release the eight dispatches. The rule is
per-dispatch, so here is the mapping — the class each one takes, and what it
still needs. This is the classification the gate will run on.

| Dispatch | What it is | Class | Opens a card? | What it still needs |
|---|---|---|---|---|
| `disp-viPlKa_e9NoeRHeGe_LCC` | verifies Mira's F2 fix (code changed) | `verification` | Yes | a verdict about this dispatch from a seat other than Flux |
| `disp-yqYB1iZlPYe7HceKux71D` | verifies never-answered shape (product code) | `verification` | Yes | same |
| `disp-xyvPfhtGxQo5x_2u4VGPU` | verifies supervisor-sync-lost (product code) | `verification` | Yes | same |
| `disp-IgwyJqKM_ef9T3j63sD-i` | re-verifies `disp-DFeH6Ut_aNfzIbuuR21V5` after a fail | `verification` | Yes | same; its verdict must name `disp-DFeH6Ut_aNfzIbuuR21V5` as the verified unit |
| `disp-i_ZLKc4E2S0Bm78K-Zmdy` | audits two merged Nimbus fixes | `verification` | Yes | same |
| `disp-VH6l1HixlUVYuy4m0NEp-` | audits three gates (read-only, no diff) | `verification` | Yes | same |
| `disp-4zePyIT3GboLBELuobHAS` | corrects a verdict that named no dispatch | `verification` | Yes | the corrected verdict event is its own evidence |
| `disp-E57bNvMK5uTMgrH7tg2cK` | verifies Mira's shift fix | `verification` | Yes | its verdict must name `disp-fCOD_8QOby_Bi20NfrsyR` as the verified unit |

Note what this table does **not** do: it does not issue the eight verdicts, and
it does not declare any of the eight passed. Each still needs a real verdict from
a real independent seat. The rule removes the structural impossibility; the
evidence is still owed per dispatch.

Contrast with a `review` dispatch such as `disp-2o_BDY1tb3b92ZQwaYXEC` (Vela's
89-row NULL investigation): read-only, no diff, no merge candidate — no card. Its
artifact is the delivery.

## 4. What this changes for the seats

- **Aegis (issuer)**: write `acceptance_class` into every dispatch payload. An
  acceptance request for a dispatch with no class is refused once the code
  enforcement lands (Follow-up 6a).
- **Flux**: keep issuing verdicts. A verdict whose `payload.dispatch_id` names a
  dispatch owned by another seat is the required evidence; a verdict naming
  Flux's own dispatch is not.
- **Worker seats**: nothing new — you were already covered by `implementation`.
- **Whoever verifies Flux's verifications**: any worker seat that did not perform
  the work. This is a routing decision for Aegis, not a new standing role.

## 5. Enforcement status

The rule is prose today. Code enforcement is Follow-up 6 in
`ENGINEERING_STANDARDS.md`: reject classless acceptance requests, forbid a
verdict naming its issuer's own dispatch, and add `engineering_standards` to the
T7 subtype allow-list. Until that lands, the gate depends on seats honouring the
declared class, and an operator can still see a card that should not exist.

## 6. Known limits of this decision

- The class is self-declared by the issuer. An issuer can mis-declare a
  code-changing dispatch as `review` and thereby avoid the card. Detection is
  manual today (the commit list is right there); a mechanical check is Follow-up
  6a/6c.
- "Independent" means *a different seat*, not *a different human*. Two dispatches
  issued from one operator session could still be self-consistent. That is a
  deeper property of this fleet and is not solved by this rule.
- The rule does not decide who verifies Flux's verifications. It says who may;
  routing is Aegis's.
