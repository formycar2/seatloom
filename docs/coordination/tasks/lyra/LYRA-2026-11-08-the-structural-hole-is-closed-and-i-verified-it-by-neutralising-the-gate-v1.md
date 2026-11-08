# Standing duty: the structural hole is closed, and I verified the containment by neutralising the gate myself. **`REAL_SANDBOX_PATH` is now configurable and `SAND_APP`'s default *references* it, so one redirect moves the target and the script's notion of the real path together and they cannot drift apart.** And there was a fifth destruction — Nimbus's own intermediate version, caught in the same run by the fingerprint he built.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-08-the-structural-hole-is-closed-and-i-verified-it-by-neutralising-the-gate-v1 |
| status | open |
| author | lyra |
| date | 2026-11-08 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 422 events and 83 dispatches.** Nimbus's containment fix is on `packet/nimbus/candidate-final` at **`a1fddfb`**, responding to `disp-MZHfmhAZ9ztz9R_byYgHY`. `get_next_dispatch` returned nothing ready. **The main application is intact**: md5 `28753f61…`, 12,713,776 bytes, pid 90805, unchanged across every measurement including all five incidents.

## The fix, and why it is the right shape

**Two lines, adjacent to each other:**

```bash
REAL_SANDBOX_PATH="${SEATLOOM_REAL_SANDBOX_PATH:-/Applications/SeatLoom-Sandbox.app}"
SAND_APP="${SEATLOOM_SANDBOX_APP:-$REAL_SANDBOX_PATH}"
```

**The insight is that the fix is at the call site, not the predicate.** His own diagnosis of why three consecutive commits had the same defect in the same file:

> the fix was always applied to the ASSERTION and never to the CALL SITE. **A detector is not a containment.**

**So `SAND_APP`'s default no longer names the real path literally — it names the variable that holds it.** Redirecting `SEATLOOM_REAL_SANDBOX_PATH` moves the target *and* the script's notion of what is real, together, so they cannot drift apart. **That is the property that was missing in all four previous fixes**, each of which guarded a path that a separate redirect could move out from under it.

## I verified the containment by defeating the gate myself

**A fix that works at baseline proves nothing about the mutation, which is the lesson of all five incidents. So I neutralised the gate** — replaced its condition with `if false`, the same shape section 17's mutation uses — and then asked where the destructive path would point:

| state | `REAL_SANDBOX_PATH` | `SAND_APP` default | destructive path reaches |
|---|---|---|---|
| gate intact, real use | `/Applications/…` | `/Applications/…` | real — **and the gate refuses** |
| gate **neutralised**, fixture | `$FIX/Applications/real-sandbox-standin.app` | **follows it** | **inside the fixture** |

**So with the gate removed entirely, the destructive path points inside the fixture, because `SAND_APP`'s default follows the redirected `REAL_SANDBOX_PATH`.** That is containment rather than detection, and it survives the mutation that defeats the guard. **The fixture sets `SEATLOOM_REAL_SANDBOX_PATH` in eight places, so there is no path through the suite where the target can fall back to `/Applications`.**

## The fifth destruction, and what it proves about the fix

**The sandbox is now a fifth fake** — 95 bytes, mtime 11-08 10:03, `seatloom-build-commit=314ab7e6e1ed`, md5 `7c0a6e2a…`. **Nimbus caused it himself, in the same session in which he read Flux's verdict describing exactly this defect**, and he recorded it:

> my first version made the literal configurable and left `SAND_APP`'s default hardcoded. I ran the suite. The fingerprint changed.

**So he reproduced the defect while fixing it, and section 20's per-mutation fingerprint caught it in the same run.** That is the detector doing the one job a detector can do — telling you the containment failed — and it is why he found it rather than shipping it.

**And the cost was zero, which is worth stating precisely rather than as comfort**: the real binary had already been destroyed at 09:18, so what he overwrote was Flux's fake. **Five destructions, and only the first one lost anything real.** That is not a defence of the sequence; it is a measurement of where the loss actually is.

## The timeline, complete

| # | when | who | what was lost |
|---|---|---|---|
| 1 | 11-07 16:00 | Nimbus | **the real 12.7 MB binary** |
| 2 | 11-07 17:34 | Flux | fake #1 |
| 3 | 11-07 20:09 | Nimbus | fake #2 |
| 4 | 11-08 09:18 | Flux | fake #3 |
| 5 | 11-08 10:03 | Nimbus | fake #4 — caught in the same run |

**The main application has never been touched, across all five.**

## Two of his own assertions were wrong rather than the code

**He recorded two of his own false verdicts**, and they are the kind I keep making: a greedy `sed` that returned the variable name instead of the path, so the assertion reported "the default was changed" when the code had not changed; and an assertion referencing a variable that does not exist in the test script's own shell, which `set -u` turned into an unbound-variable abort that killed the assertion **before it checked anything.** **A check that dies before evaluating is indistinguishable from a check that passes, unless something notices the absence of the check.**

## What he did not do

**The sandbox is not restored** — he says it is a 95-byte text file needing an operator-authorised rebuild and that he has not attempted it. **The UI gate is still uncovered by any verdict**, which he lists himself rather than leaving implicit.

## What I did not do

I did not modify, restore, rebuild, install or replace anything, and I did not run the suite. My measurements were reads plus one mutation in a throwaway `--shared` clone: `git show <commit>:<path>` for the two fix lines and the fixture's eight `SEATLOOM_REAL_SANDBOX_PATH` references, the neutralisation of the gate to `if false` with the resulting default traced by hand, `md5 -q` and `ls -l` on both binaries, and the ledger for his response. The clone was removed.
