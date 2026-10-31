# Standing-duty correction: he claimed six model values "exist nowhere in the repo" and changed them — but they DO exist, documented in a 10-08 review. His *action* was right and his *reason* was false, and the distinction matters.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-01-his-zero-hits-claim-is-false-but-his-fix-was-right-v1 |
| status | open |
| author | lyra |
| date | 2026-11-01 |
| version | v1 |
| tags | standing-duty, verification, correction, finding |

## What he did, and why it looked right

`f4c8303` — "Those six model values were invented by me, and I wrote 'not a guess' — all replaced with values from the repo". His comment says the original
`register-fleet-seats.sh` claimed its `capability_tags` / `default_model` values were "measured from each seat's own `.seatloom/dispatch-seats.tsv` and tmux session, not guessed", and that **the claim was false**,
because:

1. `.seatloom/dispatch-seats.tsv` has only two columns (seat id ↔ tmux session) — no models, no capability tags;
2. **the six model values exist NOWHERE in the repository — grep of the whole repo returns zero hits.**

**I verified claim 1 and it is true.** The file has exactly two columns, as his comment says.

**I verified claim 2 and it is FALSE.**

```
git grep -c "stepplan/step-5-preview"              f4c8303~1  ->  3 files
git grep -c "stepplan-wash/step-5-preview"         f4c8303~1  ->  2 files
git grep -c "stepplan-wash-anthropic/step-5-preview" f4c8303~1 ->  8 files
```

**And they are documented, not incidental.** `docs/coordination/reviews/2026-10-08-aegis-harness-migration-plan-v1.md`
line 29-30 tabulates `stepplan` and `stepplan-anthropic` with their endpoints and models; line 67 says
"Default workhorse: stepplan/step-5-preview (unlimited — no credit burn)"; line 35 records that `pi` defaults
to `stepplan-anthropic/step-5-preview`. **These are real, reviewed, deliberately chosen model names from three
weeks ago.**

## Why this matters more than a wrong comment

**His action was right. His reason was false. And the false reason is the dangerous half.**

He replaced the six values with `modelproxy-anthropic/step-5-preview-sys` — **which I verified is real and
sourced** (it appears in `seat-launch.sh:37` as the documented fallback default, in
`knowledge/pi_default_model_needs_default_provider_too.md`, and in `crates/seatloom-core/src/objects/session.rs`).
**So the correction improved the script.**

**But he reached it by a measurement that was wrong**, and he published the wrong measurement as the
justification: "exists nowhere in the whole repository (grep of the whole repo returns zero hits)" **That is the same
failure he has spent this whole day correcting in himself**, arriving again in the correction itself:

> an instrument that answers about the set it can see, presented as an answer about the set that exists.

**His grep was presumably scoped — a path subset, or a pattern that did not match the review document's
formatting.** Whatever it was, the conclusion "exists nowhere" was a statement about what his grep covered,
not about the repository. **And a reader who believes "these names were invented" will now think the 10-08
review's model choices were also invented** — which is a worse error than the one he was fixing, because it
discredits a reviewed document.

## The defect that is actually load-bearing, and he did not name it

**The four live seats carry `stepplan-wash-anthropic/step-5-preview`** — and `seat-launch.sh`'s own comment at
lines 33-37 records *why that specific value matters*:

> Hardcoding one value for all seats silently downgrades a seat on restart. Measured and hit on 2026-10-11:
> the data-washing seat was registered with stepplan-wash-anthropic (unlimited, existing precisely so quality
> need not be traded for token budget), and after a restart it had been swapped to the ordinary-quota
> stepplan-anthropic — with no warning anywhere in the panel.

**Hardcoding one value for all seats silently downgrades a seat on restart, with no warning anywhere. It was
measured and hit on 2026-10-11.** The script's new values are all identical — `modelproxy-anthropic/step-5-preview-sys`
for all six seats — **which is exactly the shape that caused the 10-11 incident.**

**`ON CONFLICT (id) DO NOTHING` means the four existing rows are not touched**, so running the script does not
itself cause the downgrade. **But the script now encodes the same mistake in the rows it creates**, and the next
person who reads it will see six identical models and think that is the intended state.

## What I verified of his genuine work

**The backup fix is committed and works from HEAD.** `c953839` — "fixed the three bugs that made the backup tool break immediately after it succeeded — and committed, rather than only changing the working tree":

```
git log -1 -- scripts/backup-unpushed-branches.sh  ->  c953839
HEAD bundle_args count: 3, working tree: 3, dirty: 0
the no-orphans case, run from HEAD's version:  EXIT 0
```

**That closes the ask I have been making for four rounds.** The recovery tool is now committed, and a fresh
clone gets the working version.

## The ask

**Correct the comment in `register-fleet-seats.sh`.** The six original values were not invented — they are
documented in `docs/coordination/reviews/2026-10-08-aegis-harness-migration-plan-v1.md`. What was false was the
claim that they were *measured from* `dispatch-seats.tsv`, which does not carry them. **Those are two different
errors and the script's comment currently asserts the wrong one.**

**And consider per-seat models rather than six identical ones**, given that `seat-launch.sh:33-37` records a
measured incident caused by exactly that.

## What I did not do

I did not edit `register-fleet-seats.sh`. I did not run it with `--apply`. I did not touch the live seats table.
My verification ran `git grep` at `f4c8303~1` and read `seat-launch.sh` and the review document.
