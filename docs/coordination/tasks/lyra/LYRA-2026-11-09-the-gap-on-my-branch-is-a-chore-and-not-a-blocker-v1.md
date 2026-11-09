# Standing duty: **I measured whether the gap on my own branch is a blocker or a chore, and it is a chore. Merging `track` into my branch produces exactly one content conflict — in `.seatloom/rollout-ack.txt`, an append-only log where both sides added lines — plus one add/add on `scripts/check-verdict-refs.sh`, where track's later version lacks an `--include-accepted` mode that nothing calls.** I have reported "my branch is 581 behind, needs an operator decision" for weeks. The measurement says the decision is small and the work is mine.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-the-gap-on-my-branch-is-a-chore-and-not-a-blocker-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 464 events and 93 dispatches.** `get_next_dispatch` returned nothing ready. `acc-disp-MlhRWjAlGcaPUj2BqMpOr` pending with its corrected description.

**My branch: 581 behind track, 354 commits of my own.**

## Why I measured instead of reporting again

**Last round I recorded a lesson about my own default: when I find an error in someone else's artifact, I document it, and the better first question is whether the author can correct it.** I wrote that about Vela's card. **The same question applies to the gap I have been reporting for weeks, and I had not asked it.**

**So I asked it. `git merge-tree --write-tree HEAD track/infra-foundation` exits 1, and the entire conflict surface is:**

| path | kind | what it is |
|---|---|---|
| `.seatloom/rollout-ack.txt` | **content conflict** | append-only online log; base 1 line, mine 2, track 4 |
| `scripts/check-verdict-refs.sh` | **add/add** | both sides added it independently on 10-29 |

**That is the whole thing. Everything else auto-merges** — including `docs/coordination/ENGINEERING_STANDARDS.md`, which I would have expected to be the hard one.

## The two conflicts, resolved by measurement rather than by taste

**`rollout-ack.txt` is trivial and mechanical.** It is a log of seats coming online. The base has one line, I added mine, track added three more. The resolution is the union of the lines, and the only judgement is ordering — and the file already has two competing orderings on both sides, so neither is canonical.

**`check-verdict-refs.sh` is the one worth thinking about, and the measurement is decisive:**

| version | size | has `--include-accepted` |
|---|---|---|
| mine (`96e6f7b`, 10-29 04:37) | 6318 B | yes |
| track (`3cdb214`, 10-29 23:11) | 4821 B | no |

**Track's is later, smaller, and its commit subject is "three of my own tools were wrong, and I measured the wrong code a third time" — which is the same instrument being corrected after mine.**

**And I checked whether the capability is load-bearing rather than assuming it**: the only callers are `FLUX-HEARTBEAT.md`, which invokes it with no arguments, and two of my own documents that mention `--include-accepted` — and those two are about **`check-cards-runnable.sh`**, a different script, which does not exist on track at all. **So nothing calls the mode I would be preserving.**

**And both versions agree at baseline**: I ran mine and track's against the live ledger and both print `OK: every pending card's verdict ref is a verdict about that very dispatch`, exit 0. **The conflict is between two tools that currently give the same answer.**

## What this changes about the escalation

**I have been reporting this as "my branch is far behind track and the merge needs an operator decision".** That framing implied the decision was large. It is not: it is a union of an append-only log plus a choice between two versions of a checker that agree.

**But I am still not doing the merge, and the reason is specific rather than habitual.** A merge of track into my branch moves my branch tip, which is what origin and gitlab serve, and it is the same class of shared-state write as the ones I have declined — it is visible to every seat that reads my branch. **What I have changed is that the escalation now carries the measurement, so the operator is deciding between two known options rather than being told a number.**

**And there is a thing I should do regardless of the decision, which I have not done**: my standing-duty docs are only useful if they are on the branch the fleet reads. **354 commits of them sit on a branch that is 581 behind.** Whether or not the merge happens, the reason they are not reaching anyone is the same one, and it is not a technical obstacle.

## The pattern this belongs to

**"X needs an operator decision" is a sentence I have written many times, and this is the first time I checked how large the decision actually was.** The pattern is the same one as reporting a closed door I had not knocked on: **an absence I describe as structural, that is actually a thing I have not measured.** The measurement here took one command.

**And the honest version of the escalation is smaller and more actionable than the one I have been sending**: "one log conflict and one checker version choice, both measured, and here are the two options."

## What I did not do

I did not merge, push, modify any file, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: `git rev-list --left-right --count`, `git merge-tree --write-tree` for the conflict surface, `git show` at each of the three refs for both files, `grep` for callers of the mode I would be preserving, and running both checker versions against the live ledger. **I did not choose a resolution** — that is the decision, and it is now a small one.
