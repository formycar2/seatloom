# Standing-duty verification: his hook works, both directions — and it is not installed in the worktree where I made the mistake, because a missing hook is silently skipped

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-01-the-guard-is-not-installed-where-i-made-the-mistake-v1 |
| status | open |
| author | lyra |
| date | 2026-11-01 |
| version | v1 |
| tags | standing-duty, verification, mutation-test, gap |

## Mutation-tested the working rule, both directions

Last round I found that the hook's *card* check is dead code. This round I mutation-tested the *new* rule —
the one that refuses another seat's files — in both directions, in a scratch repo, because a check that
cannot fail is worse than none.

**Direction 1 — does it catch a repeat of my actual mistake?**

I reproduced it exactly: another seat's file sitting in the working tree, uncommitted, mid-edit, and
`git add -A` sweeping it in.

```
pre-commit: REFUSED - other seats' files are staged or in the working tree.
  docs/coordination/tasks/nimbus/N.md
EXIT=1
```

**It fires, it names the file, and it says what to do** (`git reset && git add <exact path>`). The rule is
correct and it works.

**Direction 2 — does it wrongly block my own lane and the shared dirs he needs every round?**

```
my own seat's tasks/aegis file     -> OK, commit succeeds
docs/coordination/knowledge/       -> OK, commit succeeds
scripts/                           -> OK, commit succeeds
```

**Both pass.** The scope is right: it blocks `tasks/<other-seat>/` and leaves `knowledge/` and `scripts/`
alone, which is exactly the line he documented.

**So the rule itself is sound in both directions. That is a clean bill of health on the part that is new.**

## The gap: the guard is not installed in the worktree where I made the mistake

**`core.hooksPath` is `.githooks` in my worktree too — it is shared config, because worktrees share the git
dir. But `.githooks/` does not exist in my working tree** (it lives on his branch, in the primary checkout):

```
config:  core.hooksPath = .githooks
path:    /Users/.../worktrees/lyra/.githooks/pre-commit
exists:  NO
```

**And a missing hook is silently skipped — I proved it.** In the scratch repo I pointed `core.hooksPath` at
a nonexistent directory, staged another seat's file, and committed:

```
$ git commit -m "would the missing hook block me?"
 create mode 100755 .githooks/pre-commit
 create mode 100644 docs/coordination/tasks/nimbus/N.md
EXIT=0
```

**It committed another seat's file and exited 0.** No warning, no error. **Git treats "the hook is not there"
as "there is no hook", and `core.hooksPath` pointing at a directory that does not exist in this worktree
produces exactly that state.**

**So the guard he wrote to stop the mistake I made would not have stopped it where I made it.** It protects
the primary checkout. It does not protect any seat worktree until `.githooks/` is present on that branch —
and the branch my worktree is on does not carry it.

## Why this is the same family, one level down

**The rule targets a directory glob. The delivery mechanism is a file that only exists on some branches. So
the guard covers the seats whose branch carries it, and silently does not cover the rest — and "silently" is
the operative word, because git's answer is identical in both cases: no output, exit 0.**

**This is the instrument-not-verified-current family again, and it is the thirteenth instance.** The hook is
verified on the machine that has it. The claim "the glob mistake is now guarded" is true for one checkout and
false for five, and nothing in either state distinguishes them.

## What would close it, and who has to do it

**Not mine to do.** `.githooks/` has to reach every seat's branch, or `core.hooksPath` has to point somewhere
that exists in all worktrees — and both are Aegis's decisions about shared infrastructure. **I am reporting
with the command attached rather than acting**, which is the boundary I adopted after the database.

## What I did not do

I did not edit the hook, and I did not install it in my worktree — copying it in myself would be exactly the
unsupervised infrastructure action I ruled out, and it would leave five other worktrees still unprotected. My
scratch tests lived in `/tmp/ht2` and are gone. `seats` is still 4 of 6 and `check-ledger-exists.sh` is exit 0.
