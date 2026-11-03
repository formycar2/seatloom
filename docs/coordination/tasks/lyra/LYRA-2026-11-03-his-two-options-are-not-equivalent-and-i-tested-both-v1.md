# Standing-duty verification: I tested both options in his backlog and the first one does not work — adding the 8 header fields clears §2 but leaves the gate red on §10, because the operator's document is in Chinese. Only the grandfather option makes it green. His backlog presents them as equivalent choices and they are not.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-03-his-two-options-are-not-equivalent-and-i-tested-both-v1 |
| status | open |
| author | lyra |
| date | 2026-11-03 |
| version | v1 |
| tags | standing-duty, verification, choice, gate |

## Why I tested it

Aegis's standing backlog presents the operator with a choice for the budget document:

> 1. I add the 8 template header fields (header only, not one word of content changed), or
> 2. it goes into `.gitlab/docs-grandfather.txt` (acknowledge the debt, do not pay it).

**Two options offered as if they were equivalent ways to reach the same outcome.** My heartbeat's step 3.4 is
*push to the operator what is his to decide, in the smallest form that lets him decide* — and a choice between
two options that do not both work is not a decision, it is a trap. **So I tested both, on a scratch
repository, without touching his file.**

## Option 1: add the 8 header fields — does not make the gate green

I built a scratch repo, copied the gate and the grandfather file into it, prepended a complete 8-field header
to a copy of the document, and ran the gate.

```
✗ docs/coordination/knowledge/BUDGET_POOL_MODEL_MAP.md:
  contains CJK text; new docs must be English (§10)
EXIT=1
```

**The §2 complaint is gone** — I verified `missing universal-header` appears zero times. **But the gate is
still red, on §10, because the document's content is in Chinese.**

**And it always would be.** The document is the operator's own, written in Chinese, 404 lines of it. Option 1
adds a header and leaves the content, so §10 fires. **Option 1 as offered converts one red into another red.**

**I am not saying the document should be translated.** §10 exists for a reason and the operator's own knowledge
base is in Chinese by his choice. **I am saying option 1 does not do what it is offered as doing.**

## Option 2: grandfather it — makes the gate green

Same scratch setup, original headerless file, one line appended to `.gitlab/docs-grandfather.txt`:

```
verify-docs: changed docs vs HEAD — all conform.
EXIT=0
```

**Grandfathering bypasses both §2 and §10.** The gate is green.

## So the choice is not "which fix", it is "which debt"

**The two options are not two routes to green. One is a route to green and the other is a route to a
different red.** The real choice is:

- **Grandfather it** — green now, and the debt is named in the file that records it, which is the honest
  version of "we know and we are not paying it."
- **Add the header AND deal with §10** — which means either translating 404 lines of the operator's own
  content, or amending §10 to exempt a class of operator-authored knowledge documents.

**The second is a policy decision about the gate, not a metadata fix.** Aegis's option 1 understates it, and
an operator who picks option 1 expecting green will find the gate still red and will have learned that the
choice he was given was not real.

## The thing I want on record about how I tested this

**I could not run the gate against a copy of his file in place, because the gate reads the repository it
stands in.** My first attempt copied the script to `/tmp/gatetest` and ran it — and it **refused**, with
exit 2, because `cd "$(dirname "$0")/.."` resolved outside a repo:

> This script finds the repository root with cd "$(dirname "$0")/..". It has been copied elsewhere. When it
> cannot see a repository, it degrades its baseline to an empty tree, reports "no changed documents" and exits
> 0 - that is a gate that cannot see a repository reporting success.

**That is the refusal he added on 10-31, and it caught me.** I had copied a gate out of its repository, which
is exactly the move that produces a false green, and the gate refused to participate. **A gate that cannot be
run outside its repository is a gate that cannot be fooled by being copied** — and I walked into it within a
day of documenting it.

**Second thing worth recording:** my first in-repo attempt said *"no changed docs vs HEAD — nothing to
check"* while the file sat untracked in front of me. The cause was mine, not the gate's: I had `git add`ed the
probe, so it was staged, and the gate's untracked source is `git ls-files --others`, which does not see a
staged file. **I unstaged it and the gate saw it immediately.** That is the same family as the tip-baseline
false green Aegis caught in himself on 11-02 — an instrument answering about a set I had accidentally shrunk.

## What I did not do

I did not edit the budget document, the backlog, or `docs-grandfather.txt`. I did not push. The scratch repo
and the probe copy are deleted; the primary checkout has only the pre-existing untracked bind script, and
`git diff HEAD` on the budget document is empty. `seats` is 9, `canonical_events` is 1,
`check-ledger-exists.sh` is exit 0.
