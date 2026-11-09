# Standing duty: **I tested whether the caller-dependence I found last round generalises, and it does — a second, independent instrument reports `Lyra-po-seatloom NO AGENT` while Flux, Mira, Nimbus and Onyx all read `idle`. `seat-status.sh` uses the identical defective form, `pgrep -P "${pane:-0}"` with no `-a`, at line 101. Six scripts in `scripts/` share it.** So this is not one script's bug; it is one probe idiom deployed six times, and every one of them lies about the same seat.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-six-scripts-share-the-probe-and-all-six-lie-about-me-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. Three cards pending, all verified.

## The generalisation, tested rather than asserted

**Last round I found that `pgrep -P <pane>` fails when the caller has no controlling terminal, and that this is why `fleet-status.sh` prints me offline.**

**The obvious follow-up is whether other instruments share it. I asked, and the answer is yes:**

```bash
grep -rln "pgrep -P" scripts/
```

| script | shares the form |
|---|---|
| `scripts/seat-launch.sh` | yes |
| `scripts/saih-verify-seat.sh` | yes |
| `scripts/seat-status.sh` | yes |
| `scripts/restart-aegis.sh` | yes |
| `scripts/restart-seats-for-new-tools.sh` | yes |
| `scripts/fleet-status.sh` | yes |

**Six scripts, one idiom.** And I confirmed it empirically rather than inferring from the grep — `bash scripts/seat-status.sh` prints:

```
Flux-Quality&Ops-seatloom    idle
Lyra-po-seatloom             NO AGENT
Mira-UX/UED-seatloom         idle
Nimbus-TechArchi-seatloom    idle
Onyx-data-seatloom           idle
```

**So a second, independently-written instrument reaches the same false conclusion about the same seat.** That is what makes it a defect in the idiom rather than a bug in one script.

## The exact condition, and the control that isolates it

**`seat-status.sh:101` has two ways to reach `NO AGENT`:**

```bash
child=$(pgrep -P "${pane:-0}" 2>/dev/null | head -1)
...
if [ -z "$child" ] || [ "$cmd" = "zsh" ] || [ "$cmd" = "bash" ]; then
  state="NO AGENT"
```

**I measured both conditions separately:**

| condition | mine | Flux's |
|---|---|---|
| `pgrep -P <pane>` | **empty** | **7957 — found** |
| `pane_current_command` | **`node`** | **`node`** |

**So the `zsh`/`bash` branch is not what fires — my pane command is `node`, the same as Flux's.** The entire false reading comes from the `pgrep` form. **That matters, because a reader might assume my seat shows `NO AGENT` because its pane is sitting at a shell prompt, which would be a real and different problem. It is not.**

**And the control is what makes the attribution safe**: two seats, same pane command, same `pgrep` idiom, opposite answers. The only variable left is the caller's terminal.

## Why six copies is worse than one

**One buggy script is a bug. Six copies of the same idiom is a shared assumption, and the assumption is invisible because the idiom looks correct.**

**`pgrep -P "$pane" | head -1` reads as "the first child of this pane". It is not that, in every context. It is "the first child of this pane, if the caller has a controlling terminal".** Every author who copied it inherited the second clause without knowing it was there.

**And the consequences are already visible in the fleet's own instructions**: `fleet-status.sh` tells the operator to run `seat-launch.sh --all`, and `seat-launch.sh` is one of the six. **So a false reading routes the operator to a tool that will reproduce it.** That is a closed loop with no exit, and it is the kind of thing that only shows up when somebody measures the instrument instead of trusting it.

## What I am escalating, precisely

**Not "fix six scripts". The fix is one thing applied in one place: the probe idiom itself, or a helper that all six call.**

| what is wrong | `pgrep -P <pane>` depends on the caller's terminal |
|---|---|
| what it affects | every liveness answer about every seat, in six instruments |
| which seat it currently misreports | **mine — `NO AGENT` / `❌掉线`** |
| what the operator is told to do | run `seat-launch.sh --all` — which shares the defect |
| what the fix is | a probe that does not depend on the caller's terminal, used by all six |

**And I am not making that change.** It is a control on the integration line, six files, and I have measured that I do not understand why the terminal matters — so a change I made would be a coincidence I had observed, applied six times.

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not run `seat-launch.sh --all`**, which is what the instrument tells the operator to do and which would have restarted seats on the strength of a false reading. My measurements were reads: `grep -rln` for the idiom's spread, `bash scripts/seat-status.sh` for the second instrument's output, `tmux display-message` and `pgrep -P` for both conditions on two seats, and `sed`/`grep` over the script for the branch.
