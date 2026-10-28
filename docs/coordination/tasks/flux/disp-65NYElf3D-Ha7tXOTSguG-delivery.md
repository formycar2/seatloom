# Delivery — `disp-65NYElf3D-Ha7tXOTSguG` (Flux): the five-slice verdict re-issued, linked per dispatch

- **Dispatch**: `disp-65NYElf3D-Ha7tXOTSguG`, issued by seat-lyra-001. A re-issue of my
  `disp-Tn-83tjvS_B6YgOV4dCCF` verdict: the conclusion content stands, only its **association**
  changes.
- **Verdict event**: `ev-NFPr-zN36by-i9SE-0FPH` — pre-declared id, emitted exactly as required
  (`ReviewVerdictIssued`, `verdict: pass`, `reissue_of: ev-0idwakl17abEU75LNED24`).
- **What changed**: nothing about the measurement. What changed is that the verdict now **names each
  of the five dispatches** through its own `event_object_refs` row, which is the channel Nimbus's new
  merge gate accepts.

## Files changed

| File | Change |
|---|---|
| `docs/coordination/tasks/flux/FLUX-2026-10-16-mira-five-slices-verdict-v1.md` | **+10 lines, 0 deletions** — a clearly-marked editorial pointer at the top naming the re-issue and why it exists. No conclusion touched, nothing re-measured. |
| `docs/coordination/tasks/flux/disp-65NYElf3D-Ha7tXOTSguG-delivery.md` | this note |

Committed as `9354aa9`. No source file was changed; the live database was touched only by read-only
`SELECT`s, plus the one required `ReviewVerdictIssued` emission.

## The defect the dispatch described, verified in the code before acting on it

`SeatloomDb::flux_verdict_facts` (`crates/seatloom-core/src/db/repositories.rs:704`, added by
`e5dbea8`) decides whether a verdict counts for a dispatch with:

```rust
let names_dispatch = payload_dispatch_id.as_deref() == Some(dispatch_id) || names_via_ref;
```

`payload_dispatch_id` comes from `payload->>'dispatch_id'` — **one** value. `names_via_ref` comes
from `event_object_refs WHERE event_id = $1 AND ref_type = 'dispatch' AND ref_id = $2`. Both
channels were checked against my original event, and both missed for all five slices:

```
ev-0idwakl17abEU75LNED24   payload->>'dispatch_id' = disp-Tn-83tjvS_B6YgOV4dCCF
                           its only dispatch ref  = disp-Tn-83tjvS_B6YgOV4dCCF

  disp-history-sessions-ui     via_payload=f via_ref=f  => names_dispatch=FALSE
  disp-cEyS0pTJRZaC3b64vFUdR   via_payload=f via_ref=f  => names_dispatch=FALSE
```

So `request_acceptance` would have refused with `VerdictNotAboutThisDispatch` for every one of the
five — meaning the three slices still on `packet/mira/knowledge-panel` could never have opened a
legal card and could never have merged. **The dispatch's diagnosis is correct, and it was the live
state, not a theoretical risk.**

## What I emitted

`ReviewVerdictIssued` `ev-NFPr-zN36by-i9SE-0FPH`, verdict value **unchanged** (`pass`), evidence
**unchanged** (the same verdict document), subject **unchanged** (that document), actor myself. Its
refs carry **one `ref_type='dispatch'` row per slice**, all `role='context'`:

```
dispatch:disp-history-sessions-ui      role=context
dispatch:disp-ol420Y586tNIsbbeh5nlU    role=context
dispatch:disp-cEyS0pTJRZaC3b64vFUdR    role=context
dispatch:disp-QBVu3LeruaTavYLIN5BCI    role=context
dispatch:disp-CEa11raWfi8W_UHiXGKBw    role=context
document:…/FLUX-2026-10-16-mira-five-slices-verdict-v1.md   role=subject   (exactly one subject)
```

`payload.dispatch_id` names `disp-65NYElf3D-Ha7tXOTSguG` — the dispatch that asked for the re-issue,
which is the honest subject of *this* event. That channel can only carry one value, which is exactly
why the five needed the refs channel.

The payload states, in plain terms, that this is **the same measurement re-stated** and not a second
independent re-verification (`reissue_of`, `not_a_second_verification`, `reissue_reason`,
`verdict_applies_to`, `how_the_association_is_made`, `why_it_mattered`,
`boundary_honoured`).

## Verified end-to-end: the gate now accepts it for all five

Reproducing `flux_verdict_facts`'s four guards against `ev-NFPr-zN36by-i9SE-0FPH`, per dispatch:

| dispatch | exists | event_type | verdict | names via payload | names via ref | **names_dispatch** |
|---|---|---|---|---|---|---|
| `disp-history-sessions-ui` | t | ReviewVerdictIssued | pass | f | **t** | **TRUE** |
| `disp-ol420Y586tNIsbbeh5nlU` | t | ReviewVerdictIssued | pass | f | **t** | **TRUE** |
| `disp-cEyS0pTJRZaC3b64vFUdR` | t | ReviewVerdictIssued | pass | f | **t** | **TRUE** |
| `disp-QBVu3LeruaTavYLIN5BCI` | t | ReviewVerdictIssued | pass | f | **t** | **TRUE** |
| `disp-CEa11raWfi8W_UHiXGKBw` | t | ReviewVerdictIssued | pass | f | **t** | **TRUE** |

And the gate's other guards are already satisfied: all five dispatches are `status='done'` and owned
by `seat-mira-001`, so only the verdict-naming half was ever missing. Both verdict events exist as
two distinct rows; no id collision.

## Boundaries honoured

No merge. No integration. No live-database write beyond the one required verdict emission. **No
change to the original conclusion content** — the verdict doc diff is 10 insertions and 0 deletions,
and nothing was re-measured. I did not re-run any of the five slices' gates; the dispatch said not
to, and I had already measured them.

## Forward recording (D5 Phase 2 Lane B)

Nothing recorded via `remember`. The finding here is a mechanical property of a gate Nimbus just
shipped, already written into the new event's payload and the doc pointer; it does not change a
future action of mine in any way that is not already recorded there, and it is nobody's preference.
Per "三条全满足才记;存疑不记", zero entries.
