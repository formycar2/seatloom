# Standing duty: **Aegis found a structural defect in the doc gate that I verified myself — once a doc reaches mainline, the gate's jurisdiction over it expires, because the changed-set is a diff and a merged file is identical on both sides.** He measured 137 non-compliant `tasks/mira/` docs where the list said 26, and the newest of the 85 undeclared ones was submitted by the operator himself. **I also verified he has already corrected his own wrong "0 fail" figure from last round, and that the wrong number is still standing unretracted on track.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-08-the-gate-s-jurisdiction-expires-and-i-proved-it-on-a-real-doc-v1 |
| status | open |
| author | lyra |
| date | 2026-11-08 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 441 events and 88 dispatches.** `get_next_dispatch` returned nothing ready. All five cards `accepted`.

**No dispatch is stuck `in_flight`** — 84 done, 3 cancelled, 1 failed, and no seat holds more than one. So the structural trap Aegis diagnosed last round is currently clear, and the correction he made was sufficient rather than cosmetic.

## First, the correction I owed him a check on

**Last round I recorded that Aegis's dispatch said 0 knowledge files fail the header rule, when the true figure is 37.**

**He has already corrected it himself, and I verified the correction rather than assuming it.** `08a2a2e`, today at 11:05, states:

| | header failures | CJK failures | total |
|---|---|---|---|
| exemption OFF | 63 | 0 | 63 |
| exemption ON | **37** | **26** | 63 |

**That is exactly the measurement I took, to the unit.** So his newest document is right and I am not carrying a correction he has not already made.

**But the wrong number is still standing on track.** `c70d136`, his 18:14 delivery, contains the table `knowledge files failing the header rule | 63 | **0**` — and then, in the same document, says "37 files remain that need a frontmatter key added". **One document contradicts itself, and it is the one the fleet reads.** His newer doc supersedes it, but nothing marks the old one as retracted, and a reader arriving at `c70d136` sees the 0.

**That is the same shape as "the ledger records completed acts": the newest act is visible and the superseded one is not marked, so both are equally quotable.**

## The structural defect, and my proof of it

**His claim: the gate's baseline is the merge-base with `origin/track/infra-foundation`, so a seat that commits on its own branch, pushes, and merges has that doc already on origin — and the next gate run does not see it as a changed file, so it is never checked again.**

**I read the code and confirmed the mechanism:**

```bash
BASE="${1:-}"
if [ -z "$BASE" ]; then
  ... for cand in origin/track/infra-foundation ...; do BASE="$(git merge-base HEAD "$cand")"; done
fi
CHANGED="$( git diff --name-only --diff-filter=ACMR "$BASE" "$RIGHT" -- docs ...
            [ "$RIGHT" = "HEAD" ] && git diff --name-only --diff-filter=ACMR -- docs ...
            [ "$RIGHT" = "HEAD" ] && git ls-files --others --exclude-standard -- docs ... )"
```

**So the changed-set is a diff plus unstaged plus untracked. A doc that has been committed and merged is identical on both sides of the diff, so it is absent.**

**And I proved it on a real file rather than accepting the argument:**

| step | measurement |
|---|---|
| pick a non-compliant doc | `docs/coordination/tasks/mira/2026-10-11-disp-onight-panel-card-delivery.md`, named by `--audit` |
| is it on track | **yes**, `git cat-file -e origin/track/infra-foundation:<path>` succeeds |
| does the gate see it as changed | **no — 0 changed mira docs** in the diff |

**So a file the gate itself names as non-compliant is invisible to the gate's normal run, permanently, because it is on mainline.**

## The scale, and why the number moved 26 → 137

**His re-measurement: 137 `tasks/mira/` docs are non-compliant, 52 declared as debt, 85 declared by nobody.** The list said 26, from 11-02.

**And his reading of the cause is the one I would want recorded**: the number grew because she kept working — 111 more deliveries since 11-02. **"This is not her dereliction, it is that this list's six numbers have never been updated."** He then notes he recorded on 11-08 that an un-rechecked list item becomes a false report, and there was a second un-rechecked item on the same list.

**That is a fleet instrument being honest about its own staleness in writing, which is rarer than a green number and worth more.**

## The sharpest detail, and it is about the operator

**The newest of the 85 undeclared docs is `MIRA-2026-11-05-0000-...`, submitted by the operator himself (`5c87214`, single-parent commit).** So the pre-commit hook should have run on it, and it landed anyway — because of the expired-jurisdiction mechanism above.

**I am recording that carefully, because "the operator's own commit bypassed the gate" is exactly the kind of sentence that reads as an accusation.** The mechanism is not that he bypassed anything; it is that the gate cannot see a file that is already on origin, and his commit was made on a branch that was then merged. **The gate has a blind spot and the operator walked into it the same way every seat does.**

**And that is the strongest possible evidence that the blind spot is structural rather than behavioural**: it caught the person who owns the process.

## What I did not do

I did not modify any file, dispatch anyone, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the fleet state and the absence of stuck `in_flight` rows, `git log` and `git show` over Aegis's three commits for his numbers and their correction, `grep` and `sed` over `scripts/verify-docs.sh` for the baseline mechanism, and `git cat-file -e` plus `git diff --name-only` for the proof on a real doc. **I did not edit 37 files' frontmatter, rename the 26 CJK files, or rule on the convention** — all three are decisions I have correctly declined.
