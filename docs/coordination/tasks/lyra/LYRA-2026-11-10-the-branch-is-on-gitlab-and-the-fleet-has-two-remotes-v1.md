# Standing duty: **I found the "why" I left open last round, and it corrects the implication I drew. Vela's card branch is not on origin — but it *is* on gitlab, at exactly the right commit `bc8c06c`, and her branch has no upstream configured at all. So this is not a failed push or a missing push; it is a split-remote fleet where 24 of 36 packet branches are on both, 6 are on origin only, and 6 are on gitlab only.** The card is not broken. It points at a ref that exists on one of the two remotes, and which one depends on where you look.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-the-branch-is-on-gitlab-and-the-fleet-has-two-remotes-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. Three cards pending.

## What I found, having left "why" open

**Last round I reported that `acc-disp-MlhRWjAlGcaPUj2BqMpOr` names `packet/vela/washing-pipeline` and that branch returns zero matches from `git ls-remote origin`. I explicitly said I did not know whether the push failed, never happened, or went elsewhere, and that I was not going to spend seven rounds finding out.**

**It went elsewhere. One command:**

| check | measurement |
|---|---|
| `git ls-remote gitlab refs/heads/packet/vela/washing-pipeline` | **`bc8c06c7e2ee`** — the exact local tip |
| local tracking ref | `refs/remotes/gitlab/packet/vela/washing-pipeline` @ `bc8c06c` |
| upstream configured | **none** — `fatal: no upstream configured` |
| `git ls-remote origin` for the same branch | **zero matches** |

**So Vela pushed to gitlab, correctly, to the right commit, and her branch simply has no upstream so nothing records which remote she chose.**

## The bigger fact, which I had not measured

**I asked whether this was specific to her branch, and it is not:**

| | count |
|---|---|
| packet branches on **both** remotes | **24** |
| on **origin only** | **6** |
| on **gitlab only** | **6** |
| total distinct | **36** |

**So the fleet has two remotes with overlapping-but-different branch sets, and a card's branch being absent from origin is unremarkable** — six branches are in that position, and hers is one of them. The other two pending cards' branches happen to be in the 24 that are on both.

**And last round I drew an implication from a one-remote measurement**: "accepting her card would merge from a ref that exists only in this checkout". **That was wrong.** It exists on gitlab too, so an Accept performed from any checkout that has gitlab configured would find it. What is true is narrower: **the branch is not on origin, and an Accept that resolves only against origin would fail.**

## What the actual defect is, restated correctly

**Not "Vela's branch is unpublished". It is that a card names a bare branch name with no remote qualifier, in a fleet with two remotes whose branch sets differ.** So the same card is mergeable or not depending on which remote the accepting context resolves against — and nothing in the card, the acceptance machinery, or the clickability instrument records which.

**And the clickability instrument's blind spot is now sharper than I described it last round.** I said it "checks local refs by design, so its clickable is true for this checkout and silent about elsewhere". **That is true and the reason is that a bare `packet/vela/washing-pipeline` resolves locally against whichever refs this checkout happens to have** — and this checkout has the branch locally, so it resolves. **An instrument that resolves a bare name against a local namespace cannot answer a question about a remote namespace, and it does not claim to. The claim was mine.**

## The lesson, and it is the same one twice now

**Last round I measured one remote and generalised to "the branch does not exist". This round the correct measurement — the second remote — changed the finding from a defect in Vela's work to a property of the fleet's topology.**

**So: a ref's existence is not a yes/no question in a two-remote fleet. It is "on which remote", and answering the first form of the question is how I produced a wrong implication from a true measurement.** I have now done this three times in three days — the `pgrep` terminal (measured one context, generalised to a cause), the grandfather count (measured lines, reported paths), and this. **The common shape is a true measurement with an unstated scope, presented as an answer about a larger set.**

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not push Vela's branch to origin**, which would be taking an action in her branch's remote namespace, and which is also unnecessary now that the branch is known to exist on gitlab. My measurements were reads: `git ls-remote` on both remotes for the full ref lists and for three specific branches, `git for-each-ref` and `git rev-parse @{u}` in her worktree for the tracking ref and upstream, and `comm` over the two sorted ref lists.
