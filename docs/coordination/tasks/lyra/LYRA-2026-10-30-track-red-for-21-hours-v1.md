# Standing-duty finding: track has been red for 21 hours, and the merge that did it had the evidence in the branch name

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-30-track-red-for-21-hours-v1 |
| status | open |
| author | lyra |
| date | 2026-10-30 |
| version | v1 |
| tags | standing-duty, blocking, integration-line, red-tests, merge-gate |

## The finding

**`track/infra-foundation` has two permanently failing tests and has had them for about 21 hours.**

```
$ npx vitest run
 Test Files  1 failed | 30 passed (31)
      Tests  2 failed | 370 passed (372)
```

Both failures are in `ui/src/App.restartRestore.test.tsx` (test names as written on the branch):

```
× a COLD START lands on the Dispatch tab even when the recorded project has a saved tab
× switching projects inside a session still restores THAT project's tab
```

**These are Mira's two tests from yesterday — the ones that went red on unmutated code, the ones she
escalated instead of tuning green, the ones my ruling of 05:57 was written to fix.** The ruling is
`LYRA-2026-10-29-persist-from-input-ruling-v1.md`, and I verified just now that it is **not on track**.
Neither is the fix. So the defect my ruling addressed is on the integration line, red, and unaddressed.

## How it got there

| Commit | When | What |
|---|---|---|
| `fbc4bed` | 10-29 11:02 | Mira: *"the first-screen card's assertion is false. I changed it, did not finish it, and said where it is stuck"* (title in Chinese on the branch) |
| `8dfdb78` | **10-29 13:12** | **Merge `packet/mira/knowledge-panel` into track** (Xiaolong Zhang) |

The test file on track is byte-identical to the one on Mira's branch at that point. I verified
`8dfdb78` against `packet/mira/knowledge-panel` on that file: **no diff.** So the merge did not
introduce a stale copy or a conflict resolution; it merged exactly what was on the branch, red tests
included.

## The part that should not have been possible

**The commit message that introduced the failing tests states, in its own title, that the work is
incomplete and that the card asserting it is false.**

> "the first-screen card's assertion is false. I changed it, did not finish it, and said where it is stuck"
> — the commit title, in Chinese on the branch, rendered here in English

That is: *the first-screen card's assertion is false. I changed it, didn't finish, and said where it's
stuck.* A commit that says "this is not done, and here is the card that claims it is" was merged into
the integration line by a human, 21 hours ago, and the line has been red since.

**I am not claiming the merge was wrong.** Mira's branch carried real work — the cold-start fix, the
consumptive-skip invariant that is better than what I prescribed — and a branch with a known defect and
an honest escalation is a normal, healthy artifact. **The finding is that nothing between "branch is
known-red" and "line is red" exists.** There is no merge gate. I checked:

```
$ ls scripts/ | grep -iE "merge|gate|pre-merge"
merge-ql-projects.sh          <- merges QL projects, not branches

$ grep -rln "vitest" scripts/*.sh
scripts/record-appv2-deletion-ruling.sh
scripts/verify-tauri-bundle.sh   <- neither is a merge gate
```

**No script gates a merge on the UI suite.** The merge that put a self-declared-incomplete commit onto
track ran the Rust checks, which were green, and went through.

## Why this is the most expensive finding of the week

Everything else I have reported this week is a record that is wrong. **This is a product that is
broken.** For 21 hours, the integration line — the line every seat syncs from, the line the operator's
build runs — has carried two tests asserting behaviour that does not happen, and a UI suite that any
seat running it will read as "my change broke something."

The second-order cost is worse. My standing-duty protocol says a seat that finds a red check reports a
finding. If Nimbus or Vela or Onyx had run the UI suite during those 21 hours, they would each have
found two red tests about a tab-restoration race they did not touch, and each would have had to
discover on their own that the cause is 21 hours old and in someone else's merge. **That is the
"instrument not verified current" failure in a new costume: the instrument is fine, the thing it
measures was already broken.**

## What I am not doing

**I am not fixing it.** The fix is Mira's surface and it is a ruling I already wrote; she implements it
and I verify it. Editing `ui/src/App.tsx` from my branch is not my call and would produce a second
competing fix on top of a merge that already crossed two seats' work.

**I am not re-opening or cancelling anything.** The operator granted both cards at 02:29 today. One of
them — `acc-disp-H7pVqxTHO077Rlp6GrmzX` — was granted *with* the caveat I put in front of him, which he
read and accepted rather than requesting the re-verify. That was his call and it was made on accurate
information. It is not the same decision as "let the line stay red," and I am not going to retroactively
re-frame his acceptance.

**I am not blaming the merge.** It is recorded as information about a missing gate, not as a fault.

## The ask

**Two things, and the first is the one that matters:**

1. **A merge gate that runs the UI suite.** It is the only missing component and it is the reason a
   self-declared-incomplete commit could land. Without it, every future seat that is told "sync from
   track" inherits whatever was last merged, red or not.
2. **Mira's fix lands.** It is a ruling, not a discovery — `LYRA-2026-10-29-persist-from-input-ruling-v1.md`,
   written 05:57 yesterday, and the two tests it names must go green on track. She owed this already; the
   merge made it urgent rather than merely owed.

And a third, which is mine: **I should have checked whether track was green before I spent yesterday
evening verifying that two cards were sound.** Both cards were sound. Both were merged. Neither check
answered the question "is the line healthy," and that question was available to me all day.

## Corrections to my own earlier work, same document

While checking this I made an error I have now made enough times to treat as a standing instrument
fault. I queried `gitlab` for `track/infra-foundation`, got `b09ec6e` dated **10-09**, and began forming
a conclusion about a three-week-stale integration line. **Gitlab's `track/infra-foundation` is a stale
remote ref; the real line is the primary checkout at `980e490`, dated 10-30 10:29.** I also initially
read the `packet/lyra/milestone-summary` divergence as 103 lost commits — it is 5 commits *behind* on
gitlab, `HEAD..37a4b53` is empty, and a plain fast-forward push fixed it. All three revs now equal.

**Both errors are the same failure recorded seven times today: a measurement taken at a location that
was not verified to be the location the claim was about.** I am recording them here rather than in a
separate note, because the finding and the correction are one artifact.
