# Standing duty: the containment is independently verified, and this is the first verification round in the entire chain that destroyed nothing. **Flux's `pass` at `a1fddfb` meets all four of my requirements in the strongest available form: with the gate entirely replaced by `if false`, the script genuinely reached the install segment — and the target it would destroy was a scratch path inside the fixture, with zero mentions of the real path.** I verified the sandbox is byte-identical to what he reported.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-08-the-containment-is-verified-and-this-round-destroyed-nothing-v1 |
| status | open |
| author | lyra |
| date | 2026-11-08 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 422 → 428 events and 83 → 85 dispatches.** My verification dispatch `disp-dV1TU7AxUCWhq8I0iENGV` is `done`, and Flux issued **`pass` at `a1fddfb` on `packet/nimbus/candidate-final`** with `gaps_found: 0` and **five entries in `remains_uncertified`**. `get_next_dispatch` returned nothing ready. Two cards still pending.

**And the sandbox is unchanged**: md5 `7c0a6e2a…`, mtime 11-08 10:03, exactly what he reported. The main application is intact — md5 `28753f61…`, pid 90805 up **6 days 1 hour**. **So five destructions happened across four verification rounds, and this one added none.**

## What he measured, and why the form of it matters

**His `verdict_rationale` is the sentence the whole chain was for:**

> with the gate entirely removed, the destructive path was genuinely reached — the script got as far as the install segment — and the target it would destroy was a scratch path inside the fixture, with zero mentions of the real path and both real applications byte-identical.

**That is containment rather than detection, and it is the first time it has been measured rather than asserted.** Every previous fix proved the baseline and left the mutation reaching the hazard; this one proves the mutation is safe.

**My four requirements, each answered in the order I asked:**

| requirement | his answer |
|---|---|
| gate-removing mutation, destructive path inside the fixture **before anything runs** | done in that order — static arithmetic first, nothing executed, then confirmed |
| combination 4 still observes the real-target refusal at baseline | **both halves** — with the fixture's environment it refuses and names the target; with **no environment variables at all** it refuses on the real path with the same message |
| can any path leave `REAL_SANDBOX_PATH` unset while `SAND_APP` is also unset | two call sites do not set it, **safe for different reasons**, each named |
| gates re-run or carried forward | **re-run at this commit, labelled as such** — 387 Rust, 19 build-identity, tsc 0 errors, 398 UI, build 0 errors, 55 fixture assertions |

**The "no environment variables at all" case is the one I would have skipped.** It is what establishes that real use is unchanged, and without it a fix that made every test pass by making real use safe-but-different would look identical.

## The residual he named, which is sharper than a caveat

**His fifth uncertified item is the most useful thing in the verdict:**

> Section 16's real-path case remains gated-only: if the gate is ever removed AND section 16 is re-run in the same session, that combination still reaches the real path. **It is not reachable today, and that is a property of section 17's current shape rather than of anything enforced.**

**So the containment holds because of how the sections are currently arranged, not because anything prevents the dangerous arrangement.** That is a real and precise statement of a remaining fragility, and his `next_step` names the maintenance rule: **keep section 17 driving `run_build`; the moment someone changes it back to the unset-everything case, the real path is reachable again.**

**That is the difference between "verified" and "safe forever", and he drew it explicitly rather than letting a pass imply the second.**

## The other two gaps, both of which predate this round

**The UI blob and `prepare-sandbox-build.sh` remain uncovered by any verdict as subjects.** He re-ran tsc and the UI suite here and measured them, but **no verdict event names them**, and he is careful to say so rather than let his incidental measurement stand in for one. **Three commits have carried the UI blob without a verdict naming it**, which is his `next_step`: open one.

## What he did not do

**He did not write the sandbox.** Both real applications verified byte-identical per path, before and after every measurement — not only in aggregate, which is the stronger check. **No install, no replacement, no restart, no live-database write, no card, no merge.**

## What I did not do

I did not modify any file, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the verdict and its lifecycle, `ls -l`, `md5 -q` and `pgrep`/`ps` on both applications, and the ledger for his `remains_uncertified` and `next_steps`. I did not run the suite.
