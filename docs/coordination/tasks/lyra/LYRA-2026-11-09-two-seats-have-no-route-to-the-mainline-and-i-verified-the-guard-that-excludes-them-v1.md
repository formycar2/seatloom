# Standing duty: **Vela completed her dispatch and could not open the card, and she is right — `request_acceptance` requires a passing Flux verdict naming the dispatch, and all 34 verdicts name Nimbus or Mira dispatches. Zero name Vela. Zero name me.** So two of the fleet's seats have finished, verified, mergeable work with no route to the mainline at all. I verified her claim in the ledger and in the guard's source.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-two-seats-have-no-route-to-the-mainline-and-i-verified-the-guard-that-excludes-them-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 446 events and 89 dispatches.** `get_next_dispatch` returned nothing ready. All five cards `accepted`. No dispatch stuck `in_flight`.

**Aegis issued `disp-O-3Pxj-Z4jl9VWFBH1esc` to Vela at 04:01 asking her to open the acceptance card for her own gate change, and she completed it at 04:10 — by recording `016b077` "doc-gate acceptance blocked — no Flux verdict exists for any Vela dispatch".** Her branch is `packet/vela/washing-pipeline` at `016b077`, clean, 7 ahead of track.

## Her claim, verified in the ledger

**Her dispatch asked her to open the card. She could not, and said so rather than forging the reference.**

| measurement | result |
|---|---|
| `ReviewVerdictIssued` events | **34**, all by `seat:seat-flux-001` |
| verdicts naming a Nimbus dispatch | **30** |
| verdicts naming a Mira dispatch | **3** |
| verdicts naming a Vela dispatch | **0** |
| verdicts naming a Lyra dispatch | **0** |

**And one more: no card has been opened since 11-07**, so the last two cards on the board are the pair I reported and the operator accepted. The card-opening path has been silent for two days while work has landed.

## The guard that excludes them, read in source

**I did not take her word for the mechanism. Guard 2 in `crates/seatloom-mcp-emit/src/dispatch.rs`, at the `request_acceptance` entry:**

> the Flux verdict referent exists, is a `ReviewVerdictIssued`, says ... **A verdict counts for a dispatch iff `payload->>'dispatch_id'` ...**

**So the card is gated on a verdict that names the dispatch, and the only seat that issues verdicts is Flux, and Flux has only ever verdicted Nimbus and Mira.**

**Which means the acceptance path is not "open to any seat with verified work" — it is "open to the seats Flux happens to have reviewed".** That is a rule enforced at one lifecycle point that excludes everyone else, and it is invisible from the inside because the seats it excludes are the ones who cannot open cards to say so.

## Why Aegis's handling of it is the right shape

**He explicitly declined to merge it himself:**

> a change to the doc gate is a change to a control on the integration line, and that should go through the acceptance path rather than my judgement.

**And he told her the plan must state all three things — what the change does, what it does not do, and that the 26 CJK files are a separate decision — "because an operator who accepts on a false premise is worse than one who rejects."**

**So the routing was correct and the destination was closed.** That is a different failure from a wrong routing, and it is harder to see: nothing in the dispatch's execution was wrong, and the work is verified and mergeable, and it still cannot land.

**And her own wording is the correct note rather than a forged one: "Not something to forge, so the plan and the verification are recorded here for whoever can act."** A fixture that fabricates an identifier is worse than one that uses none, and she applied that to a card reference.

## What this means for me, stated plainly

**I am in the same position she is.** I have no Flux verdict naming any of my dispatches, so I cannot open a card for my own work either. My standing-duty docs have been landing on my branch for weeks and reaching the fleet only if the branch is merged — which is the operator's decision I have escalated repeatedly.

**So the exclusion is not Vela's problem. It is a property of the acceptance path that applies to every seat Flux has not reviewed, and the two seats it currently blocks are Vela and me.**

## What I did not do

I did not modify any file, dispatch anyone, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not forge a verdict reference for Vela's change, and I did not ask Flux to produce one on my or her behalf** — that would be routing around the boundary by delegation, which is the same violation as taking the action myself. My measurements were reads: the ledger for the verdict population and the card dates, `git log` and `git status` over Vela's branch for her state, and `grep`/`awk` over `dispatch.rs` for Guard 2's text.
