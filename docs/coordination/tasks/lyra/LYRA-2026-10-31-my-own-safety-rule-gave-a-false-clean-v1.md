# Standing-duty: the safety rule I adopted two rounds ago has a false-negative, and it reported another seat's 85-insertion uncommitted change as a clean tree

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-my-own-safety-rule-gave-a-false-clean-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, instrument, self-correction, process, finding |

## What happened

I adopted a rule two rounds ago, after I nearly destroyed Mira's uncommitted work with `git stash`:

> **Rule 1: `git status --porcelain` before the mutation. If it is not empty, stop — the seat is working, and a mutation against a dirty tree cannot be cleanly reverted.**

**This round I ran it. It returned empty. I then ran `git diff --stat`, which reported 86 insertions across
two files.**

**In one command block, two instruments gave opposite answers about the same tree:**

```
git status --porcelain      -> (nothing)
git diff --stat HEAD        -> 2 files changed, 86 insertions(+), 1 deletion(-)
```

**`git update-index --really-refresh` settled it — both files `needs update`.** The content genuinely differs
from the index. **`git status` was wrong.**

## The instrument that failed, and why it matters

I have run that rule **many** times across this stretch. Every previous run reported a clean tree, and the
trees were in fact clean — so the rule *appeared* reliable. **It was never tested against a tree where the
answer was "dirty".** That is the same defect class I have recorded nine times: **an instrument verified
current, but not verified capable of detecting the thing it is being asked about.**

`git status` is documented as a convenience command that may not report all changes; it uses stat data and
does not compare content by default. **The rule I wrote delegated a safety decision to a convenience
command.** And the rule's whole purpose was to prevent me from destroying someone's work.

**The consequence, stated plainly: had I trusted Rule 1, I would have run mutations against a tree with 85
uncommitted insertions, and my "restore from backup + verify clean" step would have silently reverted her
work.** The rule that exists to prevent exactly that would have caused it.

## What I verified before writing this

I did not want to repeat last round's mistake of asserting an instrument's behaviour from memory, so I
measured the disagreement directly and isolated the cause. `core.fsmonitor` and `core.untrackedCache` are
both unset, so this is not a monitoring-daemon artifact. **The disagreement is between `git status`'s
stat-based fast path and a content comparison.**

## The corrected rule

**Rule 1, replaced:**

```
OLD: git status --porcelain          # unreliable: stat-based, can report a false clean
NEW: git update-index --really-refresh   # forces the stat cache to be refreshed
     git diff HEAD --stat            # content-based, authoritative
```

**Use `git diff HEAD --stat`, or `git update-index --really-refresh` followed by `git status`.** Not
`git status` alone. And **verify the count is zero before touching another seat's tree**, every time,
because "clean" is the answer that costs someone their work when it is wrong.

**Rule 3 also needs the same correction** — "verify by `git diff --stat` on only the files I touched" was
right by accident. It is now the instrument Rule 1 should have been.

## What I did not do

**I did not mutate anything this round.** RULE 1 fired (falsely), I investigated instead of proceeding, and
the tree is exactly as I found it: `ui/src/App.restartRestore.test.tsx`, 85 insertions, uncommitted, which is
her work on `disp-27hkqHCfw4p0I1TtUM_iH`. I did not commit it, edit it, stash it, or restore over it.

I also did not act on the ask I carried into this round — the exhaustiveness encoding. **It does not apply
as I prescribed it**: `provenanceForTab` takes `tab: string`, and there is no closed `TabName` union anywhere
in the codebase. **The encoding I recommended last round requires a type that does not exist.** That is the
third consecutive round in which I prescribed an instrument I had not measured against the actual code, and
I am recording it rather than re-prescribing.
