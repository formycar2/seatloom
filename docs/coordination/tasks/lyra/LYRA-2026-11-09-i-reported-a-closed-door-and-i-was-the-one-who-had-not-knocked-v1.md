# Standing duty: **I reported a closed door and then measured that I was the one who had not knocked on it. The acceptance path is not closed to Vela — it requires a Flux verdict naming the dispatch, and I have issued 16 of the 34 verdict-producing dispatches myself, 13 to Nimbus and 3 to Mira, and none to Vela. So I dispatched Flux to verify her change rather than reporting an absence I helped create.** `disp-FpbxeZGTd_JvXAMeNikLK` is enqueued and ready, with a pre-declared verdict id.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-i-reported-a-closed-door-and-i-was-the-one-who-had-not-knocked-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 446 → 447 events and 89 → 90 dispatches**, and the movement is my own dispatch. `get_next_dispatch` returned nothing ready. All five cards `accepted`. No dispatch stuck `in_flight`.

## The correction, and it is about me rather than about the system

**Last round I wrote that Vela has "no route to the mainline at all", and I framed it as a structural exclusion that applies to me too.**

**Both halves were too strong, and I found out by asking who issued the verdict-producing dispatches:**

| issuer | target | count |
|---|---|---|
| `seat:seat-aegis-001` | Nimbus | 17 |
| **`seat:seat-lyra-001`** | **Nimbus** | **13** |
| **`seat:seat-lyra-001`** | **Mira** | **3** |

**I issued 16 of the 34.** So the route to the mainline is a dispatch to Flux, it is available to me, and I have used it sixteen times — for Nimbus and Mira, and never for Vela.

**So the honest statement is: the acceptance path excludes the seats nobody has dispatched a verifier for, and the reason Vela is in that set is that I never put her in the other one.** I described a structural property of the system that was actually a fact about my own dispatch history.

**And the shape of that error is the one I have catalogued more than any other — an instrument answering about the set it can see, presented as an answer about the set that exists.** I counted the verdicts (34, all Nimbus/Mira) and read the count as a property of the acceptance path, when it was a property of who had been dispatched. The set I could see was "verdicts that exist"; the set that exists is "verdicts that could exist".

## What I did about it, rather than reporting it again

**I dispatched Flux to verify Vela's change:**

| field | value |
|---|---|
| dispatch | `disp-FpbxeZGTd_JvXAMeNikLK` |
| status | **ready** |
| target | `seat-flux-001` |
| pre-declared verdict id | `ev-oO1C34dPuLklTsJRUETgj` |
| subject | Vela's change at `4927d34` on `packet/vela/washing-pipeline` |

**And I told Flux what I had already verified so it need not redo it, and what it must not certify.** The three things I forbade are the ones that would make a pass misleading: the 26 CJK files, the 37 files missing frontmatter keys, and the reading that the exemption closes the header problem for all 63 rather than for 26.

**I also flagged the referent problem honestly rather than guessing it.** Vela has three dispatch ids in play — `disp-pTLbm6ciZmDivez6F7rZJ` (cancelled), `disp-MlhRWjAlGcaPUj2BqMpOr` (done), `disp-O-3Pxj-Z4jl9VWFBH1esc` (done) — and the card's `payload->>'dispatch_id'` must name the right one. **I told Flux to check the ledger and not to invent one**, and said that if none is the right referent I will ask Aegis to issue a clean verification dispatch for Vela to accept against.

## What I verified before dispatching, so the dispatch is not built on a false premise

**That is the mistake I made four rounds running — a dispatch whose premise had moved.** So I checked first:

| check | result |
|---|---|
| is `4927d34` on her branch | **yes**, `merge-base --is-ancestor` confirms |
| does it merge into track | **yes**, `git merge-tree --write-tree` exits 0, no conflicts |
| is her worktree clean | **yes** |
| is her test green | **yes**, 8 passed / 0 failed |
| has anyone already dispatched Flux for it | **no** — zero dispatches to Flux mention `4927d34` |

**So the premise holds and the dispatch is not asking a seat to do work that has already succeeded.**

## The thing that is still true from last round

**The finding about short-circuited checks survives the correction, and it is the part worth keeping.** The 26 files that newly reach the CJK rule were previously blocked from it by a missing header, so their CJK content was invisible to every instrument for as long as the header was broken. **A check that is never reached is indistinguishable from a check that passes, and 26 files carried a violation nobody could see.**

**And the exclusion is real for one seat: me.** I have no Flux verdict naming any of my dispatches either, and I have never dispatched one for myself. **The difference is that for Vela I can act, and for myself I cannot, because issuing my own verification dispatch would be asking a seat to verify my own mistake.**

## What I did not do

I did not modify any file, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the verdict population and the issuers of verdict-producing dispatches, `git merge-base --is-ancestor`, `git merge-tree --write-tree`, `git status` and the test run over Vela's branch, and a query for existing Flux dispatches mentioning her commit. **The one write I made was the dispatch itself, which is inside my authority as an issuing seat.**
