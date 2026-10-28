# Flux delivery — two cards opened on Mira's verdicts, and the reproducibility gap she found

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | FLUX-2026-10-28-open-two-cards-on-mira-verdicts-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-28 |
| version | v1 |
| tags | acceptance-cards, cross-verification, reproducibility-gap, own-omission, seat-flux |

Dispatch `disp-sm1KWwFNrnTtYTsykXfa9`.
Packet `docs/coordination/tasks/aegis/AEGIS-2026-10-27-open-missing-acceptance-cards-v1.md`.

## Result: two cards, both pending, both citing Mira

```
        dispatch_id         | status  |     flux_verdict_ref
---------------------------+---------+--------------------------
 disp-06a95xPk7h6E2pj6FDo4O | pending | ev-mira-audit-flux-batch06a95
 disp-l-5ODXaSdIQcrcxTurFe_ | pending | ev-mira-audit-flux-emitdefer01
```

One card per dispatch, no merging. Both refs are Mira's, as the dispatch required — **not**
my own `ev-RZkE0mWIuSNWp0FmpJ8nU` or `ev-flux-emit-three-deferred-verdicts-01`. I confirmed
before opening that each ref is a `ReviewVerdictIssued` recording `pass` whose
`payload->>'dispatch_id'` names that dispatch, and that each event's `actor_ref` is
`seat-mira-001` rather than me. My own two events were self-verdicts and per Lyra's
2026-10-27 decision they cannot serve the gate.

The branch precondition needed work again: it was 2 commits behind track, so I merged
`track/infra-foundation` in first (three knowledge docs, no conflicts) and re-verified
`behind: 0`.

## The omission Mira found, and it is mine

This is the part of this dispatch worth the most, so I am putting it first rather than
burying it.

I recorded "UI 25 files / 311 tests" as a gate figure for Nimbus's dual-render delivery.
That number is **not reproducible on track**, because `ui/src/stores/useConversationStore.test.ts`
— the file carrying 11 of those tests — was never merged. Aegis's path-limited merge
`f72b515` brought in `useConversationStore.ts` without it.

I verified her finding myself rather than accepting it:

| check | result |
|---|---|
| the test file on Nimbus's `3b52ea2` | present, 11 `it()` blocks |
| the test file on `track/infra-foundation` | **absent** |
| the source `useConversationStore.ts` on track | present — source merged, tests did not |
| UI suite on track right now | **24 files / 300 tests** |
| UI suite I recorded | 25 files / 311 tests |

So the delta is exactly her finding: 11 tests, one file, never brought in. There is no
`--diff-filter=D` deletion to find, because the file was never added in the first place.

**Where I went wrong is narrower than "I missed a file".** I verified the dual-render
delivery on Nimbus's branch, which is the correct place to verify it, and the gates I ran
there were green. But I then quoted the branch's test count into a verdict whose subject
is work that will be merged to track, without saying "this count is on the subject branch,
not on track". Anyone re-running my number on track gets 24/300 and concludes either that
I fabricated it or that tests were lost. Both conclusions are wrong, and the ambiguity is
mine.

I did record "24 files / 300 tests" for the *merged* tree in the six-cards delivery note,
which is why I can state the gap precisely. But the verdict event itself carries 25/311
with no qualifier, and the verdict is the artifact downstream reads.

**Not fixed by me, for the reason Mira gives:** restoring the file changes the UI test
count on a branch that currently carries nine pending acceptance cards, which moves those
cards' baseline. That is a decision for Aegis and the operator, and it deserves its own
dispatch. I am reporting it, not doing it.

## Mira's audit of my two dispatches

Both pass, and the audit was substantive rather than a rubber stamp. She re-ran every
load-bearing claim she could reach: the live drift 87/3/0, the view's 90 rows,
sessions-still-5, the `apply-db.sh` guard passing with 20 classified and 0 unclassified,
both acceptance cards' branch/base/verdict/timestamps/plan shape, `pty/mod.rs`'s empty
diff, the whole change being 5 files, and the `session:output` emit lines being
byte-identical. She also listed four things she could **not** verify and why — my 8.6 µs
latency figure, the 3-TurnStarted/3-TurnEnded panel run, my MT-A/B/C mutation counts, and
the per-path breakdown of 65 commits — each because it needed my worktree or a live panel,
and each of which I had already flagged as not-verified myself. That is the correct shape
for an audit.

She also recorded one overbroad phrase of mine: I wrote that `session_cmds.rs` emits
`session:output` "unchanged with the same base64 payload", when in fact that file's entire
change in `3b52ea2` is rustfmt collapsing a type declaration from three lines to one — no
logic change at all, and the emit line byte-identical. The direction was right; the
phrasing implied a payload-structure check I had not specifically performed. She is
correct, and I am recording it rather than defending it.

Notably she also caught and corrected **two of her own errors in flight**: a first pass
that nearly marked me wrong because she queried `memories` for a `source_ref` prefix that
does not exist (the real figure is the view's row count, 90), and a mis-transcribed event
id she briefly reported as MISSING. Both were hers, both were self-caught, and both are
recorded in her verdict payload. That is the behaviour I try to hold myself to.

## What I did not do

- **I did not use my own verdicts.** `ev-RZkE0mWIuSNWp0FmpJ8nU` and
  `ev-flux-emit-three-deferred-verdicts-01` remain in the ledger as accurate records of
  what I did, but neither serves a gate.
- **I did not restore the lost test file.** Reported above, with the reason.
- **I did not re-verify anything.** The verification is the earlier dispatches' work; this
  dispatch is a process act.

## Escalation

1. **`useConversationStore.test.ts` (11 tests) is missing from track.** Source merged,
   tests not, by path-limited merge `f72b515`. Needs its own dispatch: restore the file,
   then decide what that does to the nine pending cards' baselines. This is the same
   failure mode Aegis recorded in
   `docs/coordination/knowledge/a_path_limited_merge_is_not_a_whole_branch_merge.md`.
2. **A gate figure measured on a subject branch must be labelled as such.** My 25/311 was
   true where I measured it and misleading where I published it. Worth adding to the
   verification checklist: state which commit the number was measured at.

## Files

`packet/flux/intent-verdict` at `cac5617` plus the track merge. No code changed.
