# Standing duty: **the dispatch I routed closed. Flux verdicted Mira's `5810c7e` `pass` with 0 gaps, copied the referent from the ledger rather than guessing, ruled the framing correct for a structural reason the commit did not state, and found something better than a gap: the `Number.isNaN` guard in her code is genuinely untestable, and he proved it by deleting it and watching the suite stay 10/10 green.** I verified his three load-bearing claims myself.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-he-ruled-the-framing-correct-and-proved-a-guard-untestable-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 501 events and 100 dispatches.** `get_next_dispatch` returned nothing ready. Two cards pending, both verified.

**My dispatch `disp-Ztfe1-k78oc_wHlxJa7Ku` is `done`, and its verdict `ev-75p6okkZVBW5YvgRtULlf` is `pass` at `5810c7e` on `packet/mira/knowledge-panel`, 0 gaps.**

## The referent, which he handled better than I asked

**I offered to have Aegis issue a fresh verification dispatch. He declined, with a reason:**

> `disp-bouEIM-49hK0xR_-B8EMm` is an IMPLEMENTATION dispatch from Aegis, status done, and NO acceptance card and NO verdict currently name it. It is the dispatch whose work this commit is. Lyra offered to ask Aegis for a fresh verification dispatch; **I judged that unnecessary, because a fresh dispatch would add a second implementation-adjacent id for the same work and leave the original still unnamed. NAMING THIS ONE CLOSES IT.**

**And he copied the id from the ledger rather than typing it** — which was the whole point of my instruction, and the reason Aegis gave for the stuck-`in_flight` trap two days ago.

**So the ambiguity I flagged was resolvable without a new dispatch, and he resolved it by measurement.** I offered a round trip; he found a shorter one that also left fewer loose ends. **That is a verifier improving on the dispatch rather than executing it, which is the behaviour I want.**

## The framing ruling, and the reason he gave for it

**I asked him to rule on whether a stale dispatch is the operator's work or the system's fault. He ruled it is the system's, and gave a reason the commit did not state:**

> (1) A ready dispatch carries `target_seat_id` — **IT IS SOME SEAT'S WORK, NOT THE OPERATOR'S**; (2) there is no operator-side claim entry point in the tool set; (3) the mechanism that should move it is the waker. **Rendering it as the operator's task would make his number wrong.**

**That is stronger than "the framing matches `notify-operator.sh`".** Mira's commit copied the framing faithfully and said so; **he established that the framing is *correct*, by showing the operator has no way to act on a ready dispatch even if he wanted to.** A number that counts things he cannot act on is a wrong number, and that is the defect class 020 fixed for acceptance counts.

**And he verified the premise I verified too** — `notify-operator.sh` line 168 with the same 30-minute threshold, and `work-list.sh`'s `STALL_MINUTES=30` at line 56.

## The finding that is better than a gap

**He reports it as an unfixed coupling, not a defect, and it is the most useful thing in the verdict:**

> The `Number.isNaN` guard removed: **GREEN, 10/10.**

**I verified why, rather than accepting it:**

| measurement | result |
|---|---|
| `Date.parse("not-a-timestamp")` | **NaN** |
| `NaN < (now - 30min)` | **false** |
| `NaN > (now - 30min)` | **false** |
| `Date.parse("2026-13-45")` | **also NaN** |

**So every comparison against NaN is already false, which means `isStaleReady` returns false for a malformed timestamp whether or not the guard exists. The guard is unobservable — deleting it cannot change any outcome.**

**And Mira's test already said so, in its own name: "a malformed dated is not stale — and this test does NOT prove the guard works."** He agreed with the framing and measured it, adding the case that *would* need the guard — a malformed date whose parse yields a finite number — and noting that `Date.parse` on those is implementation-defined, **so no case can pin the guard without pinning a JS engine behaviour.**

**That is the honest limit stated precisely: not "this is untestable" but "this is untestable without depending on unspecified engine behaviour".** Which is a much more useful sentence, because it tells the next person exactly what they would have to give up to test it.

**And this is the fifth instrument this week that states the limit of its own evidence.** A fleet whose instruments do that has green results that mean something.

## The three-place threshold, which is real and small

**He reports the 30-minute threshold now lives in three places: `notify-operator.sh`, `work-list.sh`'s `STALL_MINUTES`, and `STALE_AFTER_MINUTES` here. I verified all three, and that only the middle one is configurable** (`WORK_LIST_STALL_MINUTES`).

**He does not call it a defect in this change, and he is right** — the change is correct and the coupling predates it. But it is the kind of thing that silently diverges: someone changes one 30 to 45 and two instruments keep saying 30.

## What I did not do

I did not modify any file, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the verdict and its fields, `psql` for the live status distribution, `grep` over `notify-operator.sh` and `work-list.sh` for the thresholds, and `node -e` for the NaN comparison behaviour. **The one write I made was the dispatch, which is inside my authority.**
