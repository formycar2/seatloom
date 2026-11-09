# Standing duty: **the card I made possible opened, and I ran its four verification steps in an isolated worktree before the operator acts on it. All four pass exactly as written — the negative case is refused with exit 1 and no commit object is created, the positive case is allowed, the exemption does not leak, and the test is 8/8. But the card's description of what remains is arithmetically wrong in two places, and an operator who accepts on it would be accepting on a false premise about the residual.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-09-the-card-passed-and-its-residual-arithmetic-is-wrong-v1 |
| status | open |
| author | lyra |
| date | 2026-11-09 |
| version | v1 |
| tags | measured, standing-duty |

## The route, closed end to end

**Last round I dispatched Flux to verify Vela's change because I had measured that I was the one who had never knocked. It worked:**

| step | result |
|---|---|
| my dispatch | `disp-FpbxeZGTd_JvXAMeNikLK` → **done** |
| verdict | `ev-oO1C34dPuLklTsJRUETgj`, **pass**, naming `disp-MlhRWjAlGcaPUj2BqMpOr` |
| card | `acc-disp-MlhRWjAlGcaPUj2BqMpOr` → **pending** |

**And Flux used the referent I flagged rather than guessing one.** I told it three dispatch ids were in play and to check the ledger; it named the right one. **A dispatch that asks a seat to do work requiring an artifact that does not exist gets refused or faked, and this one got answered correctly because I named the ambiguity instead of resolving it silently.**

## The four steps, run by me in an isolated worktree

**I did not take the card's word either. I added a detached worktree at `packet/vela/washing-pipeline`, checked out `4927d34 -- scripts/`, and drove the probes:**

| step | card expects | I measured |
|---|---|---|
| new knowledge file, no frontmatter, committed | refused, exit 1, **no commit object** | ✓ refused with the exact message, **`HEAD` did not move** |
| same file with frontmatter, committed | allowed, exit 0 | ✓ commit `4fcf37e` created |
| plain `.md` outside `knowledge/` | still refused | ✓ refused, exemption does not leak |
| `scripts/test-knowledge-convention.sh` | passed=8 failed=0 | ✓ **8/8** |

**The negative case is genuinely load-bearing and I verified the part that is easiest to get wrong**: the refusal message prints and the command still exits non-zero, so a reader who checks only the message would miss that a commit object was created. I compared `HEAD` before and after. **It did not move.**

**Containment also verified**: `bba1589` (the sync merge) and `4927d34` (the change) are both ancestors of the branch, and `git merge-tree --write-tree` against `track/infra-foundation` exits 0 with no conflicts.

**Both real applications are byte-identical to before — sandbox still the 95-byte fake md5 `7c0a6e2a…`, main app md5 `28753f61…`. The worktree was removed and Vela's tree is still clean.**

## Where the card is wrong, and why it matters before you act

**The card's `expected` block says:**

> 26 knowledge files are unblocked from the header rule, and 37 still fail: 26 on the CJK rule and 11 on frontmatter genuinely missing a key.

**That does not add up, and it contradicts itself twice.** Its own `preconditions` say the same thing a second way, and both are wrong:

| what the card claims | what I measured |
|---|---|
| "37 still fail: 26 CJK + 11 frontmatter" | **26 CJK + 37 frontmatter = 63** |
| "the header rule goes 63 → 37, NOT 63 → 0" | **correct**, and it is the same 37 |
| "11 on frontmatter genuinely missing a key" | **37** — by key: `id` 27 files, `tags` 24, `date` 15 |

**So the residual is 63 files still failing, split 26 CJK and 37 header, and not 37 files split 26 and 11.** The 11 appears to be a miscount of one row of the breakdown (`id` alone on 13, or `id date tags` on 14) rather than a sum.

**Why I am recording this before you accept rather than after: the card explicitly warns against the opposite error** — "Do NOT accept this card on the assumption that the exemption empties the directory — it does not, and 37 files remain non-conforming." **The warning is right and the number attached to it is wrong, and the direction of the error is the same one the warning is about.** An operator who reads "37 remain" and forms a mental model of 11 small frontmatter gaps has understated the remaining work by 26 files.

**The card's substance is sound.** The change does what it says, the negative case is real, and the scope exclusions are exactly right. **This is a number in the description, not a defect in the change** — which is precisely the kind of thing that should be corrected before an accept rather than discovered after one.

## What I did not do

I did not modify any file, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads plus one isolated detached worktree: `git worktree add --detach`, `git checkout 4927d34 -- scripts/`, `printf`, `git add`, `git commit` and `git rev-parse HEAD` inside `/tmp/lyra-vela-probe`, then `git worktree remove --force`. **I did not accept the card** — that is the operator's action, and I have now given him the correction he needs to make it an informed one.
