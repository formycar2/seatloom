# Standing-duty verification: he found a script that says the opposite of what it measures, and confirmed a second defect in it — then my own measurement turned up an anomaly I cannot explain and am recording as unexplained rather than guessing

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-02-his-two-findings-hold-and-i-have-one-i-cannot-explain-v1 |
| status | open |
| author | lyra |
| date | 2026-11-02 |
| version | v1 |
| tags | standing-duty, verification, unexplained, honest |

## His first finding: a heading that claims a number and a loop that measures one

I reproduced it exactly. `bash scripts/register-fleet-seats.sh` in dry-run, with all nine seats present:

```
Seats currently in the database:
  seat-flux-001  flux
  seat-lyra-001  lyra
  seat-mira-001  mira
  seat-networkorchestrator-001  networkorchestrator
  seat-nimbus-001  nimbus
  seat-onyx-001  onyx
  seat-ql-001  ql
  seat-stepaicloud-001  stepaicloud
  seat-vela-001  vela

Missing are these five (FK, so they cannot be dispatched to until registered):
                                     <- empty
If that is correct, run:  bash scripts/register-fleet-seats.sh --apply
```

**Nine seats listed, then "missing these five", then nothing, then a command to run.** The cause is one line —
`scripts/register-fleet-seats.sh:45` is a hardcoded string, and the loop below it prints only the seats whose
count is actually zero. **When nothing is missing the heading still says five.**

**His reading of the harm is right and it is the important part:** `ON CONFLICT DO NOTHING` makes a re-run
harmless, so the damage is not data corruption — it is that **an empty list under an assertion reads as a
conclusion**, and here the assertion points at "missing", which he notes he had previously judged the more
dangerous direction. **It is the same family he has been recording: an empty list wearing a conclusion's
clothes.**

**He did not fix the script in this commit** — the commit is documentation only. I am recording that plainly,
because a documented defect that still fires is still firing, and the heading will mislead the next person who
runs the dry-run.

## His second finding, which I verified and which is more serious

`ae1bb07` renamed the providers — `stepplan`/`stepplan-anthropic` → `sp-personal`,
`stepplan-wash`/`stepplan-wash-anthropic` → `sp-team` — and the live `seats.default_model` was updated to
match. **But `scripts/register-fleet-seats.sh` still INSERTs the old names.** I confirmed both halves:

```
live DB:  sp-personal/step-5-preview (5 seats), sp-team/step-5-preview (4 seats)
script:  'stepplan-wash-anthropic/step-5-preview', 'stepplan/step-5-preview'
```

**So the script would write provider names that no longer exist.** His harm analysis is correct and I verified
the mechanism: the five core seats already exist and `ON CONFLICT DO NOTHING` will not change them, so
`--apply` cannot corrupt them today. **But the script is the rebuild path, and a rebuild would write dead
names.** That is the same shape as the heading defect — a stale claim inside a tool that looks current.

**And it supersedes my own 11-02 finding about stale model provenance.** I reported that the script registered
`stepplan/step-5-preview` against the operator's 10-16 requirement. **The provider names have since been
renamed by the operator's own instruction, so my finding was about a state that no longer exists, and my
conclusion ("the DB still has the old value") is now false in a way I did not anticipate.** The rename is the
operator's decision, made after I reported — which is what I asked for, and it makes my measurement historical
rather than current.

## The anomaly I cannot explain, and am not going to guess at

**My own running session reports `PI_PROVIDER=sp-team`, but the database says my seat is `sp-personal`:**

```
env (this session): PI_PROVIDER=sp-team, PI_MODEL=step-5-preview
DB:                 seat-lyra-001 -> sp-personal/step-5-preview
```

**I cannot explain it, and here is exactly what I measured:**

- `git log --all -S "sp-team" -- scripts/` returns exactly one commit, **`ae1bb07`, dated 11-02**. The string
  `sp-team` did not exist in this repository before today.
- My session file is named `2026-10-21T14-04-35-...` — **the session started on 10-21**, twelve days before the
  name existed.
- The session file's mtime is 17:35 today, but it is appended to every round, so mtime is not the start time.
- Nothing in the repo sets `PI_PROVIDER` at all — `grep -rn PI_PROVIDER scripts/ crates/` returns nothing.

**So either the harness re-resolves the provider per round and the name is current, or the env is stale and
the name is impossible, or my reading of when the session started is wrong.** **I do not know which, and the
honest answer is that I do not know.** A tidy explanation here would be a guess, and a guess is the failure I
have spent this stretch documenting in other people's work — including, last round, in my own.

**What I am not doing:** I am not concluding that my seat is misconfigured, that the DB is wrong, or that the
rename missed a seat. **All three would be plausible and none is measured.**

## What I did not do

I did not edit the registration script, the rename script, or any seat's model. I did not run
`rename-stepplan-providers.sh` in any mode — it is a live-database write and the script itself says the
operator runs it, not a seat. I did not change `PI_PROVIDER` or restart anything. My measurements were the
script's own dry-run output, `psql` selects, `git log -S`, `git show`, and `env`. `seats` is 9,
`canonical_events` is 1, `check-ledger-exists.sh` is exit 0.
