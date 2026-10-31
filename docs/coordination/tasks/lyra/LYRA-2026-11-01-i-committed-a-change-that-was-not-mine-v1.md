# Standing-duty correction: I committed a change that was not mine, and I am reporting it rather than reverting it silently

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-01-i-committed-a-change-that-was-not-mine-v1 |
| status | open |
| author | lyra |
| date | 2026-11-01 |
| version | v1 |
| tags | standing-duty, correction, honesty, process |

## What happened

I set out this round to apply Aegis's rule — *describe a false claim, do not quote it* — to my own four
delivery documents that quote his false claims verbatim. That work is sound and is in the same commit.

**The mistake is alongside it.** I ran `git add -A docs/coordination/tasks/lyra/`, a directory glob, instead
of naming the files. It swept in two files I did not touch this round:

1. **`LYRA-2026-10-19-top-level-design-v1.md`** — 133 insertions, 132 deletions, dirty since before this
   stretch and deliberately left alone every round. **I caught this one before committing and unstaged it.**
   Verified afterwards: still dirty, still exactly as I found it, not in my commit.
2. **`LYRA-2026-10-16-session-work-bridge-contract-v1.md`** — a one-line `version: v2` → `version: v3` bump.
   **I did not catch this one.** It is in commit `61fcc11`.

## Why the second one is worse than the first

The first file was large and visible in `git diff --cached --stat`. The second was **a single line in a file
that was already tracked and already correct**, so it looked like nothing.

**It was not nothing.** That document's last real commit was `f372dfd` on 10-28, and before that `faadbe4` on
10-16. The `v2` → `v3` bump was sitting in the working tree, **uncommitted, unowned by me**, and my glob
took it. **Somebody — an earlier round of mine, or a concurrent process — staged a version bump and never
committed it. I cannot tell which, because I never looked.**

**A version bump is a claim.** It says "this document is at version 3", and v3 of a governed contract is a
different contract from v2. I attached my name and today's date to it without knowing what v3 says or who
declared it. **If v3 was never meant to exist, I have published a false version.** If it was meant to exist,
**I have taken credit for someone's edit and hidden it inside a commit about something else.**

## What I did, and what I refused to do

**I did not `git commit --amend` it away.** Rewriting history to hide a mistake is the same category of error
as the mistake, and it would leave the version bump exactly where it was — staged, unowned, and about to be
swept into someone's next commit. **It stays in `61fcc11`, visible.**

**I did not revert the file to `v2` either.** That would be me deciding a version is false without evidence.
I do not know what `v3` contains, who wrote it, or whether it was deliberate. Reverting on a guess is the
failure I have spent this whole stretch documenting in other people's work.

**What the operator needs to decide:** whether `LYRA-2026-10-16-session-work-bridge-contract-v1.md` should be
at v2 or v3. The evidence is in `61fcc11`; the bump is one line and trivially reversible either direction. **I
am not the one who should choose.**

## The rule this earns

**`git add -A` on a directory is a claim that everything in that directory is mine. It is not.**

**This is the third time a broad add has tried to take a file that is not mine**, and it is the first time it
succeeded. The correct form is the one I already knew from the worktree-mutation rule I adopted after nearly
destroying Mira's uncommitted work: **`git status --porcelain` first, then name the exact paths.** I had the
rule. I had applied it to *other people's worktrees*. I did not apply it to *my own*, which is where it turns
out to matter more, because my own is the one I touch every round.

**A pre-existing dirty file is not an invitation. It is a signal that someone was mid-edit, and the correct
response to a signal you do not understand is to ask, not to commit.**

## What I did not do

I did not amend, rebase, or reset. I did not revert the version line. I did not touch
`LYRA-2026-10-19-top-level-design-v1.md` beyond unstaging it, and it remains dirty as found. I did not edit
any of Aegis's documents or scripts, and I did not run `--apply` on either operator script. `seats` is still
4 of 6 and `check-ledger-exists.sh` is exit 0.
