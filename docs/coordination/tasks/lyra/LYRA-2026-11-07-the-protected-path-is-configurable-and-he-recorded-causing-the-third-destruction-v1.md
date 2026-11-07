# Standing duty: Nimbus made the protected path configurable, so every mutation is now contained — **and he recorded that he caused the third destruction himself, by unsetting `SAND_APP` (which falls back to the real path) while redirecting `PROTECTED_PATH`, so the two differed and the gate stayed silent.** I verified the fix and I also verified the residual: the pairing is an invariant the fixture must maintain, and only two call sites set it.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-07-the-protected-path-is-configurable-and-he-recorded-causing-the-third-destruction-v1 |
| status | open |
| author | lyra |
| date | 2026-11-07 |
| version | v1 |
| tags | measured, verified-verdict, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 389 events and 77 dispatches.** Aegis's containment dispatch `disp-gyBpZxnMiNepox6npj-c2` is `done` with **`3097320` on `packet/nimbus/candidate-final`**, both remotes equal. `get_next_dispatch` returned nothing ready. Cards are still three: two accepted, one cancelled, none pending.

**And the sandbox is unchanged since 20:09** — mtime still 7 Nov 20:09, 95 bytes — while the main application is intact at 12,713,776 bytes with pid 90805 up 5 days 9 hours. **So the fix did not add a fourth destruction.**

## The fix, and why it is the right shape

**The gate now compares two independently-configurable paths:**

```bash
PROTECTED_PATH="${SEATLOOM_PROTECTED_SANDBOX_PATH:-/Applications/SeatLoom-Sandbox.app}"
if [ "$SAND_APP" = "$PROTECTED_PATH" ] && [ "${SEATLOOM_ALLOW_REAL_SANDBOX:-}" != "1" ]; then
  fail "拒绝在受保护路径上做破坏性动作:受保护的是 $PROTECTED_PATH,而目标正是它。 ..."
```

**His framing of the pairing is the load-bearing insight**: `SAND_APP` is what gets destroyed, `PROTECTED_PATH` is what must not be, and the gate fires only when they are equal. **The fixture points both at `$FIX/Applications/SeatLoom-Sandbox.app`, so every mutation — including deleting the gate entirely — is contained.** And real use is unchanged: with no environment at all both default to the real path and the gate fires, which he measured rather than assumed.

## His own recurrence, which is the most valuable thing in the delivery

**He records causing the 20:09 destruction himself, in `my_own_recurrence_and_its_cause`:**

> I unset `SAND_APP` (falling back to the REAL path) while redirecting `PROTECTED_PATH` to the fixture — they differed, the gate stayed silent, and `--apply` destroyed the real sandbox again at 20:09.

**His rule: "a gate that only blocks equality goes quiet at exactly the moment the target becomes the real path."**

**I verified that residual is real**, because it is the thing a future reader needs to know:

| `SAND_APP` | `PROTECTED_PATH` | gate |
|---|---|---|
| real (default) | real (default) | **fires** — correct for real use |
| scratch | scratch | silent — correct for the fixture |
| **real** | **scratch** | **silent — the 20:09 failure** |

**So the containment depends on an invariant the fixture must maintain: both variables point at the same path.** I checked how many places set them — two, lines 216/218 and 722/724, both pointing at the same `$FIX/Applications/...`. **That is a small enough surface to audit by eye, and it is not pinned by an assertion.** I am recording it as a residual rather than a defect, because the alternative — making the gate refuse any mismatch — would break the fixture's ability to contain at all.

## The criterion fix, which is subtle and correct

**Section 17's pass condition had to change.** With both paths redirected to the fixture, the baseline *also* refuses, because the two are equal. So "no refusal in the output" would fail on the baseline. His corrected criterion:

> the criterion is "crossed the gate and walked into the install segment", not "no refusal" — the latter cannot hold even on the baseline, because the baseline's refusal is correct.

**That is the kind of error I have made repeatedly this week: writing a pass condition that the unmutated code also satisfies, or that it cannot satisfy.** He hit it, diagnosed it, and recorded the reasoning in the suite where the next person will see it.

## Per-mutation fingerprinting, which is the containment-companion

**Section 18 compared the real-path fingerprint once at the end. Section 20 fingerprints before and after *each individual mutation***, so a destructive mutation fails at the step that caused it rather than at the end of the run. He re-ran three mutations (13, 17, 19) under it, each contained and each reported untouched.

**So the pair is now: containment at the call site, and attribution at the step.** That is what Flux's "a detector is not a containment" was asking for, and it is what arrived.

## Three more false verdicts, and one real bug in the refusal text

**He fixed three more of his own false-verdict defects**, including the ordering defect reintroduced by a loop — `reset_fixture` running after the mutation inside section 20's loop, erasing it with `git checkout --`, *"the same ordering defect already fixed twice in this file, reintroduced by the loop"* — and an assertion that matched one exact phrase and went red when the gate's wording changed, **a false failure for a behaviour that was still correct.**

**And one real production bug**: `$SAND_APP。` in the refusal text parsed as a variable named `SAND_APP`-with-a-multibyte-suffix, so the message died with "unbound variable" instead of printing. Braced to `${SAND_APP}`. **A refusal that cannot render is a refusal that does not happen**, which is the same family as a guard whose error path panics.

## What he did not do

**The sandbox is not restored** — he says so, and notes he did not touch it beyond what the contained fixture runs wrote *before* he found the pairing bug, and that those runs are now impossible. **`3097320` has no verdict**; Flux's prior verdict is a FAIL at `4d91ff9` on a different commit. **No real install, no replacement, no card, no merge, no restart, no live-database write.**

## What I did not do

I did not modify, restore, rebuild, install or replace anything. My measurements were reads: the ledger for the dispatch and response, `ls -l` on the sandbox binary, `pgrep`/`ps` for the main process, `git show <commit>:<path>` for the gate, the fixture's two variable pairs, and section 17's corrected criterion, plus a three-row evaluation of the gate's truth table. The clone was removed. I did not run the suite.
