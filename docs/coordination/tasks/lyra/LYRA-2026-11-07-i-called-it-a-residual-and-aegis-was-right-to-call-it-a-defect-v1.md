# Standing duty: the residual I recorded last round as "a residual rather than a defect" has been dispatched as REQUIRED, and the fix is already in and verified in all four cases. **I called it a residual; Aegis called it a required defect, and Aegis was right** — the asymmetric case was the exact shape that destroyed the sandbox at 20:09. Separately, Nimbus failed a dispatch by checking its premise and finding it false, which was the correct call and is a real finding.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-i-called-it-a-residual-and-aegis-was-right-to-call-it-a-defect-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty, correction |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger moved 389 → 401 events and 77 → 79 dispatches.** Two things happened: a `SeatDispatchFailed`, and the equality-gate fix landing. `get_next_dispatch` returned nothing ready. Cards are still three: two accepted, one cancelled, none pending.

## The correction I owe on my own last document

**Last round I wrote that the `SAND_APP`/`PROTECTED_PATH` pairing was "a residual rather than a defect, because the alternative — making the gate refuse any mismatch — would break the fixture's ability to contain at all."**

**Aegis dispatched it as REQUIRED, and his reasoning is better than mine**: the gate must fire when the *target* is the real application path, **independently of what `PROTECTED_PATH` is set to** — two independent conditions, either one blocking. That is not "refuse any mismatch"; it is "refuse the real path, and separately refuse the protected path", which is exactly the shape that closes the hole without breaking containment.

**And the fix is in at `857ca85`, citing my measurement by name:**

```bash
REAL_SANDBOX_PATH="/Applications/SeatLoom-Sandbox.app"
if { [ "$SAND_APP" = "$REAL_SANDBOX_PATH" ] \
     || [ "$SAND_APP" = "$PROTECTED_PATH" ]; } \
   && [ "${SEATLOOM_ALLOW_REAL_SANDBOX:-}" != "1" ]; then
```

**I verified all four cases myself rather than reading his report:**

| `SAND_APP` | `PROTECTED_PATH` | gate |
|---|---|---|
| real | real | **REFUSE** — correct for real use |
| scratch | scratch | **REFUSE** — correct for the fixture |
| scratch | other | **ALLOW** — a scratch target with an unrelated protected path is not a hazard |
| **real** | **scratch** | **REFUSE** ← **the 20:09 hole, now closed** |

**So my "residual" was a defect, and my reason for downgrading it was wrong**: I assumed the only alternative to equality was refusing every mismatch, when the actual fix is a second independent condition on the real path. **I classified a hazard out of the defect category because I could not see the fix, which is the same error as the proxy criterion in a new position — and Aegis, who could see the fix, classified it correctly.**

## The failed dispatch, which is a finding rather than a failure

**`disp-OupuAHH6tLwlzqXnswXV8` asked Nimbus to open two acceptance cards for verified CLI deliveries. He failed it**, and his reason is measured:

> `scripts/work-list.sh` exists only on `packet/nimbus/work-list-snapshot-actors` at `310c80a`, which is not an ancestor of `candidate-final` and not on `track`.

**I verified that myself:**

| file | `candidate-final` | `track` | `work-list-snapshot-actors` |
|---|---|---|---|
| `scripts/work-list.sh` | **NO** | **NO** | YES |
| `scripts/prepare-verification-build.sh` | YES | YES | YES |

**So the dispatch's premise was false for one of its two subjects, and a card opened against it would have named a file that does not exist on the branch the card points at.** He says he would rather report that than write two plans whose first step fails, and that is the right call.

**`retry_count` is 0 and `max_retries` is 0, so it will not auto-retry** — it needs a fresh dispatch with a corrected premise. That is an operator-or-PO action, and the corrected premise is now measured: **the work-list delivery needs either its own branch as the card's subject, or a landing decision, before any card can name it.**

## What this round establishes about the candidate

**The candidate lineage at `857ca85` now has: the gate with two independent conditions, four-case fixture coverage, and per-mutation fingerprinting (`APP_SUM` appears four times, consistent with before-and-after inside a loop).** Aegis's dispatch also noted the trap I would have fallen into: **`--build-only` exits at line 442 and never reaches the gate, so a `--build-only` probe proves nothing about it.** He wrote that into the dispatch, which is the kind of warning that only comes from having been bitten.

## What I did not do

I did not modify any file, dispatch anyone, merge, open or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: the ledger for the failed dispatch, the verdict-bearing dispatch, and the fleet's state; `git cat-file -e <branch>:<path>` for the three-branch file existence table; `git show <commit>:<path>` for the new gate; and a four-row evaluation of the gate's truth table in a throwaway clone, removed afterwards.
