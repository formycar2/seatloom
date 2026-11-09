# Standing duty: **I measured the gitlab item properly and it is now safe, which changes it from a decision into a non-issue. My branch is 137 commits ahead of gitlab's `717d9bf`, and gitlab has exactly one commit I do not have. That commit's entire content is already in my history in a larger form — 108 of its 108 lines appear verbatim in my 169-line version of the same document. So a force-with-lease would lose nothing, and the reason it needs force is an amend, not a divergence.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-the-gitlab-gap-loses-nothing-and-that-was-measurable-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The Ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. Three cards pending.

**And I first corrected a false alarm of my own**: I read my local HEAD as `6b23d35` against origin's `caca1c1` and briefly recorded a discrepancy. **`6b23d35` is `track/infra-foundation`'s tip in the primary checkout, not my branch.** My branch is at `caca1c1` locally and on origin, matching exactly. **The tell was that I read the rev from the wrong directory** — the same class of error as reading the wrong remote, one level down.

## The gitlab item, measured instead of described

**My carried claim, across many rounds: "gitlab stuck at `717d9bf`, not an ancestor of HEAD — no force-push."**

| measurement | value |
|---|---|
| my branch tip (local and origin) | `caca1c1` |
| gitlab tip | `717d9bf` |
| merge-base | `a5fe14f` |
| commits I have that gitlab lacks | **137** |
| commits gitlab has that I lack | **1** |
| that one commit's file | `LYRA-2026-11-04-both-gates-green-…-v1.md`, 108 lines |
| the same file in my history | **169 lines**, at commits `44640f1` and `3c4e499` |
| lines in gitlab's version not in mine | **0** |

**So the single commit gitlab has that I lack is an earlier revision of a document I later amended and extended.** Every one of its 108 lines is present verbatim in my 169-line version. **Nothing would be lost by bringing gitlab current.**

## What that changes

**I have been escalating this as "needs a force-push decision" for many rounds, on the grounds that `717d9bf` is not an ancestor of my HEAD and force-pushing is something I refuse to do.** Both of those were true. **What I had never measured is whether the force would destroy anything, and the answer is no.**

**So the item is smaller than I presented it:**

| | what I said | what is true |
|---|---|---|
| condition | "gitlab stuck" | gitlab's copy of *my branch* is 137 commits behind; gitlab's mainline is 987 behind origin |
| risk | implied | **zero content loss, measured line by line** |
| action needed | "operator decision" | **a force-with-lease on a remote nothing reads**, or nothing at all |
| does it block anyone | — | **no** — origin is healthy and every seat's tooling I have seen reads origin |

**And I want to be careful about what I am not saying.** I am not saying the force should happen, and I am not doing it — force-pushing remains refused, and a remote nothing reads is not worth a force even when it is safe. **I am saying the decision the operator is being asked to make is "may I overwrite a remote copy that contains nothing I do not already have", and that is a smaller question than the one I had been asking.**

## The measurement that mattered, and the one that nearly misled me

**The decisive measurement was `comm -23` between the two versions of the file — gitlab's lines not present in mine.** Zero. That is what converts "needs force" from a risk into a formality.

**And the measurement that nearly misled me was `git patch-id`**, which reported that gitlab's commit content was "not in my recent 200 commits". **That was true and misleading**: patch-id compares whole commits, and my version of the document is spread across two later commits, so no single commit has the same patch. **The right instrument was a line-level comparison of the file, not a commit-level comparison of the history.** I had the answer from the wrong tool and it pointed the wrong way.

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not force-push to gitlab, and I did not push to gitlab at all.** My measurements were reads: `git rev-parse` and `git log` for the ancestry, `git show --stat` and `git cat-file -s` for the two file versions, `git patch-id` for the commit-level comparison, and `comm -23` over sorted unique lines for the content comparison.
