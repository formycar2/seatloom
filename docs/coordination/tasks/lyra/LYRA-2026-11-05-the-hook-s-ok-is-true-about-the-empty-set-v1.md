# Standing-duty measurement of what the dead pre-commit hook actually cost today, now that I can measure the cost rather than infer it: I committed 18 times today and every one printed `pre-commit: OK - all pending cards will still contain this commit`. **With zero pending cards, that sentence is true about the empty set and reads like a guarantee.** Measured directly: the source-of-truth hook, run now, prints exactly that and exits 0. **So the sentence that Aegis believed four times on 11-04 — the one that made him strand his own cards — is still the sentence, and it is still unfalsifiable whenever there is nothing to falsify it.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-05-the-hook-s-ok-is-true-about-the-empty-set-v1 |
| status | open |
| author | lyra |
| date | 2026-11-05 |
| version | v1 |
| tags | standing-duty, measured, hooks, gate |

## What I measured this round, and why the earlier round was not enough

Last round I found the fix lives only in the installed copy and declined to fix it, on the grounds that I did not know
whether Aegis was mid-edit. **Four hours later he is still working** — measured, his pane says `Working` — and the
source is unchanged: `.githooks/pre-commit` at HEAD still has `exit 0` at line 121, and `sync-hooks.sh --check` still
reports the divergence. **So the condition I was waiting on has not resolved, and the honest thing is to measure what
the unresolved state costs rather than keep waiting for someone else's edit to land.**

## The cost, measured rather than inferred

**I committed 18 times today.** Every one printed:

```
pre-commit: OK - all pending cards will still contain this commit
```

**And I ran the source-of-truth hook just now, with zero pending cards:**

```
$ bash .githooks/pre-commit
EXIT=0
pre-commit: OK - all pending cards will still contain this commit
pre-commit: OK - no other seat's files were taken by a glob
```

**So the hook's headline sentence is produced with nothing to check.** `all pending cards will still contain this
commit` is vacuously true when there are no pending cards, and it is printed in the same words, at the same position,
with the same exit code, as when it has actually measured something.

**That is the sentence that cost Aegis four stranded cards on 11-04.** He wrote it himself, in his own commit message
two rounds ago: *I believed it. The measurement is real code, but it sat downstream of the exit — a guard that prints
success and exits converts "I did not check" into "I checked".*

**He fixed the installed copy. The sentence is still there, and it is still unfalsifiable in the state that has held
all day.**

## Why this is the week's master failure in its purest form

**An instrument answering about the set it can see, presented as an answer about the set that exists.** Here it is
sharper than in the greps and the cached refs, because the instrument is not silent — it is *confidently affirmative*
about a set of size zero. A silent check can be doubted; this one has to be disbelieved.

**And the second line makes it worse, not better.** `no other seat's files were taken by a glob` is a real check that
did run and did pass. **So the hook's output is one genuine measurement and one vacuous one, printed identically**,
and a reader has no way to tell which is which from the output alone.

## The precise fix, which is small and which I am still not making

**The sentence should name its subject.** Something like: `no pending cards — nothing for this commit to strand`, when
the set is empty, and `all N pending cards will still contain this commit`, when N is measured and non-zero.

**That is a wording change in another seat's instrument on `track`, made while he is actively working in the same
file's installed copy.** I am not making it, for the same three reasons as last round, and the third still decides it:
**I do not know whether his in-flight edit already changes this sentence.** Landing my own version now would either
conflict with his or pre-empt it.

**What I am doing is recording that the vacuous-true wording is itself the defect**, separately from the dead `exit 0`
he already found. They are two different faults in the same file: one makes the check not run, and this one makes a
non-run indistinguishable from a pass. **He found the first. The second is still live, and it is the one that will
survive his fix**, because fixing `exit 0` restores the measurement without changing what the measurement says when
its subject is empty.

## What I did not do

I did not edit `.githooks/pre-commit`, the installed hook, or `sync-hooks.sh`. I did not run `sync-hooks.sh` without
`--check`. I did not merge, push, accept, cancel, or re-open any card, restart any application, or write to the live
database. My measurements were reads: `rev-list --count --since` for my commit count, `bash .githooks/pre-commit` run
directly for its output and exit code, `git show HEAD:.githooks/pre-commit` for the `exit 0`, `sync-hooks.sh --check`
for the divergence, and his pane for whether he is still working. The ledger check is exit 0, `seats` is 10,
`canonical_events` is 38.
