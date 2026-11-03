# Standing-duty verification: I finished auditing my own landing script's never-exercised steps — Step 3's append is sound, Step 6's self-guard fires in both directions, the staged set is named rather than globbed, and the pre-commit hook does not block the landing because on track there is no seat to derive. Three of four steps verified; the one I could not verify is the commit itself.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-03-i-finished-auditing-my-own-landing-script-v1 |
| status | open |
| author | lyra |
| date | 2026-11-03 |
| version | v1 |
| tags | standing-duty, verification, audit |

## Why this round

Over the last two rounds I found and fixed two real defects in my own landing script: a seven-versus-six seat
list that would have made it die, and a stale source ref that would have made it land the pre-Step-0 documents
while reporting success. **Both were in steps the script had never reached, because Step 1's precondition
refuses first.**

**A script whose failure I have found twice in its unexercised half is not a script I should stop auditing at
two.** So I audited the remaining steps.

## Step 3 — the grandfather append: sound

The block is appended with a heredoc that begins with a blank line, and I verified track's file ends with a
newline (`tail -c1 | xxd` → `0a`). **So the result is existing content, a blank separator, then the comment
block — a valid append, not a concatenation onto an unterminated line.**

The idempotence check is `grep -qF "docs/coordination/tasks/onyx/ONYX-HEARTBEAT.md"`, and I verified track's
file contains that string **zero** times, so the append will run rather than skip.

**And the append is provably pure**: `BEFORE_LINES` is captured from `HEAD`, the new file's first
`BEFORE_LINES` lines are compared byte-for-byte against the old content with `cmp -s`, and a mismatch dies.
**That is a guard that reads the pre-state before changing it** — the correct shape, and the one I documented
as a failure family when Aegis's first hook version measured the state before the commit instead of after.

## Step 6 — the self-guard: fires in both directions

I misread this one first. I saw `git add` at line 254 and the guard at 248 and briefly concluded the guard came
after the staging. **It does not** — 248 is before 254. **I corrected myself by reading the line numbers rather
than trusting my impression**, which is the same discipline that caught me twice this week.

Then I mutation-tested it in a scratch repo:

```
script tracked on track   -> guard fires, dies   (correct: it would be swept into the commit)
script untracked          -> guard passes        (correct: the script arrives untracked by design)
```

**Both directions correct.** And the staged set is `"${DOCS[@]}" "$GRANDFATHER"` — **seven explicit paths, not
a glob.** So the commit cannot sweep in the script itself, or the probe, or anything else. **That is the
opposite of the `git add -A` mistake I made on 11-01, and it is why the guard is a backstop rather than the
primary defence.**

## The pre-commit hook does not block this landing

The hook that blocks another seat's files is live, and this commit touches `tasks/mira`, `tasks/nimbus`,
`tasks/flux`, `tasks/onyx`, `tasks/vela`. **It does not fire**, and I verified why rather than assuming:
`SEAT` is derived from the branch via `packet/<seat>/...`, and on `track/infra-foundation` there is no such
prefix, so `SEAT` is empty and the rule is explicitly skipped:

```bash
if [ -z "$SEAT" ]; then
  : # rule does not apply; continue (the card check still runs)
```

**That is the correct behaviour** — a merge onto the integration line is supposed to cross seat directories,
and a guard that blocked it would block every landing. **It is also the reason the hook's author-privilege
defect was worth fixing: with `SEAT` empty on track, the hardcoded-`aegis` version would have blocked all six
seats' landings.**

## What I could not verify, and it is the part that matters

**The commit itself.** Everything up to the commit is now verified; the commit requires the operator's `YES`,
a rebase of `docs-only` onto track (96 behind), and the script to be on track. **I cannot verify the last step
without performing it, and performing it is the operator's action.**

**So the honest state of my ask is: the script is now sound in every step I can reach, it is blocked on a
rebase and a landing that are not mine, and the remaining risk is entirely in the parts I have not been able to
run.** That is a better position than two rounds ago, when it was blocked on a rebase *and* carrying two
defects.

## What I did not do

I did not run the script. I did not rebase, commit the script to track, or push. My measurements were reading
Steps 3 and 6, `tail -c1 | xxd` on the grandfather file, `grep -c` for the idempotence string, a scratch-repo
mutation test of the self-guard in both directions, the line numbers of the guard versus `git add`, and the
hook's `SEAT`-empty branch. `seats` is 10, `canonical_events` is 1, `check-ledger-exists.sh` is exit 0.
