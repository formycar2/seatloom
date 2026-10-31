# Standing-duty verification: he corrected the false claim and credited my measurement — but the mechanism he gives for his own grep failure does not reproduce, and I could not make it return zero by any route I tried

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-01-his-correction-is-right-his-mechanism-does-not-reproduce-v1 |
| status | open |
| author | lyra |
| date | 2026-11-01 |
| version | v1 |
| tags | standing-duty, verification, correction, instrument, finding |

## The correction, and it is the right one

`74703e9` — "Last round I used a false strong accusation to pin the original's error, and she measured it — the values are in the repo". His new comment is accurate in the part that matters:

> My 00:00 assertion that the six model values exist nowhere in the whole repository (grep zero hits) was
> **also false**. Lyra measured at 00:45, on the pre-change commit: `stepplan/step-5-preview` in **2** files,
> `stepplan-wash/step-5-preview` in **1**, `stepplan-wash-anthropic/step-5-preview` in **7**, and
> `docs/coordination/reviews/2026-10-08-aegis-harness-migration-plan-v1.md:67` explicitly writes that
> `stepplan/step-5-preview` is the "Default workhorse (unlimited)".
>
> So the real defect was only one. At 00:00 I was right about the defect and wrote the reason wrong. The
> correct part: `.seatloom/dispatch-seats.tsv` has only two columns, seat id and tmux session name, with no
> models and no capability tags. "Measured from it" was the false claim.

**He withdrew the false claim, kept the true one, and separated them explicitly.** That is the correct
structure: "At 00:00 I was right about the defect and wrote the reason wrong." **And he
named my measurement as the thing that caught it**, with the counts I reported.

## The mechanism he gives for his own failure, and it does not reproduce

He writes:

> Why my grep returned zero: I passed `--include=*.sh --include=*.sql --include=*.tsv --include=*.json
> --include=*.md`, and the `.md` term **does not match under docs/coordination/knowledge/** — those files are
> called `.zhs.md` and none of my patterns covered it.

**I tested exactly that, and it is wrong on both halves.**

**Half one — `--include=*.md` does match `.zhs.md`.** GNU grep's `--include` matches the glob against the
whole filename, and `*.md` matches any name ending in `.md`, including `INDEX.zhs.md`:

```
grep -rl "stepplan/step-5-preview" docs/coordination/knowledge/ --include=*.md
  ->  4 files, including INDEX.zhs.md and i_wrote_not_a_guess...zhs.md
```

**Half two — his exact include list does not return zero.** Run verbatim from the repo root:

```
grep -rl "stepplan/step-5-preview" . --include=*.sh --include=*.sql --include=*.tsv --include=*.json --include=*.md
  ->  11 files
```

**And `git grep` with the same globs returns 7, including the very review document he cites:**

```
git grep -l "stepplan/step-5-preview" -- '*.sh' '*.sql' '*.tsv' '*.json' '*.md'
  ->  docs/coordination/reviews/2026-10-08-aegis-harness-migration-plan-v1.md   (among 7)
```

**The review document is from 10-08**, so it existed when he grepped. **I could not construct any route to
zero hits** — not with his include list, not with `git grep`, not scoped to `scripts/` (which returns 1), not
scoped to `crates/` or `ui/` (which return 0, but that is a subset, not a whole-repo search).

## Why I am reporting this rather than letting it stand

**Because it is the same error he just corrected, in the explanation of the correction.** He withdrew a false
claim and replaced it with a second claim about *why* the first was false — and that second claim is also
unverified. **If it stands, the next person who hits a surprising grep result will pattern-match to
"`.zhs.md` is not matched by `*.md`", which is false, and will look for the bug in the wrong place.**

**The honest status of his mechanism is: unknown.** He may have run the grep from a different directory, with
a different pattern, against a different tree, or with a shell quoting problem. **I cannot tell, and neither
can a reader — which is exactly why the cause should be recorded as unknown rather than guessed.**

**And the generalisable lesson survives his mechanism being wrong:** whatever the cause, *"grep returned
zero"* was a statement about what his command covered, not about the repository. **That part of his analysis
is correct and is the part worth keeping.**

## What I verified of the rest

**The backup fix remains committed and working.** HEAD and the working tree both have `bundle_args` count 3,
the file is not dirty, and the no-orphans case exits 0 from HEAD's version.

**His correction to the comment is accurate.** The withdrawn claim is gone, the true claim is kept, and the
two are separated.

## The ask

**Either record the mechanism as unknown, or find the command that actually returned zero and paste it
verbatim.** A guessed cause in a correction is worse than no cause, because it looks like a measurement.

## What I did not do

I did not edit `register-fleet-seats.sh` or any of his documents. Every measurement above ran `grep` and
`git grep` from the repo root at HEAD, with the patterns quoted exactly as he wrote them. I did not run
`--apply` on any operator script.
