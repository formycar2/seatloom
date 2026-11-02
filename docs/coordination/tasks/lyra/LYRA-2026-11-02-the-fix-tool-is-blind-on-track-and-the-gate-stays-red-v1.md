# Standing-duty verification: the gate is red on track for the second round running, and the tool that exists to fix it is blind to the file — because it computes its baseline as `merge-base HEAD track/infra-foundation`, which is HEAD itself when you are standing on track

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-02-the-fix-tool-is-blind-on-track-and-the-gate-stays-red-v1 |
| status | open |
| author | lyra |
| date | 2026-11-02 |
| version | v1 |
| tags | standing-duty, defect, tool, gate-red |

## The state, measured

**The docs gate is red on `track/infra-foundation`, for the second consecutive round:**

```
$ bash scripts/verify-docs.sh
  FAIL docs/coordination/knowledge/BUDGET_POOL_MODEL_MAP.md:
  missing universal-header field(s): template subtype id status author date version tags (section 2)
EXIT=1
```

**One violation, and it is the operator's own file.** It was added at 17:50 and touched again by two later
commits (`2d5bacd`, `5b071a4`), neither of which added a header. The gate is right.

**And the tool that exists to fix exactly this is blind to it:**

```
$ bash scripts/fix-doc-headers.sh
baseline (same as CI) = 5b071a4

Dry run finished. Add --apply to actually change things.
EXIT=0
```

**"Dry run finished" with no file found.** It prints a baseline and reports success while the file it should
fix sits one directory away, committed, failing the gate.

## The cause, and it is one line

```bash
BASE="$(git merge-base HEAD track/infra-foundation 2>/dev/null)"
```

**When you are standing on `track/infra-foundation`, the merge-base of HEAD and track is HEAD.** So:

```
fixtool BASE: merge-base(HEAD, track) = 5b071a4 = HEAD
```

`git diff --name-only --diff-filter=ACMR "$BASE" HEAD -- docs` is then empty by construction, and the fallback
(`git diff` with no base, i.e. unstaged changes) is also empty because the file is committed. **Neither source
can see a file that is already in HEAD.**

**I proved it cannot be reached by argument:** the script takes no base parameter, so
`bash scripts/fix-doc-headers.sh origin/track/infra-foundation` is silently ignored — it printed the same
`5b071a4` and found nothing.

## Why the gate catches it and the fix tool does not

**The two tools compute different baselines, and the gate's is strictly better:**

```
gate:    origin/track/infra-foundation = 4080fa0   (11 commits behind local track)
fixtool: track/infra-foundation        = 5b071a4   (= HEAD)
```

**The gate's candidate list starts with `origin/track/infra-foundation`** (`verify-docs.sh:154`), which is
behind local track, so `git diff origin/track HEAD -- docs` includes the file. **The fix tool's hardcoded
local-track merge-base equals HEAD, so its diff is empty.**

**So the tool that is supposed to repair the gate is blind in exactly the situation where the gate is most
likely to be red — standing on the integration line with a bad commit already in it.** That is the situation
right now, and it is the situation the tool was built for.

## This is the same defect I reported on 10-31, and it is not fixed

I documented this shape on 10-31 as *"the header-fix tool could not see an uncommitted doc"* — then it was
about untracked files. **This is the same tool, a different blind spot, and the earlier fix did not cover
it.** The general form is: **the tool's set is computed by a diff, and any file outside that diff is invisible
to it while remaining fully visible to the gate.** Untracked was one way to be outside the diff; already
committed-to-HEAD is another.

**The instrument-not-verified-current family, fourteenth instance, and this one has a cost:** the operator has
a red gate, a tool that claims to fix it, and a tool that reports success while doing nothing. **A fix tool
that says "dry run finished" when it found nothing is worse than no fix tool**, because the operator now
believes the path to green is available.

## What would close it, and who owns it

**The fix is the gate's own candidate list**: try `origin/track/infra-foundation` first, then the MR base, then
fall back. **It is Aegis's script and it is shared tooling, so it is his to change and I am reporting it rather
than editing it.**

**The immediate unblock for the operator is smaller than the tool fix:** the file needs a header. It is his
document and the header fields are his to supply — I am not writing his knowledge document's metadata.

## What I did not do

I did not edit `fix-doc-headers.sh`, `verify-docs.sh`, or `BUDGET_POOL_MODEL_MAP.md`. I did not run
`fix-doc-headers.sh --apply`, which would have written nothing anyway. My measurements were the two tools' own
output, `git rev-parse` on both baselines, `git rev-list --count` between them, and `grep` on both scripts'
baseline logic. `seats` is 9, `canonical_events` is 1, `check-ledger-exists.sh` is exit 0.
