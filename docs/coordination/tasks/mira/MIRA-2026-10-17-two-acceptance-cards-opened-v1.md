# Delivery: two acceptance cards opened, with the live blockers written in rather than smoothed over

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-17-two-acceptance-cards-opened-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-17 |
| version | v1 |
| dispatch | disp-AAJ7X_ohCWhlM960Z4xxS |
| depends_on | MIRA-2026-10-16-projects-page-hydration-delivery-v1, MIRA-2026-10-16-seat-attribution-by-binding-delivery-v1 |
| tags | acceptance-cards, live-blockers, delivery, mira |

---

## 0. The one-paragraph version

Two acceptance cards opened — `acc-disp-KnJYxcf7_Zze8MhPRFNJM` (the Projects page reads
the database) and `acc-disp-DgFKxw7pNYTM2eWaLIMhm` (seats attributed by role binding) —
both citing `ev-VV2HvohnhRxRU1YHQRf2C`, which I verified myself names each dispatch and
concludes pass. Both cards carry the live blockers in their **preconditions**, measured by
me rather than taken on trust, and the second card's expected outcome says plainly that
`ql_delivery` will show 0 seats today, not 1.

## 1. The verdict, verified before citing it

I have been burned twice by citing a verdict that did not cover the work, so both of the
dispatch's criteria were read out of the event rather than taken from its prose:

| Check | Result |
|---|---|
| `ev-VV2HvohnhRxRU1YHQRf2C` exists, is `ReviewVerdictIssued`, actor `seat-flux-001` | ✅ |
| `payload.verdict` | **pass** |
| names `disp-KnJYxcf7_Zze8MhPRFNJM` via its own `event_object_refs` row | ✅ |
| names `disp-DgFKxw7pNYTM2eWaLIMhm` via its own `event_object_refs` row | ✅ |

Its `payload.dispatch_id` is a third id (`disp-nhpxzPaQKm0JzIrD-OYL6`) — the dispatch that
asked for the verdict — which is exactly the shape that is *not* a rejection: that field
carries one value, and the per-dispatch refs are the channel that carries several.

I also read what Flux declares unverified, and it is now in both cards: she does **not**
assert the packaged app's pixel-level rendering (her seat's screen is locked — screencapture
fails, assistive access denied), so she verified at the component, data and bundle layers.
And she does not assert the two live data defects are corrected, because they are live
writes outside her boundary.

## 2. The two live blockers, measured by me

Flux's payload names both. I verified each independently rather than repeating her words.

**Blocker 1 — `default_harness` is `Pi` on all 8 seat rows.** `enum Harness` has no `Pi`
variant, so `Agent`/`Harness` validation fails for every row, `list_seats` throws for the
whole table, and `hydrateFromBackend`'s eight-read `Promise.all` aborts. This is the same
whole-table failure mode as the `default_agent` one, one column over — and the operator's
2026-10-17 ruling (`9d8cb57`, `scripts/fix-seat-harness-to-pi.sh`) made it apply to all
eight rows rather than two.

**Blocker 2 — `ql_delivery`'s binding has `role = 'worker'`, which is not in the
`SeatRole` domain.** Measured: the domain is `supervisor / product_owner / architect /
verifier / designer / data_engineer`, plus the tagged `custom(...)` form; `worker` is
neither. `FromStr` errors, my per-project `try/catch` around `listRoleBindings` turns that
into empty roles, and the seat filter then yields **0 seats** for `ql_delivery`.

That second one changes what my own earlier delivery claimed. I wrote "ql_delivery carries
exactly ql", which is true of the filter in a test and **false of what live renders today**.
The card says so in the expected row: today it will show 0, that is a reject, and the root
cause is the data rather than the code.

**What already works:** `seatloom`'s seven bindings all parse — `aegis supervisor`,
`flux verifier`, `lyra product_owner`, `mira designer`, `nimbus architect`,
`onyx data_engineer`, and `vela data_engineer` (someone chose that role when running the
script I provided; I deliberately shipped it with no default). So that half of card 2 is
verifiable the moment blocker 1 clears.

## 3. The two cards

| Card | Steps / expected / preconditions |
|---|---|
| `acc-disp-KnJYxcf7_Zze8MhPRFNJM` | 6 steps, 4 expected, 3 preconditions |
| `acc-disp-DgFKxw7pNYTM2eWaLIMhm` | 6 steps, 3 expected, 3 preconditions |

Both read back `pending` and cite the verified verdict. Aegis's drafts were used as the
base — he wrote them to the plain-language rule and I did not re-invent them — with the
preconditions made specific using my own measurements, and card 2's second expected row
corrected to state the 0-seats reality.

## 4. What I did not do

No live-database write (both fixes are live writes and belong to the operator). No gate
event. No merge into `track`. No acceptance standard relaxed to make a card clickable —
both cards say they cannot be clicked yet, and why.

## 5. Artifact index

- Branch: `packet/mira/knowledge-panel`, containing `track/infra-foundation` @ `9d8cb57`.
- Ledger: `AcceptanceRequested` × 2, and `emit_event(ArtifactCreated)`.
- This delivery note.
