| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-08-0900-i-asked-someone-to-run-the-destructive-test |
| status | delivered |
| author | aegis |
| date | 2026-11-08 |
| version | v1 |
| tags | aegis, correction, honesty, verification |

# The verification came back fail, and the destructive mutation it ran was the one I asked for

## What happened

Last hour I reported the gate hole closed and dispatched independent verification at priority
96. It came back: **PASS on the gate, FAIL on the fixture.**

The verifier confirmed the gate at `857ca85` is correct, that its default behaviour is
unchanged, and that the baseline is green — 50 assertions, all 20 sections. Then it ran the
mutation I asked for: remove the second condition and confirm section 15 goes red.

That mutation reaches the real path, because section 15's combination 4 deliberately unsets
`SEATLOOM_SANDBOX_APP` so the target falls back to the real path. Under the un-mutated code the
second condition catches it. Under the mutation, nothing catches it, and the script reaches
`rm -rf` on the real path.

At 09:18 today the sandbox path was rewritten again, 95 bytes, commit `7d577df02462`.

## The thing I have to own

**My dispatch told it to run that mutation.** I wrote: "mutate the second condition away and
confirm section 15 goes red, so the new condition is load-bearing and not decoration."

I had already established load-bearing-ness myself an hour earlier — by evaluating the gate's
condition directly against all four combinations plus the allow flag, which touches the real
path zero times. Then I asked someone else to do the destructive version of the same proof.

The verifier's own words, which I accept: *"A VERIFIER WHO KNOWS A TEST IS DESTRUCTIVE AND RUNS
IT HAS NOT MEASURED ANYTHING; THEY HAVE RE-ENACTED THE DAMAGE."* That is true, and I am the one
who told it the test was safe to run. The damage was not theirs.

## What was and was not lost

Stated precisely, because "the sandbox was destroyed again" would be false:

| | before | after |
|---|---|---|
| sandbox | 95-byte fake, `3beaa54bd7ae`, Nov 7 20:09 | 95-byte fake, `7d577df02462`, Nov 8 09:18 |

The target was **already** a fake when the mutation ran. No additional real application was lost
this time. The real sandbox was lost once, at 17:34 yesterday, and that was reported then.

The main application is intact: pid 90805, 12,713,776 bytes, Oct 28 22:16, fingerprint unchanged
across every measurement in this dispatch.

## The root cause, and it is not the assertion

The verifier named it better than I did. This is the **third commit in a row with the same
defect in the same file**: section 13 at `0a7f1a7`, section 17 at `4d91ff9`, section 17 at
`857ca85`. Each fixed the previous instance and left the next, because the fix was always
applied to the **assertion** and never to the **call site**.

> A detector is not a containment.

Section 20 fingerprints the real paths and reports the damage. That is a detector. The
containment has to be that the mutation *cannot reach the real path at all*.

## The structural fix, and it is small

The mechanism already exists. `PROTECTED_PATH` is configurable:

```sh
PROTECTED_PATH="${SEATLOOM_PROTECTED_SANDBOX_PATH:-/Applications/SeatLoom-Sandbox.app}"
```

The second condition compares against a hardcoded literal:

```sh
REAL_SANDBOX_PATH="/Applications/SeatLoom-Sandbox.app"    # line 527
```

Make that literal configurable the same way, defaulting to the real path. Then the fixture
points both values at scratch paths, combination 4 exercises the identical code path, and the
mutation that removes the condition reaches `rm -rf` on a **scratch** path. That is containment.

Dispatched `disp-MZHfmhAZ9ztz9R_byYgHY` at priority 99, with three things said explicitly: run
every mutation with the real-path default redirected and fingerprint around each one; if a
mutation cannot be run that way, report that instead of running it; and that my previous
instruction was the error.

## Measured state

- Ledger: 35 tables. Six seats online. Self-wake POKED on time.
- All six checks exit 0. Both pending cards clickable.
- Sandbox: 95-byte fake, no backup. **Still needs your authorisation to rebuild.**
- Main application: untouched.

## Needs you

**The sandbox is the only thing standing between six verified deliverables and you.** It has now
been a fake for three days. I am not going to ask anyone to run anything else against it, and
the fix above is designed so that no future verification needs to.
