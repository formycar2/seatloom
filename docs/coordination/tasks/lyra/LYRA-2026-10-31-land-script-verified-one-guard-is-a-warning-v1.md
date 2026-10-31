# Standing-duty verification: he answered my 44-commit finding with two dry-run-by-default scripts, and I verified both — including the fact that one of the "two guards" is a warning that does not stop the merge

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-land-script-verified-one-guard-is-a-warning-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, verification, mutation, tooling, blocking |

## What landed

Two commits on the operator's packet branch, both answering the finding I filed last round — that 44 commits
of verified work were sitting off the integration line:

- **`55ecd85`** — `register-fleet-seats.sh` now defaults to **dry run**, and lists exactly who is missing.
- **`9aa50cf`** — a new `scripts/land-aegis-packet-branch.sh` that merges the packet branch into
  `track/infra-foundation` with `--ff-only` and pushes to origin. Default dry run.

**His 17:00 heartbeat says the quiet part out loud:** all of today's verified work is off the integration
line, 45 commits ahead.

## I ran both in dry-run and read the actual output

**`register-fleet-seats.sh`** now prints the four seats that exist and then names the five that are missing,
with the reason:

```
Seats already in the database:
  seat-networkorchestrator-001  networkorchestrator
  seat-ql-001  ql
  seat-stepaicloud-001  stepaicloud
  seat-vela-001  vela

Missing are these five (they have an FK, so without them they cannot be
dispatched to):
  seat-mira-001
  seat-nimbus-001
  seat-flux-001
  seat-onyx-001
  seat-lyra-001

When you are satisfied:  bash scripts/register-fleet-seats.sh --apply
```

**That is a better ask than the one I have been repeating for five rounds.** It names the five seats instead
of saying "1 of 6", and it names the FK consequence. **My ask was correct in substance and vague in shape**,
and he fixed the shape without being asked.

**`land-aegis-packet-branch.sh`** dry-run prints the state, proves the merge can fast-forward, counts the
files, and asserts there is no product code:

```
Current branch: packet/aegis/heartbeat-1800-un-gated-merges
track/infra-foundation                         052f6ff 10-30 17:37
packet/aegis/heartbeat-1800-un-gated-merges    40dffb1 10-31 17:02
origin/track/infra-foundation                  4c13ffc 09-30 08:49

Can fast-forward: the packet branch is 47 commits ahead of
track/infra-foundation, with no possibility of conflict.

49 files changed, 4872 insertions(+), 21 deletions(-)
No product code — documents, scripts and salvage only.

Will run:
  git merge --ff-only packet/aegis/heartbeat-1800-un-gated-merges
  git push origin track/infra-foundation
```

## The thing I have to report about the "two guards"

The commit subject says "dry run by default, two guards". **I verified both.**

- **Guard 1 (`--ff-only`)** is a real guard. If the merge cannot fast-forward, `git merge --ff-only` fails and
  the script exits 1 with "merge failed — switched to track but did not change it". It cannot produce a merge commit, and it
  cannot leave a half-merged track.
- **Guard 2 (no product code)** is **not a guard. It is a warning.** The code is:

```bash
if git diff --name-only "$TRACK".."$BRANCH" | grep -qE '^(crates|ui|src-tauri)/'; then
  echo "  WARNING: this batch contains product code (crates/ ui/ src-tauri/)."
  echo "     Per your rule, product code needs an acceptance card before it enters the integration line. Confirm this first."
else
  echo "  No product code — documents, scripts and salvage only."
fi
```

**There is no `exit` in that branch.** In `--apply` mode the script prints the warning and then runs
`git merge --ff-only` anyway. **So if the packet branch ever carries product code, the script will tell the
operator about it and merge it regardless.**

**I am reporting this rather than fixing it, for two reasons.** First, it is Aegis's script and his rule, and
the honest fix is his decision — a warning that a human is expected to act on is a legitimate design, as long
as it is *called* a warning. Second, **the subject says "two guards," and one of them does not guard.** That
is the exact discrepancy I have recorded all stretch: an artifact whose stated behaviour and its enforced
behaviour disagree.

**For today's payload it does not matter** — there is genuinely no product code in the 47 commits, I checked
the file list myself (docs, scripts, salvage, `.gitignore`). **But the next time this script runs, the batch
may not be clean, and then the "guard" will be a line of text.**

## What I did not do

**I did not run either script with `--apply`.** Both write shared state — one writes the live database, one
merges into track and pushes to origin. I have declined that action for six consecutive rounds, and the
reason has not changed: **I destroyed the live database by taking an unsupervised infrastructure action.**

## The ask

```bash
bash scripts/register-fleet-seats.sh --apply
bash scripts/land-aegis-packet-branch.sh --apply
```

Two commands, both dry-run-verified, both listing exactly what they will do. **And either change guard 2 from
a warning into a refusal, or say in the subject that it is a warning** — because a script that says "two
guards" and ships one is the shape I have been measuring all day.
