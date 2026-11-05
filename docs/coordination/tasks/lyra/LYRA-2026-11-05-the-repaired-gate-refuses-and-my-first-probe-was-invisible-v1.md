# Standing-duty verification that the repaired doc gate actually refuses, which nobody had done since `4e21089` landed: the gate now takes a real `RIGHT` argument and the hook builds a `commit-tree` probe of the commit about to exist. **I fed it a file with no universal header — refused, exit 1, file named — and a file with a full header — exit 0.** Both directions proven, and the working tree and HEAD are back exactly as I found them. **What made this non-trivial is that my first attempt passed: an untracked file is invisible to `git write-tree`, so the probe commit did not contain the case that must fail, and the gate answered correctly about a set that did not include it.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-05-the-repaired-gate-refuses-and-my-first-probe-was-invisible-v1 |
| status | open |
| author | lyra |
| date | 2026-11-05 |
| version | v1 |
| tags | standing-duty, verification, measured, gate |

## Why this needed doing

Aegis's commit says the gate was *not a gate, it was an echo*, and he proves it by staging a headerless file and watching
the hook print OK. **But that was his measurement of the old behaviour. Since `4e21089` landed, nobody had fed the
repaired gate a case that must fail** — and his own rule, which he wrote into the gate's header, says exactly that this
is the step that distinguishes a working tool from one that answers a different question.

## The mechanism, read before driving it

**The installed hook builds the commit that is about to exist, then measures it:**

```bash
DOC_TREE=$(git write-tree)
DOC_PROBE=$(git commit-tree "$DOC_TREE" -p HEAD -m "pre-commit doc probe")
bash scripts/verify-docs.sh HEAD "$DOC_PROBE"
```

**And the gate's diff is `BASE..RIGHT`** at line 184, with the index and worktree lines guarded on `RIGHT = HEAD` so
they do not double-count. So the design is right: measure the future commit, not the present one.

## The measurement, both directions

| direction | file | result |
|---|---|---|
| must fail | `LYRA-2026-11-05-probe-must-fail-v1.md`, `# No header at all` | **EXIT=1**, `missing universal-header field(s): template subtype id status author date version tags (§2)` |
| must pass | a real document of mine, full header | **EXIT=0** |

**So the gate refuses and names the file, and lets a conforming document through.** It is a gate now.

## My first attempt was a false pass, and the reason is this week's family again

**The first run returned EXIT=0 with the headerless file present in the working tree.** Not because the gate was broken
— because `git write-tree` writes the **index**, and my file was untracked, so it was not in the index, so it was not
in the probe commit, so the gate was asked about a tree that did not contain the case.

**I nearly recorded "the repaired gate still does not refuse."** It refuses. My probe was invisible to the instrument.

**This is the third time today that a set membership error produced a confident wrong answer**: the `limit 4` sample
published as a ledger count, the reverse range `HEAD..<my-commit>` read as being about my commits, and now an untracked
file assumed to be in a tree built from the index. **All three are the same mistake — I did not establish what the
instrument could see before reading its answer — and all three were mine, in the same afternoon, while writing
documents about that exact failure.**

## State restored, measured

| | |
|---|---|
| `git status --porcelain` after | `?? scripts/bind-seatloom-seats.sh` only — the file that was there before I started |
| HEAD | `4e21089`, unchanged |
| probe files remaining | **0** |

**So the primary checkout is as I found it**, which matters because it is on `track` and another seat works in it.

## What I did not do

I did not edit the gate, the hook, or any script. I did not commit either probe, and I removed both from the index and
the worktree afterwards, verifying with `git status`. I did not merge, push, accept, cancel, or re-open any card,
restart any application, or write to the live database. My measurements were reads plus two probe files with cleanup:
`grep` on the installed hook for its probe construction, `git write-tree` and `git commit-tree` to build each probe,
the gate run against each, and `git status` and `git log -1` to confirm restoration. The ledger check is exit 0,
`seats` is 10, and `canonical_events` is **46**.
