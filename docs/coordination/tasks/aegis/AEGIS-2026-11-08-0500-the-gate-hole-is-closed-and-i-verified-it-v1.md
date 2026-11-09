| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-08-0500-the-gate-hole-is-closed-and-i-verified-it |
| status | delivered |
| author | aegis |
| date | 2026-11-08 |
| version | v1 |
| tags | aegis, sandbox, containment, verification |

# The gate hole I reported last night is closed, and I verified it before asking anyone else

## What Nimbus landed

`857ca85` adds a second, independent condition to the one gate in
`scripts/prepare-sandbox-build.sh`:

```sh
if { [ "$SAND_APP" = "$REAL_SANDBOX_PATH" ] \
     || [ "$SAND_APP" = "$PROTECTED_PATH" ]; } \
   && [ "${SEATLOOM_ALLOW_REAL_SANDBOX:-}" != "1" ]; then
```

Either condition blocks. That is the right shape: a guard that can be bypassed by changing one
of the two variables is not a guard, and the previous version could be.

## What I measured myself before dispatching anyone

I did not take the commit message on faith. Four combinations, evaluated directly against the
gate's condition:

| target | protected | before | now |
|---|---|---|---|
| real | real | refuse | **refuse** |
| scratch | scratch | refuse | **refuse** |
| scratch | other | silent | **silent** — the redirect still works |
| real | scratch | **silent** ← the hole | **refuse** |

Plus the two cases that would mean the fix overshot: `real/real` with
`SEATLOOM_ALLOW_REAL_SANDBOX=1` is still silent, so explicit acknowledgement is honoured; and
`scratch/other` with the flag is still silent, so the redirect works with it set.

Then I ran the whole suite, `scripts/test-build-failure.sh`, in its own isolated fixture:
**50 assertions, all 20 sections pass**. Section 15 covers all four combinations explicitly.
Section 20 fingerprints the real `/Applications` before **and after each individual mutation**,
not only at the end — which is the change I asked for after last night's lesson that a green
end-of-run fingerprint does not prove the mutations were contained.

Fingerprint before and after my entire run was identical: `90fcd5ecd2107bd7782aec850158fd89`.
The main application, pid 90805, was never written. The suite cleans its own temporary
worktrees, and `nimbus-psr` is clean at `857ca85`.

## The case that used to be dangerous

Section 15's combination 4 is the one that destroyed the sandbox at 20:09: `env -u
SEATLOOM_SANDBOX_APP` makes the target fall back to the real path while `PROTECTED_PATH` is
redirected. That call still exists in the fixture, deliberately, because it is the only way to
test the case. The difference is that the gate now refuses it.

## Dispatched for independent verification

`disp-JHPUXAgN6W2RJ6pxCIWWn` to the verification seat, priority 96, with my measurements
stated so it can confirm or refute rather than repeat. Four specific asks: mutate the new
condition away and confirm section 15 goes red; confirm the redirect still works when only the
target is redirected; confirm the suite cannot write outside the fixture by fingerprinting
around each mutation; confirm the real-use default is unchanged with no environment at all.

## What this does not mean

- **The sandbox is still a 95-byte fake.** This fix means the sandbox can no longer be
  destroyed by a test that thinks it redirected the path. It does not bring the sandbox back.
- No card, no merge, no install. This work is on `packet/nimbus/candidate-final` and is not on
  the mainline.
- The six pass-verdict-without-card dispatches are still held back, because each would ask you
  to accept work you cannot run while there is no sandbox to run it on.

## Needs you

**The sandbox rebuild is still the one thing blocking six verified deliverables.** It has now
been a fake for two days. Nothing I have built since makes that decision for you.
