# Standing-duty finding and correction to my own ruling: my ruling asked for "an isolated build from the pinned commit". I fed the build tool that pinned commit, and it did not refuse — it resolved to a worktree **868 commits behind** that does not contain the fix, and labelled the whole build with the commit I asked for. Separately, the sandbox database does not contain the operator's landing project, so the persisted-state half of the same acceptance cannot be exercised there either.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-the-sandbox-build-tool-accepts-a-commit-and-builds-a-different-tree-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, correction, measured, live, gate |

## What my ruling asked for, and whether the tool can do it

Last night's ruling (`LYRA-2026-11-03-2100-cold-start-takeback-ruling.md`, delivered as `dd34a2b`) said in
section 3.3:

> The isolated build: take a **pinned commit** (`packet/mira/knowledge-panel` @ `9f29b060`), build it, install it
> as the sandbox app, and verify A-D there. ... The sandbox database must really contain those two projects with
> different saved tabs, or A and B cannot be verified at all.

**I asked for a build from a pinned commit. Tonight I checked whether the tool that builds sandboxes can be given
one. It cannot, and the way it fails is worse than refusing.**

```
$ bash scripts/prepare-sandbox-build.sh 9f29b060          # dry-run, changes nothing
EXIT=0
  ref asked for  : 9f29b060
  build directory: .../seatloom-worktrees/d2preview
  commit printed : 9f29b06
  vs track       : 31 ahead / 2 behind
```

(The tool's own banner labels are Chinese; the field names above are my English renderings of them, kept in
the tool's order.)

**Exit 0. It names the commit I asked for. And it names a build directory that is not on that commit.**

## The defect: the label is resolved from the argument, the build from the checkout, and nothing compares them

`scripts/prepare-sandbox-build.sh` resolves the build directory in two passes:

```bash
# pass 1 — match by BRANCH NAME
if [ "$(git -C "$d" rev-parse --abbrev-ref HEAD)" = "$BRANCH" ]; then WT="$d"; break; fi
# pass 2 — match by ANY resolvable ref
git -C "$d" rev-parse --verify "$BRANCH" >/dev/null && { WT="$d"; break; fi
```

**A commit hash can never equal a branch name, so pass 1 always misses. Pass 2 accepts any object, so it
matches the first worktree that merely *contains* the commit object** — and then, decisively:

```bash
say "  commit printed : $(git -C "$WT" rev-parse --short "$BRANCH" 2>/dev/null || echo '?')"  # line 120
( cd "$WT/ui" && pnpm build ) 2>&1 | tail -4                                                 # line 192
```

**Line 120 prints the commit resolved *from the argument*. Line 192 builds the *checked-out working tree*. The
script never checks anything out — I grepped for `checkout`/`switch`/`reset` and there is not one.** So the
printed commit and the built tree are two independent facts, and the script presents the first as a description
of the second.

**What actually gets built:**

| | |
|---|---|
| pinned commit I passed | `9f29b060` (Mira's branch tip) |
| worktree chosen | `d2preview`, checked out at `608fc5f` on `track/d2-preview` |
| commits the build is missing | **868** |
| does it contain the cold-start fix `bd0cabb`? | **NO** |

**So a sandbox "from `9f29b060`" would be built from a tree that predates the cold-start mechanism fix by 868
commits, while every line of its banner says `9f29b06`.** The operator would open it, exercise A-D, and be
verifying a build that never had the mechanism.

**The same argument as a branch name works correctly:**

```
$ bash scripts/prepare-sandbox-build.sh packet/mira/knowledge-panel
  build directory: .../seatloom-worktrees/mira
  commit printed : 9f29b06
```

Pass 1 matches by name and lands on the right worktree. **So the tool is correct on the input it documents
(`<branch-name>`, or `--from-dispatch <dispatch-id>`) and silently wrong on an input it never documented but never refuses.**

## Why this is my error and not only the tool's

**I wrote a ruling that asked for "a build from a pinned commit" without checking that the build tool accepts
commits.** That is the same defect I recorded two days ago as *my own ask pointed at an unclonable script* — **an
ask is a claim, and a claim gets measured like any other.** I measured the ask's conclusion (the ruling is
correct) and not the ask's precondition (the tool can be pointed at a commit).

**And the tool's part is the same failure family again: an instrument answering about the set it can see,
presented as an answer about the set that exists.** It can see *the commit the argument names*; the set that
exists is *the tree that gets compiled*. It printed the first, labelled the second with it, and exited 0.

## The second half: the sandbox cannot exercise the persisted state either

The ruling's other requirement — "the sandbox database must really contain those two projects with different
saved tabs" — is also not met, and I measured it:

```
projects in seatloom_sandbox that also appear in the operator's persisted state:
  sf-code/storage                      in sandbox: 1
  ql_delivery                          in sandbox: 1
  project_delivery/QL                  in sandbox: 1
  seatloom                             in sandbox: 0     <- the operator's landing project
  project_delivery/networkOrchestrator  in sandbox: 0
  project_delivery/stepaicloud           in sandbox: 0
```

**The operator's persisted landing project — `seatloom`, the one I read off his disk last night, the one whose
saved tab is `live-seats` — does not exist in the sandbox database.** And the sandbox app runs under a separate
key prefix (`SANDBOX_KEY_PREFIX="slsandbox:"`, line 52), so it does not share the persisted state I read.

**Therefore the sandbox is a permanently cold start with no saved tabs to restore, and the restore-tab mechanism
has never been exercised in the sandbox at all.** Verifying A-D there would pass vacuously — a mechanism with
nothing to restore always restores correctly.

## What this means for the acceptance

**Both halves of the build verification my ruling demanded are unmet, and now I know why rather than listing them
as gaps:**

1. **Point it at the branch name, not the commit.** `prepare-sandbox-build.sh packet/mira/knowledge-panel`
   resolves to Mira's worktree at `9f29b06` (31 ahead, 2 behind track). That is the correct build. **Passing the
   commit hash is the one input that silently builds the wrong tree.**
2. **The sandbox database needs the operator's real projects before A/B mean anything.** Until `seatloom` and its
   saved `live-seats` tab exist in `seatloom_sandbox`, the cold-start restore has nothing to restore there and a
   pass is not evidence.

**Both are prerequisites I should have measured before writing "isolated build from the pinned commit" into a
ruling.** The ruling's conclusion held — the defect is real, I proved it live from the operator's disk last
night — but its verification recipe did not, and I published the recipe anyway.

## What I did not do

I ran only the dry-run, which by the script's own documented contract (dry-run is the default; without
`--apply` it never builds, never installs and never starts a process) builds nothing, installs nothing and
starts nothing. I did not run `--apply`. I did not modify the
sandbox database or the operator's persisted state. I did not edit the delivered ruling; this document corrects
it. My measurements were `prepare-sandbox-build.sh 9f29b060` (dry-run, exit 0), the same script with the branch
name, `git merge-base --is-ancestor`, `git rev-list --count 608fc5f..9f29b060`, `git cat-file -e`, `git branch
--contains`, a grep for `checkout|switch|reset` in the script, and `psql` against `seatloom_sandbox`. The ledger
check is exit 0, `seats` is 10, `canonical_events` is 4.
