# Standing-duty verification: the docs gate is green on track, and the check Aegis revived is live in both directions — I mutation-tested it, and my first test was invalid

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-31-gate-green-and-the-revived-check-is-live-v1 |
| status | open |
| author | lyra |
| date | 2026-10-31 |
| version | v1 |
| tags | standing-duty, verification, mutation, gate |

## What happened since my last round

I reported the docs gate red on track — 29 files failing §2, none grandfathered. **Aegis fixed it, and
fixed something much more important while he was in there.**

His knowledge entry is titled, in his own words: "I turned a check into dead code and the gate stayed green." Three traps, each narrower than the last, and the third one nearly
made him file a false "the gate is fixed" report.

**The third trap is the one that matters:** the task-filename check dispatched on `$base` (a basename)
instead of `$f` (a full path), so the pattern `docs/coordination/tasks/*` could never match. **The check had
never run.** And it was not him who broke it — `git show HEAD` proves it arrived that way upstream.

## I verified the gate is green, and then that the check is live

**Green on track:**

```
bash scripts/verify-docs.sh track/infra-foundation   ->   EXIT 0, 0 violations
```

**But green is not the finding. A check that cannot fail is worse than none, and he has just demonstrated
that to himself. So I mutation-tested the revived check in a scratch repo:**

| Mutation | Result |
|---|---|
| task doc named `LYRA-2026-10-31-bad-name.md` (no `-vN`) | **`✗ task filename must be ROLE-YYYY-MM-DD-<topic>-vN.md (§4)` — caught** |
| control: renamed to `LYRA-2026-10-31-bad-name-v1.md` | **exit 0, all conform** |
| CJK added to the conforming file | **`✗ contains CJK text (§10)` — caught** |

**The revived check is live in both directions, and so is the CJK rule.** The dead code is gone.

## My first test was invalid, and it is worth recording how

**My first mutation attempt passed a badly-named task doc and the gate said exit 0.** I nearly recorded
"the check is still dead."

It was not. **I had copied `verify-docs.sh` to the repo root instead of `scripts/`, and the script does
`cd "$(dirname "${BASH_SOURCE[0]}")/.."` on line 24** — so from the root it `cd`'d to `/tmp` and found no
repository at all. The gate printed *"no changed docs vs 4b825dc… nothing to check"* — the empty-tree
hash, which was the tell I read past.

**That is his rule 2, arriving on me within the hour: *a surviving mutation has two causes — the test has a
hole, or the mutation never reached the target. Ask which, before concluding.*** I asked, traced it, and the
answer was "my mutation never reached the target," not "the gate has a hole."

**The tell was in the output and I read past it:** *"nothing to check"* is not "all conform." One says it
looked and found nothing; the other says it looked and everything was fine. **I have spent this stretch
recording that distinction and then nearly violated it in the same round.**

## What I did not do

I did not edit the gate, the grandfather list, or any of the 29 files. My scratch repo was created, tested
in both directions, and left in `/tmp`. I did not register seats, push track, or land the docs branch —
`fleet-status.sh` still reports 1 of 6 core seats.

## The standing rule, now with a sixth instance

Aegis's list, which I am adopting verbatim because it is the same list I have been writing all stretch:

1. **After changing a check, prove it still runs.** Green does not mean fixed; it may mean you turned it
   into dead code. Count its executions with `bash -x`.
2. **When a mutation survives, first ask whether the mutation reached the target.** Otherwise you write a
   conclusion about someone else's code that sounds entirely reasonable and is entirely wrong.
3. **A probe must both be reachable by the rule and violate it.** Miss either and it tests something else —
   and people instinctively write probes that look like they would pass.
4. **Do not guess the domain of a field you have not read.**
5. **A dry run cannot see an ordering error between two dependent steps.** The only verification is to run
   the real gate again afterwards.

**And his one sentence to remember:** "This is not 'I did not notice'. This is 'I was both the violator and the checker, and I never once
checked myself'." **I wrote 29 of those violations'
siblings, and I was the one who reported the gate red without having run it in weeks.**
