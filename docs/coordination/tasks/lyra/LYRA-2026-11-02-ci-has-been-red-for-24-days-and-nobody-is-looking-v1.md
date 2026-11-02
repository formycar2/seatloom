# Standing-duty verification: the "red gate" I have been reporting for two rounds is the small one — gitlab's track has not been pushed since 10-09, it is 830 commits behind, and the docs gate on THAT branch has 287 violations. The CI has been red for 24 days and nobody has been looking at it.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-02-ci-has-been-red-for-24-days-and-nobody-is-looking-v1 |
| status | open |
| author | lyra |
| date | 2026-11-02 |
| version | v1 |
| tags | standing-duty, measurement, ci, drift |

## What I measured, and why it changes the picture

I have spent two rounds reporting that the docs gate is red on `track/infra-foundation` because the operator's
budget document has no header. **That is true, and it is one violation.** Tonight I measured the gate against
the branch CI actually runs on, and the scale is different by two orders of magnitude.

**GitLab is reachable right now** — `git ls-remote gitlab HEAD` exits 0, which it has not reliably done all
stretch. So I fetched and compared:

```
gitlab track tip: b09ec6e  10-09 18:46
local  track tip: 4663f7b  11-02 19:15
git rev-list --left-right --count gitlab/track...track  ->  gitlab ahead 0, local ahead 830
```

**`track/infra-foundation` has not been pushed to GitLab since 10-09. 830 commits of work — every fix, every
verification, every document written in the last 24 days — exist only on this machine.**

**And the gate on the branch CI does see:**

```
$ bash scripts/verify-docs.sh gitlab/track/infra-foundation
EXIT=1
287 violations
```

**287 documents fail the gate on GitLab's track.** Not one file, not a recent regression — the entire corpus
from 10-09 onward, including files dated 10-10 and 10-16 that were written *after* the last push. **The
budget document is not even on that branch** (`git cat-file -e` fails), so the failure I have been reporting
twice is invisible to CI.

## Why this is the most important thing I have measured this stretch

**Everything I have reported about "the gate is red" has been a local phenomenon, and I presented it as a CI
state.** Aegis said it correctly on 11-01 — the gate is red *in a place nobody can see*, because GitLab is
unreachable from this machine. **I did not extend that far enough: it is not that the gate is red and hidden,
it is that the gate has not been run on 830 commits of work at all, and when it does run it finds 287
violations rather than 1.**

**The number 287 vs 1 is the whole finding.** I have been reporting a one-file header problem for two rounds
while a 287-file corpus sits unpushed and unmeasured. **A reader of my last two delivery documents would
conclude the problem is nearly solved. It is 1/287th of the problem.**

## What this means for the asks, concretely

**The operator's push is not a sync step — it is the difference between "one document needs a header" and
"287 documents need headers or grandfather entries."** Those are different amounts of work by two orders of
magnitude, and the decision about which is a different decision.

**And it reframes every green check in the fleet.** `check-cards-clickable`, `check-verdict-refs`,
`check-verdict-defects`, `check-card-verifications` all exit 0 — against an empty ledger, on a branch CI has
never seen. **The instruments are honest about the set they can see, and the set they can see is 830 commits
away from the one CI sees.**

## What I did not do

I did not push to GitLab — pushing track is the operator's action and it carries 830 commits, which is exactly
the kind of unsupervised infrastructure write I ruled out after the database. I did not grandfather anything.
I did not edit any document. My measurements were `git ls-remote`, `git fetch`, `git rev-list --count` between
the two refs, `git cat-file -e` for the budget doc on the remote ref, and the gate's own run against that ref.
`seats` is 9, `canonical_events` is 1, `check-ledger-exists.sh` is exit 0.
