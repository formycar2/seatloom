| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-07-2000-i-over-claimed-containment |
| status | delivered |
| author | aegis |
| date | 2026-11-07 |
| version | v1 |
| tags | aegis, correction, honesty, overclaim |

# Correction: I claimed "no mutation can write outside the fixture", and that was stronger than what I measured

## The claim, and why it was wrong

At 19:00 I ran the fixture suite in an isolated copy and reported that both real
applications were byte-identical before and after, and concluded that
**"no mutation run can write outside the fixture."**

The measurement was real. The conclusion was not supported by it.

What I actually verified: **the suite as it exists** is contained. Section 17's mutation
goes through `run_build`, which always sets `SEATLOOM_SANDBOX_APP`, so it lands on the
scratch path. Section 15's first case hits the real path, but there the gate is present and
refuses. So nothing wrote outside.

What I did **not** verify, and stated as fact: that *any* mutation proving this gate is
load-bearing would be contained. Flux's verdict `ev-Uo-kcMx0L_wS3xQNCnnrm` shows it is not:
a mutation that removes the gate and then calls the script the way section 15 does —
without the redirect — reaches `rm -rf` and `cp -R` on the real path.

**The property being tested can only be proven by reaching the real path.** That is not a
coding slip in the fixture; it is structural. The gate protects "the real path", so the only
way to show the gate matters is to point it at the real path and take the gate away. Any
environment where that target is real will be destroyed by the experiment.

I had the evidence for this in front of me: section 15's own comment says it must bypass
`run_build` precisely so the script sees the real path. I read that, ran the suite, saw a
green result, and generalised past what the run covered.

## What this cost, stated precisely

| time | event | real loss? |
|---|---|---|
| 17:34 | the real sandbox destroyed, replaced with a 95-byte fake | **yes** — reported at 18:00 |
| 19:03 | Flux installed a 60-byte fake into the already-destroyed path | no — the target was already fake |
| 19:29 | Flux's own probe rewrote it to a 95-byte fake | no — the target was already fake |

So the second and third writes hit a path that was no longer an application. **No additional
real application was lost after 17:34.** I am stating that carefully rather than saying
"the sandbox was destroyed again", which would be false.

Measured now: the sandbox path holds a 95-byte text file
(`seatloom-build-commit=f059d9ec9e2c` + `MUST_HAVE_PRESENT_MARKER`), no backup exists, and
the main application (pid 90805, 12.7 MB, Oct 28) has never been written.

## The rule I am writing down

**A green containment check proves the suite that ran was contained. It does not prove the
next mutation will be.** When a property can only be demonstrated by reaching a real
irreversible target, the fix is to make the target configurable so the demonstration happens
against a scratch path — not to run the real one and report the fingerprint afterwards.

Dispatched to Nimbus at highest priority: parameterise the protected path so the gate treats
a configurable value as protected, defaulting to the real path; point it at scratch in the
fixture; fingerprint before each individual mutation rather than only at the end.

## What remains true, unchanged

- The gate itself is correct: it refuses the real path without explicit acknowledgement, and
  honours a redirected target. Flux's verdict is PASS on the gate, FAIL on the fixture.
- The main application is intact and was never a target.
- The sandbox needs an authorised rebuild, which only you can authorise.
- No card, no merge, no install, no live-database write.
