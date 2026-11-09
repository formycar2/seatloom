# Standing duty: **I asked whether the three pending cards are actually waiting on me, and found a real problem with one of them. `acc-disp-MlhRWjAlGcaPUj2BqMpOr` names `packet/vela/washing-pipeline`, and **that branch is not on origin at all** — `git ls-remote origin` returns zero matches for it. The other two cards' branches are on origin. So accepting Vela's card would merge from a ref that exists only in this checkout.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-one-of-the-three-cards-names-a-branch-that-is-not-on-origin-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. Three cards pending.

## The question I asked, and what it found

**Last round I established that of my five recurring escalations, only one was a real blocker. This round I applied the same test to the three pending cards: are they genuinely waiting on the operator, and is there anything about them that would make his Accept go wrong?**

**The first check was clickability, and I got a contradiction that turned out to be my own error:**

| probe | result |
|---|---|
| `git merge-tree --write-tree track origin/packet/vela/washing-pipeline` | **`not something we can merge`** |
| `git merge-tree --write-tree track packet/vela/washing-pipeline` | **clean** |
| `bash scripts/check-cards-clickable.sh` | **OK: all 3 pending cards are clickable** |

**The `origin/` prefix was my mistake** — there is no local `refs/remotes/origin/packet/vela/washing-pipeline`, so that ref name resolved to nothing. The instrument was right and I nearly recorded a false conflict. **The tell was that the error message said "not something we can merge" rather than reporting a conflict**, which is a message about a bad ref and not about a merge.

**But chasing my own error is what found the real problem.** Once I asked why that ref did not exist locally, the answer was that it does not exist on origin either:

| card | branch | on origin |
|---|---|---|
| `acc-disp-4xBIRHNOnacKkCwCPJhJG` | `packet/nimbus/candidate-final` | **yes** |
| `acc-disp-MlhRWjAlGcaPUj2BqMpOr` | `packet/vela/washing-pipeline` | **no — zero matches** |
| `acc-disp-bouEIM-49hK0xR_-B8EMm` | `packet/mira/knowledge-panel` | **yes** |

## Why this matters for the operator's Accept

**The card's `how_to_verify` and the acceptance machinery both operate on a branch ref.** If the branch exists only in this checkout, then an Accept performed from here would work and an Accept performed from anywhere else — a fresh clone, another seat's machine, a CI runner — would fail with a missing ref.

**And the corrected clickability instrument cannot detect this, because it checks local refs by design.** `check-cards-clickable.sh` runs `git -C "$GATE_REPO_ROOT" merge-tree --write-tree "$base" "$branch"`, which resolves `packet/vela/washing-pipeline` locally and finds it clean. **So the instrument's "clickable" is true and complete for this checkout and silent about whether the branch exists anywhere else.** That is the same shape as the ancestor-exclusion defect: an instrument answering about the set it can see.

**I am not claiming the card is unclickable.** It is clickable from here, and I verified its four steps pass in an isolated worktree last week. **What I am claiming is that its branch is unpublished, which is a fact about the merge that the operator should have before he clicks**, because it determines whether the merge is reproducible or depends on my checkout.

## What I do not know, and am not guessing

**I do not know whether Vela pushed and the push failed, or never pushed, or pushed to a different remote.** Her branch tip `bc8c06c` exists locally and her worktree is clean. **The honest statement is that the branch is absent from origin and I have not established why** — and unlike the `pgrep` investigation, I am not going to spend seven rounds on it, because last round's lesson is to ask first whether it blocks anyone, and the answer here is that it blocks the operator's merge from anywhere but this machine.

**The actionable item is small: Vela pushes her branch, then the card is fully clickable.** That is a one-command fix by the seat who owns the branch, and it is not mine to do — pushing her branch would be taking an action in her worktree's remote namespace.

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not push Vela's branch.** My measurements were reads: `git ls-remote origin` for the ref list, `git rev-parse --verify` for the local refs, `git merge-tree --write-tree` for clickability under both ref spellings, `git config --get-all remote.origin.fetch` for the refspec, and `grep` over `check-cards-clickable.sh` for what it resolves.
