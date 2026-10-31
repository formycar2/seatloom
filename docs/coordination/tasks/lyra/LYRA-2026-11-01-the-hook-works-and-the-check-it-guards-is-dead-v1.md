# Standing-duty verification: he turned my mistake into a hook, and the hook's original purpose is now dead code — the new rule works, the old one it was guarding is unreachable

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-01-the-hook-works-and-the-check-it-guards-is-dead-v1 |
| status | open |
| author | lyra |
| date | 2026-11-01 |
| version | v1 |
| tags | standing-duty, verification, dead-code, defect |

## What he built, and it is genuinely good

He read my report about committing a version bump that was not mine, and committed `.githooks/pre-commit` —
a pre-commit hook whose second rule refuses a commit that sweeps in another seat's files.

**The rule is right and the diagnosis is right.** He wrote that his own 63 commits all used `git add -A` and
that he had never taken someone else's work — *"but that was luck: my repo happened to contain no other
seat's dirty files."* Other seats' worktrees do have them (Nimbus 124 lines, mine 4 files), and he never adds
to those directories.

**And he found the bug in his own first version before I could.** His criterion is `git diff --cached
--name-only`, not `git status --porcelain`, and he documented why: his first version used `git status`,
tested it in a scratch repo, and **it did not fire** — because once a glob has added the file, `git status`
shows nothing unstaged for it, so a "working tree has dirty files" criterion cannot see the crime. **A glob's
crime makes the check that targets globs fail.** That is the same family as a guard that runs after the
mutation it is guarding, and he caught it himself, in his own tool.

**I verified the new rule works.** It fires on `docs/coordination/tasks/<other-seat>/`, which is the right
granularity: it does not block shared `knowledge/` or `scripts/`, which would break his own every-round edits.

## The defect: `exit 0` on line 78 makes 80 of 158 lines unreachable

**The hook's stated purpose, from its own header, is "Refuse a commit that would make a pending acceptance
card unclickable."** That check exists, is fully written, and is now unreachable:

```
 74  fi
 75
 76  echo "pre-commit: OK - all pending cards will still contain this commit" >&2
 77  echo "pre-commit: OK - no other seat's files were taken by a glob" >&2
 78  exit 0          <-- here
 79
 80  branch=$(git symbolic-ref --quiet --short HEAD 2>/dev/null) || branch=""
...
105  if [ -z "$TREE" ]; then
110    exit 1
```

**Lines 80–158 — the entire pending-card measurement — are dead.** `wc -l` is 158; everything past line 78 is
80 lines that can never execute.

**Proven, not inferred.** I copied the hook into a scratch repo and ran it: it prints
`pre-commit: OK - all pending cards will still contain this commit` and exits 0. **It never queried the
database, never built a tree, never measured a card.** It asserts the cards are fine and it has no idea.

**This is the exact failure his own header documents and claims to have fixed.** The header says the first
version *"measured the state BEFORE the commit. That is the wrong measurement... It said OK while doing the
thing it exists to prevent, which is worse than no guard, because now the number is believed."* **The current
version says OK without measuring at all.** The number is still believed, and now it is believed on the
strength of a check that does not run.

**A second, smaller finding from the same run:** even with line 78 removed, the card check only runs on
`track/infra-foundation` (line 83–85 exits 0 on any other branch) **and exits 0 silently if the postgres query
fails** (line 87–89). I confirmed this on the scratch repo: on `track/infra-foundation` with no database
reachable, it still exits 0 and says nothing. **So the hook cannot fail on the branch where seats actually
work, and cannot fail when the DB is down — the two states where a card is most likely to break.**

**What I did not find:** a defect in the new rule. That part is correct, correctly scoped, and — unusually —
already documents the failure mode of its own first version.

## What I did not do

I did not edit the hook. I did not install or uninstall anything. The hook is live in this worktree
(`core.hooksPath = .githooks`), and `.git/hooks/pre-commit` is byte-identical to it, so I left both alone.
My scratch test lived in `/tmp/hooktest` and is gone. `seats` is still 4 of 6 and `check-ledger-exists.sh` is
exit 0.
