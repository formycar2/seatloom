| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-07-2100-the-gate-only-blocks-equality |
| status | delivered |
| author | aegis |
| date | 2026-11-07 |
| version | v1 |
| tags | aegis, sandbox, containment, correction |

# The new gate only blocks equality, so the sandbox can still be destroyed by unsetting one variable

## What I found

Nimbus landed `3097320` ("the protected path is configurable, so every mutation is
contained") in response to last hour's dispatch. It fixes the structural problem for the
symmetric case. It does **not** close the hole.

There is exactly one gate in `scripts/prepare-sandbox-build.sh`, at lines 504–507:

```sh
PROTECTED_PATH="${SEATLOOM_PROTECTED_SANDBOX_PATH:-/Applications/SeatLoom-Sandbox.app}"
if [ "$SAND_APP" = "$PROTECTED_PATH" ] && [ "${SEATLOOM_ALLOW_REAL_SANDBOX:-}" != "1" ]; then
```

It fires only when the target **equals** the protected path. `SAND_APP` at line 84 is
`${SEATLOOM_SANDBOX_APP:-/Applications/SeatLoom-Sandbox.app}` — left unset, it silently
becomes the real path. So I evaluated all four combinations:

| target | protected | result |
|---|---|---|
| real | real | refuse |
| scratch | scratch | refuse |
| scratch | other | silent |
| **real** | **scratch** | **silent** ← the hole |

In that fourth row the gate says nothing, and execution reaches `rm -rf "$SAND_APP"` at
line 541 and `cp -R` at line 542, on the real application path.

That is not hypothetical. Nimbus's own commit message for `3097320` records it: "Unsetting
SAND_APP while redirecting PROTECTED_PATH made them differ, the gate stayed silent, and
`--apply` destroyed the real sandbox again at 20:09." I measured it, the fixture caught it
in the same run, and the gate as committed still has it.

Dispatched `disp-sLODoywUUUpXpPSCfZY96` at priority 98: two independent conditions, either
one blocking — refuse if the target is the hardcoded real path, and refuse if the target
equals the protected path.

## One thing I got wrong while investigating, and it matters

I first tried to probe the gate end-to-end with `--build-only`. It exited at line 442,
long before the gate, and I nearly recorded that as evidence. **`--build-only` can never
reach the gate**, because it stops before the install segment. A probe that cannot reach
the code under test measures nothing. I switched to evaluating the gate's own condition
directly, which is what produced the table above.

## Measured state, not assumed

- The real `/Applications/SeatLoom-Sandbox.app` is a 95-byte text file
  (`seatloom-build-commit=3beaa54bd7ae` + `MUST_HAVE_PRESENT_MARKER`), written at 20:09.
  No backup exists. It needs an authorised rebuild — only you can authorise that.
- `/Applications/SeatLoom.app` is untouched: pid 90805, real 12.7 MB binary, Oct 28.
- Nothing I ran this hour wrote to either path. Fingerprint before and after my work:
  `90fcd5ecd2107bd7782aec850158fd89`.
- `nimbus-psr` worktree left clean at `3097320`.

## Salvage

`nimbus-candidate-final-20261107-1600.patch` is now `.superseded.patch`, and
`check-salvage-fresh.sh` is green again (exit 0). The evidence that this is **superseded and
not lost**: the patch's result blob `f38f5d2` is in history as commit `0a7f1a7`, and its
base `f6ee787` is that same commit's parent state. I did not rename it on a guess — I looked
up the blobs first, which is the rule from the 08:00 lesson.

## Not done

- No card, no merge, no install, no application restart, no live-database write.
- The residual gate hole is dispatched, not fixed.
- The sandbox remains a fake and needs your authorisation to rebuild.
