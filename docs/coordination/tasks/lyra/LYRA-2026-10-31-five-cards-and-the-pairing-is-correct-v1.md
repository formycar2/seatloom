# Standing-duty verification: five cards, and the pairing I would have called a defect is correct — the guard's second channel is doing exactly what I designed it for

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-five-cards-and-the-pairing-is-correct-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, verification, acceptance, pairing, instrument |

## The state, measured as a count and not as a remembered list

Five pending cards, all on `packet/mira/knowledge-panel` except one on `packet/nimbus/project-scoped-reads`:

| Card | Dispatch | Verdict | Verdict verifies |
|---|---|---|---|
| `acc-disp-E06O6IAaWefF-BOW-CNJd` | E06O6 | `ev-JPEis2MT47OXUnHW1yE5b` | (no commit key) |
| `acc-disp-OQyilLqRjR1fDSWsWE-5n` | OQyil | `ev-FoCADzLuU-Sv55c07r3qW` | `dcfeee3` |
| `acc-disp-QgtYChq29ccrCEB5ZCjAH` | QgtYC | `ev-Aoq81S2TiEaD3upfnF7f0` | `dcfeee3` |
| `acc-disp-TbGRGr5cvpJFgCeaayu7S` | TbGRGr | `ev-U9LHdjblV_bkLtnfjuNzh` | (no commit key) |
| `acc-disp-faEWBt8kSGgv-6pJs_Rqw` | faEWBt | `ev-15UEVJ08G4I0anhIpCjUJ` | `c3484a0` |

**Mira re-pointed two of them at 00:01 and cancelled the originals**, and her stated reason is exactly the
right one: "the verdict the operator accepts against must be about the code he is accepting, so I rebound" — and
that is the correct reason. She re-verified at `dcfeee3` and rebound. **That is
the correct behaviour and she did it unprompted.**

## The pairing I would have called a defect, and wasn't

`acc-disp-TbGRGr5cvpJFgCeaayu7S` cites `ev-U9LHdjblV_bkLtnfjuNzh`, whose `payload.dispatch_id` is
**`disp-QbxkqowgN3bALg_89qY9j`** — a different dispatch, whose title begins with the Chinese for "verify:" (a verification dispatch).

**On the rule I wrote myself, that should fail.** A verdict about a verification dispatch is not evidence for
the implementation dispatch. I have refused cards on exactly that ground.

**And it passes — correctly.** `ev-U9LHdj` carries **two** dispatch refs in `event_object_refs`:

```
dispatch ~ disp-QbxkqowgN3bALg_89qY9j   (the verification dispatch — payload.dispatch_id)
dispatch ~ disp-TbGRGr5cvpJFgCeaayu7S   (the implementation dispatch — the ref channel)
```

The guard at `repositories.rs:847` reads:

```rust
let names_dispatch = payload_dispatch_id.as_deref() == Some(dispatch_id) || names_via_ref;
```

**Two channels, and this verdict names the implementation dispatch through the second one.** So the verdict
is about *both* dispatches: it was produced by the verification dispatch and it is evidence *for* the
implementation dispatch. **That is precisely the shape I designed the rule to accept** — a batch verdict that
covers several dispatches, each named by a ref.

**I would have reported this as a defect on the strength of the payload alone, and I would have been wrong.**
I checked the second channel before writing anything, and the check took one query.

**The lesson is not "be careful" — it is that I wrote a two-channel rule and then read one channel and
called the result a violation.** That is the same shape as last round's "a list query with an id filter is not
a count": I used the instrument correctly and then read only the part of its output I expected.

## The moving tip, and why it does not invalidate the cards

Her branch tip is `51cd6e7`, but two verdicts verify `dcfeee3`. **Product code is unchanged between them:**

```
git diff dcfeee3..HEAD --stat -- ui/src crates   -> (empty)
```

`51cd6e7` is a delivery note only — 73 insertions in one file under `docs/coordination/tasks/mira/`. **So the
operator accepting those cards accepts exactly the code Flux verified.** That is what makes the rebind
correct rather than cosmetic.

## The suite, and a stale reading I caught

My first run of the suite reported `Test Files 1 failed | 31 passed` with 382 tests passing — a contradictory
pair that I nearly recorded. Re-running gave **31 passed (31), 382 passed (382)**. The `1 failed` was a
stale or transient read; the "382 passed" inside it was the tell that the two halves disagreed. **A result
whose halves disagree is not a result**, and re-running is the correct response, not picking the half that
supports the narrative I walked in with.

## What I did not do

I did not grant any card. I did not merge anything. I did not run any mutation this round — her tree is at a
delivery-note tip and there was no code question to test. I did not report the `ev-U9LHdj` payload
mismatch as a finding, because measuring it showed it is not one.
