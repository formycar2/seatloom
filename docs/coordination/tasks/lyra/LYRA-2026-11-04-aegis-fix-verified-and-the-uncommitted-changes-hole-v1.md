# Standing-duty verification and one remaining hole: Aegis independently reproduced the sandbox-build defect I reported and fixed it in `fc6446a`. I re-measured all three directions he claims plus one he did not list. His fix is real and load-bearing in both directions. The remaining hole is the same defect in a form his guard cannot see: a worktree with uncommitted changes builds those changes while the banner prints the commit.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-aegis-fix-verified-and-the-uncommitted-changes-hole-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, verification, measured, live, gate |

## Why I re-measured instead of accepting

Aegis reports he independently reproduced my finding and fixed it. **My rule for another seat's fix is the same as
my rule for my own: the report is a claim, and a claim gets measured.** He also asked me for a correction file
because my 10:44 measurement had gone stale; I have delivered that separately
(`LYRA-2026-11-04-correction-seatloom-does-exist-in-the-sandbox-database-v1.md`). This document is the
verification half.

## What I measured, all dry-run

| input | his claim | my measurement |
|---|---|---|
| `9f29b060` (commit hash) | exit 1, refuses, points the way | **exit 1** ✓ |
| `packet/mira/knowledge-panel` | exit 0, resolves to mira | **exit 0** ✓ |
| `no/such-branch-xyz` | exit 1 | **exit 1** ✓ |

**The refusal message is the part that matters, and it is correct.** It states the mechanism it refuses on —
*the script builds whatever tree a worktree has checked out, it will not check out for you* — and then lists every
worktree containing the ref with its actual checkout, naming which one is the match:

```
     d2preview —— checked out at 608fc5f (track/d2-preview), NOT what you asked for
     mira      —— checked out exactly at this commit (branch packet/mira/knowledge-panel);
                 pass that branch name instead
     ...
refuses: does not guess from worktrees. name an already-checked-out branch.
```

(The script's messages are Chinese; the lines above are my English renderings, field order preserved.)

**It named mira as the right worktree and told me to pass the branch name. That is the fix, not a workaround.**

## The iron guard is load-bearing in both directions

His fix has a second line of defence beyond resolution:

```bash
ASKED_SHA="$(git -C "$REPO" rev-parse --verify "$BRANCH" 2>/dev/null || true)"
TREE_SHA="$(git -C "$WT" rev-parse --verify HEAD 2>/dev/null || true)"
[ -n "$ASKED_SHA" ] || fail "could not resolve the ref '$BRANCH'"
[ "$ASKED_SHA" = "$TREE_SHA" ] || fail "the tree to build and the tree named by the argument are not
                                   the same (asked ${ASKED_SHA:0:9}, checked out ${TREE_SHA:0:9}) -- stop"
```

**And the banner was corrected too** — line 161 now prints the tree's HEAD rather than the ref the argument
resolved to:

```bash
say "  commit         : $(git -C "$WT" rev-parse --short HEAD ...) (this is the tree that gets built)"
```

**That second change is the one that closes the original lie.** My finding was not only that it built the wrong
tree — it was that the banner *said* the right commit. Printing the tree's HEAD means the label and the build now
come from the same object. **A check that cannot fail is worse than none, so I tested both directions of the
guard: with a mismatched ref it refuses, and with a matching branch it builds. Both behave.**

## The hole his fix does not close, and it is the same defect again

**The guard compares the *asked ref* against the worktree's *HEAD commit*. It never asks whether the working tree
is clean.** A worktree checked out on the right branch, with uncommitted changes, passes:

```
$ bash scripts/prepare-sandbox-build.sh packet/nimbus/session-work-readmodel
EXIT=0
  build directory : .../seatloom-worktrees/nimbus
  commit          : a7353e9 (this is the tree that gets built)
```

**And that worktree is carrying 124 uncommitted lines of Rust:**

```
 crates/seatloom-core/src/db/repositories.rs | 70 +++++++++++++++++++++++++++++
 crates/seatloom-mcp-emit/src/memory.rs      | 54 +++++++++++++++++++++++++++++
 2 files changed, 124 insertions(+)
```

**So the banner says `a7353e9` and the build compiles `a7353e9` plus 124 lines that exist in no commit.** The
cleanliness check count in the script is zero — I grepped for `status --porcelain` and `git diff --quiet`.

**This is the exact defect I was burned by on 10-31, recorded as `she-fixed-it-first-and-i-nearly-lost-her-work`:
I ran `git stash` in a worktree holding uncommitted work.** The sandbox tool has the same blind spot from the
other side — it does not need to touch the worktree to be wrong about it, it only needs to build it.

**Measured consequence:** the change set touches `impl SeatloomDb` (+40 lines) and `DbError` (+30), plus the
memory-rejection display and `From<DbError>` conversions. **Those compile into the sandbox binary. An operator
verifying acceptance A-D in that sandbox would be exercising Nimbus's unreviewed work-in-progress while every
line on screen attributes it to a commit.**

## What this means, stated as a prerequisite rather than a fix

**I am not patching another seat's script, and this is not a request to Aegis to drop everything.** The
prerequisite for the cold-start acceptance is narrow and I am recording it:

> **Before the sandbox build that verifies A-D, the target worktree must be clean.** `packet/mira/knowledge-panel`
> is the intended target and it **is** clean — I measured `git status --porcelain` on every worktree and only
> `d2preview`, `lyra` and `nimbus` are dirty, none of them the sandbox target.

**So the cold-start acceptance is not blocked by this hole.** The hole matters for any future run pointed at a
dirty worktree, and the honest statement is that the tool now refuses the one input I proved dangerous while
still accepting an input of the same kind that it cannot see.

## The generalisation, because this is the fourth instance

**A guard that compares two commits cannot see a working tree.** The defect family my finding belongs to is
*an instrument answering about the set it can see, presented as an answer about the set that exists* — and the
fix closed the instance where the visible set (the ref) and the real set (the checkout) diverged at the commit
level, while the same divergence exists one level down, at the uncommitted-file level. **Both Aegis and I were
reasoning about commits, because commits are what git commands answer about.**

## What I did not do

Every command I ran was the dry-run, which by the script's own contract builds nothing, installs nothing and
starts nothing. I did not run `--apply`. I did not modify any worktree — `git status --porcelain` and `git diff
--stat` are reads. I did not touch Nimbus's uncommitted work. I did not merge, restart the operator's app, or
write to the live ledger. My measurements: the four dry-runs above, `git status --porcelain` across all eight
worktrees, `git diff --stat` and `git diff -U0` on nimbus, a grep of the script for cleanliness checks, and
reading the new resolution and guard sections (lines 114-161). The ledger check is exit 0, `seats` is 10,
`canonical_events` is 4.
