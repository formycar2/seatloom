| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-08-1030-the-destructive-mutation-is-contained |
| status | delivered |
| author | aegis |
| date | 2026-11-08 |
| version | v1 |
| tags | aegis, sandbox, containment, verification, salvage |

# The mutation that destroyed the sandbox four times is now contained, and I proved it by running it

## What Nimbus did, and why it is the right fix

The independent verdict said the reason three commits in a row had the same defect was that the
fix was always applied to the **assertion** and never to the **call site**. Nimbus took that
literally and changed both:

```sh
REAL_SANDBOX_PATH="${SEATLOOM_REAL_SANDBOX_PATH:-/Applications/SeatLoom-Sandbox.app}"  # line 93
SAND_APP="${SEATLOOM_SANDBOX_APP:-$REAL_SANDBOX_PATH}"                                  # line 94
```

The target's default now **reads** the configurable value, so the target and the script's
notion of the real path move together and cannot drift apart. Redirect one value and both move.

Nimbus went further than I asked, and recorded it: it made the literal configurable, left the
call site hardcoded, ran the suite, watched the fingerprint change, and **reproduced the very
defect the verdict described in the same session that read it.** That is the honest version of
this work — the half-fix was tried and measured, not assumed.

## What I verified, by running the destructive mutation

I ran the full suite, including the gate-removing mutation that has destroyed the sandbox four
times. This time it is safe to run, because every path is a fixture stand-in.

**52 assertions, all 20 sections pass.** Fingerprint of the real `/Applications` before and
after the entire run is identical:

```
before: 0d00e85f3682fb8bdf52b9a2aceecdfb
after:  0d00e85f3682fb8bdf52b9a2aceecdfb
```

The four gate combinations all behave as required, and the real-use default is pinned: with no
environment variables at all, the script still refuses on the real path and the refusal message
names the real path.

The difference from the last three attempts is not more assertions. It is that the mutation now
reaches `rm -rf` on a **scratch** path. **A detector is not a containment**, and this is the
first version that is a containment.

## The sandbox was not damaged by this run

I want to be precise, because "destroyed again" would be false:

- The verifier wrote to the sandbox at 09:18 — that was the mutation **I** asked for, and the
  target was already a 95-byte fake, so no additional real application was lost.
- My run this hour did not write to it. Fingerprint identical before and after.
- The real sandbox was lost once, at 17:34 on Nov 7, and was reported then.
- The main application is intact: pid 90805, 12,713,776 bytes, Oct 28 22:16.

## Salvage

`nimbus-candidate-final-20261107-2100.patch` renamed to `.superseded.patch` **on evidence, not
on a guess**: 121 of its 133 added lines are verbatim in HEAD, and the 12 that are not are
exactly the hardcoded literals this fix replaced. So its work was completed and superseded, not
lost.

Nimbus's new in-flight work — the fixture no longer prints `git rev-parse --short=12 HEAD`,
which produced a label like `7d577df02462` that looks like a real commit but does not exist in
this repository — is captured as `nimbus-candidate-final-20261108-1030.patch`, verified both
directions. That change came from Lyra's measurement that she could not tell a real build from a
fabricated one.

`check-salvage-fresh.sh` is green again, exit 0.

## Push status, stated honestly

**Remote state is unknown.** Two commits are local-only. `git push` to GitHub timed out
repeatedly this hour (`Connection to 172.182.252.133 port 22: Operation timed out`), and a
`ls-remote` to confirm also failed. I am not claiming pushed.

Pushed is deliberately not a completion criterion in the protocol, and the reason is structural:
with pending cards open, pushing can break your ability to accept. Both pending cards remain
clickable, and I confirmed that after each local commit.

## Measured state

- Ledger: 35 tables. Six seats online. All six checks exit 0.
- Both pending cards clickable.
- Sandbox: 95-byte fake, no backup. **Still needs your authorisation to rebuild.**
- Main application: untouched.
