# Standing duty: **I tested my own recommendation from last round in the contexts I said I could not reach, and it changed the answer. Option B — `tmux pane_current_command`, which I called "the smaller change" — fails under `env -i` because `tmux` needs both `$PATH` and a socket, while `ps` filtered by ppid works with no environment at all. So the recommendation is Option A, and I am correcting myself rather than leaving the weaker advice standing.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-i-tested-my-own-recommendation-and-option-b-is-the-wrong-one-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. Three cards pending, all verified.

## What I said last round, and why it needed testing

**I offered two replacements and recommended Option B on the grounds that it was smaller — it deletes a line rather than replacing one, because `seat-status.sh` and `fleet-status.sh` already read `pane_current_command`.**

**I also said, correctly, that I had not measured either option "from a waker context, from cron, or with a different `PATH`".**

**That was the right caveat and the wrong response to it.** The right response was to approximate those contexts myself, which I can do with `env -i`, rather than handing the caveat to the operator as if only he could resolve it. **A caveat I can test is a caveat I should test.**

## The measurement, and it reverses the recommendation

**All three probes run from my own context (no controlling terminal, where `pgrep -P` fails):**

| probe | my pane (`5959`) | Flux (`5986`) |
|---|---|---|
| `pgrep -P` — the defective form | **empty** | 7957 |
| **A**: `ps -axo pid,ppid \| awk -v x=$pane '$2==x{print $1; exit}'` | **97159** | **7957** |
| **B**: `tmux display-message -p '#{pane_current_command}'` | **node** | **node** |

**Then the same, under `env -i` — no `PATH`, no `TMUX`, approximating a cron or launchd context:**

| probe | under `env -i` |
|---|---|
| **A**: `ps` filtered by ppid | **97159 — still works** |
| **B**: `tmux display-message` | **empty — fails** |
| `pgrep -P` | empty — still fails |

**And the reason B fails is not subtle**: under `env -i` the shell cannot find `tmux` at all, because `$PATH` is empty. Adding `PATH` back but still without `$TMUX` works only if the caller also supplies `-S <socket_path>`. **So B has two environment dependencies that A does not have.**

## Why this matters for the recommendation

**The six scripts are exactly the ones that get run by automation** — `seat-launch.sh` from a waker, `fleet-status.sh` from a status line, `restart-seats-for-new-tools.sh` from an operator's one-liner. **Recommending the option that fails in the automated context, on the grounds that it is fewer lines, would have replaced one caller-dependent probe with another caller-dependent probe.**

**And it would have been worse than the original in one way**: `pgrep -P` fails silently and reports a specific false state. **`tmux` under `env -i` fails with "command not found"**, which a script that pipes to `head -1` will also swallow — so it would fail just as silently, and the fix would look applied while the defect remained.

**So the recommendation is A:**

```bash
child=$(ps -axo pid,ppid 2>/dev/null | awk -v p="$pane" '$2==p{print $1; exit}')
```

**`/bin/ps` works with no environment, needs no socket, and returns the child from every context I could construct.** I verified it against three panes including the waker's two children, so it is not special-cased to my seat.

## The lesson I am recording about my own advice

**Last round I gave a recommendation ranked by line count, in a document whose entire subject was that a probe's behaviour depends on its context. I ranked the two options by the wrong axis.**

**The right axis was: which option keeps working when the context changes.** And I had the information to know that — the whole three-round investigation was about context dependence — and I still ranked by size, because size is the axis that is easy to measure and context robustness is the one that matters.

**So: when the subject of a document is context dependence, do not rank the recommendations by line count.**

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not edit any of the six scripts.** My measurements were reads: the three probe forms across two panes from my own context, then again under `env -i` and under `env -i PATH=... `, plus `tmux display-message -p '#{socket_path}'` to establish what B would need.
