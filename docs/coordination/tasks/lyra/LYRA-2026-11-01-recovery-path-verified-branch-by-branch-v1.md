# Standing-duty report: the recovery path is healthy and I verified it branch-by-branch rather than trusting the bundle's ref count. Four of five seat branches match their tips exactly; the fifth is correctly excluded.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-01-recovery-path-verified-branch-by-branch-v1 |
| status | open |
| author | lyra |
| date | 2026-11-01 |
| version | v1 |
| tags | standing-duty, verification, backup, measurement |

## What I verified, and why this shape

My protocol says a pass with numbers is a result. Last few rounds I verified the backup *tool*; this round I
verified the backup *content*, branch by branch, because a bundle that "has 60 refs" tells you nothing about
whether the refs you care about are at the right commits.

**The recovery path, from the committed version:**

```
HEAD bundle_args count: 3     working tree: 3     dirty: 0
bash scripts/backup-unpushed-branches.sh  ->  EXIT 0
newest bundle: git-20261101-024927.bundle
git bundle verify -> okay, "The bundle records a complete history."
orphan refs carried: 12
```

**And the branch-by-branch check, exact-match on ref names rather than a grep pattern:**

| Branch | In bundle | Matches tip |
|---|---|---|
| `packet/aegis/heartbeat-1800-un-gated-merges` | yes | **`95b8738` — exact match** |
| `packet/mira/knowledge-panel` | yes | **`c9a7fa7` — exact match** |
| `packet/nimbus/session-work-readmodel` | yes | **`a7353e9` — exact match** |
| `packet/flux/intent-verdict` | yes | **`5806317` — exact match** |
| `packet/vela/washing-pipeline` | yes | **`dd41974` — exact match** |
| `packet/onyx/topic-opened-019` | **no** | **correctly excluded** |

**The one exclusion is correct, and I verified why rather than assuming:** onyx's branch is **0 commits ahead
of origin and 0 ahead of gitlab** — it is fully pushed to both remotes, so it is not at risk and the tool
correctly leaves it out. That is the tool working as designed, not a gap.

## The near-miss, recorded because it is the recurring failure

**My first check used a grep pattern and reported three branches as "NOT in bundle" — including
`packet/flux/intent-verdict`, which is the single most at-risk branch in the repository (29 commits ahead of
gitlab, nothing on origin).** That would have been a serious false alarm, and it came from `grep " $b$"`
against a `list-heads` output where the ref column is `refs/heads/<name>` — the pattern anchored on the wrong
column.

**I re-ran it with `awk` doing exact field comparison and got the true answer: it IS in the bundle, at its
exact tip.** The difference between "three branches are unprotected" and "four of five match, one correctly
excluded" was the instrument, not the repository.

**This is the twelfth-plus instance of one family, and the first where the wrong instrument would have
produced a false alarm rather than a false all-clear.** Both directions matter: a check that says "fine" when
it is not, and a check that says "broken" when it is not. The second wastes the operator's attention and
teaches him to discount my reports — which is the more corrosive failure over time, because it spends
credibility rather than safety.

## The live asks, unchanged

**My finding from last round is still unanswered and still live:** the comment in `register-fleet-seats.sh`
still asserts the `--include=*.md` / `.zhs.md` mechanism, which I measured as false, and I re-verified the
underlying numbers this round — `stepplan-wash-anthropic` in 20 files, `stepplan-wash/step-5-preview` in 8,
`stepplan/step-5-preview` in 9 at HEAD. **None of the three candidate mechanisms reproduces a zero-hit
result.**

**And the two operator commands remain:**

```bash
bash scripts/register-fleet-seats.sh --apply      # five core seats unregistered; the FK blocks all dispatches
bash scripts/land-aegis-packet-branch.sh --apply   # carries today's work into the integration line
```

## What I did not do

I did not edit any script or document in the primary checkout. I did not run `--apply` on either operator
script. The bundle I verified was produced by the committed tool; I did not delete any of the existing
bundles. `seats` is still 4 of 6 and `check-ledger-exists.sh` is exit 0.
