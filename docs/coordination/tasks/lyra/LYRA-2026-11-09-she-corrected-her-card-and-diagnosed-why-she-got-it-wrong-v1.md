# Standing duty: **Vela corrected her own card using the path I found, and she did more than fix the number — she diagnosed the mechanism of her own error. "An earlier version of this card said 11 files genuinely lacked a frontmatter key, which was 37 minus 26: the CJK count subtracted twice." That is exactly right, and it is a different class of correction from replacing a wrong figure with a right one.** I re-verified every number in the corrected card against my own measurement and they match to the unit.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-she-corrected-her-card-and-diagnosed-why-she-got-it-wrong-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 463 events and 93 dispatches.** `get_next_dispatch` returned nothing ready. **`acc-disp-MlhRWjAlGcaPUj2BqMpOr` is `pending` and now carries the corrected plan.**

## The reopen path worked as I predicted

**The event history for her card:**

| time | event |
|---|---|
| 11-09 13:09 CST (05:09 UTC) | AcceptanceRequested |
| 11-09 13:47 CST (05:47 UTC) | **AcceptanceCancelled** |
| 11-09 13:49 CST (05:49 UTC) | **AcceptanceRequested** (reopened) |

**Same id, two minutes apart, and my dispatch `disp-RLW1rHOazPBn45dZKQY6O` is `done`.** So the mechanism I verified in source behaves as documented: the row is never deleted, the id is stable, and the correction is attributable to a cancel event.

**And this is now the second time this path has been used on this board**, the first being one of the cards you accepted on 11-05. So a first-class correction operation is not just available, it is in use.

## The correction, and why the diagnosis matters more than the fix

**Her corrected plan now says, verbatim:**

> THE RESIDUAL, CORRECTED. 63 of the 90 knowledge files still fail, split by first-reported rule into 37 on the header rule and 26 on the CJK rule. 37 and 26 are the SAME population counted two ways — 37 + 26 = 63 — not two separate populations. An earlier version of this card said 11 files genuinely lacked a frontmatter key, which was 37 minus 26: the CJK count subtracted twice. There is no population of 11 anywhere in the breakdown.

**I re-verified every figure independently:**

| her claim | my measurement |
|---|---|
| 63 of 90 knowledge files still fail | **63** |
| 37 on the header rule | **37** |
| 26 on the CJK rule | **26** |
| `id` missing from 27 files | **27** |
| `tags` from 24 | **24** |
| `date` from 15 | **15** |
| groups: `id date tags` 14, `id` 13, `tags` 9, `date tags` 1 | **14 / 13 / 9 / 1** |

**Every number matches to the unit.** And she added the warning that the missing-key counts overlap, so they sum to more than 37 — which is the thing that would otherwise make a reader distrust the 27/24/15 figures.

**But the diagnosis is the part worth keeping.** She did not write "11 was wrong, it is 37". She worked out *how* the 11 arose: it was 37 − 26, the CJK count subtracted a second time. **That means the error was a double subtraction on overlapping populations, not a typo, and knowing the mechanism is what makes it impossible to repeat** — a reader who knows the trap will not subtract a count that was already excluded.

**A correction that names its own mechanism is worth more than a correction that supplies a right number**, because the right number can be re-derived wrongly and the mechanism cannot.

## What this says about the correction path

**I found the path and she walked it, and the result is better than what I asked for.** I gave her the figures; she gave the figures *and* the mechanism *and* the overlap warning *and* the note that 37 and 26 are the same population counted two ways.

**And the alternative was available: I could have written a second document saying the number was wrong, which is what I have done before and which changes nothing.** The difference is that this time I looked for whether the number could be fixed by the person who owned it, and it could.

**So the lesson I am recording is about my own default.** When I find an error in another seat's artifact, my instinct has been to document it. **The better first question is whether the author can correct it, and whether a mechanism exists for that** — because a correction the author makes is attributable, carries their diagnosis, and leaves the artifact usable, while a correction I document leaves the artifact wrong and adds a second document nobody reads.

## What the operator now has

**A card whose substance I verified independently in an isolated worktree (four steps, all passing, negative case checked by comparing `HEAD` before and after) and whose description now matches my measurement exactly.**

**So the residual is honest: 63 of 90 knowledge files remain non-conforming — 37 on the header rule, 26 on the CJK rule — and the card says plainly not to accept on the assumption the exemption empties the directory.** The remaining work is a convention decision that belongs to you, and it is stated as such rather than absorbed into a number.

## What I did not do

I did not modify any file, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the dispatch and acceptance event history, the card's `acceptance_plan` for her corrected text, and a patched copy of `verify-docs.sh` in her worktree — removed afterwards, with her tree verified clean — for the independent re-verification.
