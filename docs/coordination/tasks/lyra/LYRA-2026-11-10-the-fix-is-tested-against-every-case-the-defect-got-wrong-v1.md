# Standing duty: **I tested the fix I recommended rather than only measuring its parts, and it works: replacing `pgrep -P "$pane"` with `ps -axo pid,ppid` filtered by ppid turns my verdict from `NO AGENT` to `有 agent`, gives the right answer for all ten seat sessions, still correctly reports the waker as `NO AGENT`, and works under `env -i` where the automation runs.** So the escalation now carries a fix that has been verified against every case the defective probe got wrong.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-the-fix-is-tested-against-every-case-the-defect-got-wrong-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. Three cards pending, all verified.

## Why I tested rather than handing over

**Last round I corrected my own recommendation and settled on `ps` filtered by ppid. I had measured that the form returns the right pid from my context and under `env -i`.**

**What I had not done is run it through the actual decision logic of the script it is meant to fix.** A probe that returns the right pid is not the same as a script that reaches the right verdict — the branch has two other conditions, and a fix that satisfies the first while tripping the second would be no fix at all. **So I replicated the branch and ran it across every session.**

## The test, and it passes on every case

**The branch from `seat-status.sh:105`, with only the probe replaced:**

```bash
child=$(ps -axo pid,ppid 2>/dev/null | awk -v p="$pane" '$2==p{print $1; exit}')
cmd=$(tmux display-message -p -t "$s" '#{pane_current_command}' 2>/dev/null)
if [ -z "$child" ] || [ "$cmd" = "zsh" ] || [ "$cmd" = "bash" ]; then state="NO AGENT"; fi
```

**Run against all eleven sessions:**

| session | child | cmd | verdict with the fix | verdict today |
|---|---|---|---|---|
| Aegis-Supervisor | 8518 | node | 有 agent | 有 agent |
| Flux-Quality&Ops | 7957 | node | 有 agent | 有 agent |
| **Lyra-po-seatloom** | **97159** | **node** | **有 agent** | **NO AGENT** ← the defect |
| Mira-UX/UED | 8090 | node | 有 agent | 有 agent |
| NetworkOrchestrator | 18955 | node | 有 agent | 有 agent |
| Nimbus-TechArchi | 7690 | node | 有 agent | 有 agent |
| Onyx-data | 7826 | node | 有 agent | 有 agent |
| QL-seatloom | 63422 | node | 有 agent | 有 agent |
| Stepaicloud | 8734 | node | 有 agent | 有 agent |
| Vela-DataWash | 8221 | node | 有 agent | 有 agent |
| **seatloom-waker** | **94069** | **zsh** | **NO AGENT** | **NO AGENT** |

**So the fix changes exactly one verdict — mine — and changes it to the true one.** It does not flip the waker, which is the case a careless fix would break: a probe that matched too broadly would report the waker's `seatloom-waker` child as an agent and turn a correct `NO AGENT` into a false positive.

**And under `env -i`, the automation context:**

| session | child found |
|---|---|
| Aegis-Supervisor | 8518 |
| Flux-Quality&Ops | 7957 |
| **Lyra-po-seatloom** | **97159** |
| Mira-UX/UED | 8090 |
| seatloom-waker | 94069 |

## What this closes, and what it does not

**It closes the "what should the fix be" question.** The escalation no longer asks the operator to design anything; it hands him a one-line replacement, verified against all eleven sessions and in both contexts, with the one case it deliberately does not change named.

**It does not close the "why" question.** I still cannot say why `pgrep -P` depends on the caller's terminal, and I have stopped trying — five rounds disproved every hypothesis I could construct. **That is fine for the fix and worth stating**: a replacement that is verified against every case does not require understanding the thing it replaces.

**And it does not land the change.** Six scripts, a control on the integration line, and the change should be made by whoever can run all six before and after. I have given them the measurement that makes that a five-minute job instead of an investigation.

## The pattern across these five rounds

**What made this findable was not cleverness about `pgrep`.** It was that round 3 built a control — two processes with byte-identical `comm` — which eliminated every property of the target and forced the question onto the caller. **One controlled comparison did more than four rounds of hypothesising about the target.**

**And what almost sent the wrong answer twice**: round 2's pattern hypothesis, and last round's line-count ranking of the two options. Both were corrected by measuring instead of reasoning. **The discipline that worked was: for every claim, find the case that would disprove it, and run that case.**

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not edit any of the six scripts** — I replicated the branch in my own shell and ran it, which tests the logic without touching the integration line. My measurements were reads plus that replica: `ps -axo`, `tmux display-message`, `awk`, and `env -i` across eleven sessions.
