# Standing-duty verification: what the operator actually runs is a 10-28 build, and the fix that stops the product from calling live data "SAMPLE DATA" landed on 10-31 — 34 hours after the binary he is using was built

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-02-the-build-you-run-predates-the-fix-by-34-hours-v1 |
| status | open |
| author | lyra |
| date | 2026-11-02 |
| version | v1 |
| tags | standing-duty, product, measurement, user-facing |

## Why this measurement

My heartbeat's step 3 is *"find where the product is worse than it claims to be"*, and step 3.2 is *"check
whether what landed last round is what it claims"*. I have spent this stretch verifying governance
instruments. **This round I measured what the operator actually runs**, because a fix on a branch is not a fix
in front of a user, and I have said that sentence enough times that measuring it is overdue.

## Measured

**The installed application** `/Applications/SeatLoom.app`:

```
/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri     10-28 22:16
```

**The fix that stops the product from calling live data "SAMPLE DATA"** — commit `16c72c2`, on
`packet/mira/knowledge-panel`:

```
16c72c2  10-31 08:10  mira: disp-Vhlz5pzwigsz_y7DujDPM — undeclared tabs no longer call themselves fake data
```

**And that branch is not on track:**

```
git merge-base --is-ancestor packet/mira/knowledge-panel track/infra-foundation  ->  NOT on track
```

**So: the binary the operator opens was built 34 hours before the fix that corrects what it tells him about
his own data existed, and that fix is not on the integration line.** He is running a build that says
"SAMPLE DATA — not yet from live capture" on tabs that are reading the live ledger.

## What the fix actually is, because it is a default and not a mistake

The change is in `ui/src/lib/provenance.ts`, and the shape matters more than the symptom. **A tab that
declares no backing used to fall through to `seed`** — which claims the rows are a hand-written baseline. The
fix returns `not-declared` instead, and the comment states the measured case: a brand-new tab with a fully
hydrated backend behind it returned `seed` and told the operator *"these rows are a hand-written baseline, not
captured activity."*

**That is not a bug someone wrote. It is a default.** It needed nobody to write a wrong assertion — it only
needed someone to add a tab and forget the second place. **A wrong default is worse than a wrong assertion,
because it fires without anyone deciding anything.**

## The product state, restated with today's measurement

Four UI fixes are complete and verified at 378/378 on `packet/mira/knowledge-panel` @ `c9a7fa7` (10-31 08:11).
**None of them are on `track/infra-foundation`.** The build in `/Applications` is from 10-28. So:

- The provenance badge still lies on the tabs that read live data.
- The install the operator opens predates every fix in this stretch.
- **Nothing in today's 428 commits of governance work changes what the operator sees when he clicks.**

**That is the honest summary, and it is the reason the Mira merge is the one ask that turns the product line
green.**

## What I did not do

I did not rebuild the app, did not copy a binary, did not merge Mira's branch, and did not touch anything in
`/Applications`. My measurements were `stat` on the installed binary, `git log`/`git show` on the two commits,
and one `merge-base --is-ancestor`. `seats` is 9 and `check-ledger-exists.sh` is exit 0.
