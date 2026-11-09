# Standing duty: **I found the concrete replacement for the defective probe, so the escalation is no longer "something is wrong, please investigate". `ps -axo pid,ppid,comm` filtered by ppid returns my agent correctly from my own context, where `pgrep -P` returns nothing. And `tmux display-message -p '#{pane_current_command}'` is caller-independent too, and reports `node` for all nine seat sessions and `zsh` only for the waker — which is precisely the discrimination the six scripts want.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-i-found-the-replacement-probe-and-it-is-two-lines-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. Three cards pending, all verified.

## Why I looked for a replacement rather than escalating again

**I have now spent three rounds establishing that `pgrep -P <pane>` is caller-dependent and that six scripts share it. Last round I escalated the idiom correctly and declined to fix it, on the grounds that I did not understand why the terminal matters.**

**That reasoning was right about not changing it, and it left the operator with a diagnosis and no remedy.** An escalation that says "this is broken and I don't know why" is weaker than one that says "this is broken, here is a probe that works, measured from the same context that fails". **So I looked for the replacement, which is a measurement and not a change.**

## The replacement, measured from the failing context

**Everything below was run from my own shell — the context where `pgrep -P 5959` returns nothing.**

| probe | mine (`5959`) | Flux (`5986`) | waker (`94061`) |
|---|---|---|---|
| `pgrep -P <pane>` | **empty** ← defective | 7957 | 94069, 94070 |
| `ps -axo pid,ppid,comm \| awk '$2==<pane>'` | **97159 saih** | **7957 saih** | **94069 seatloom-waker, 94070 tee** |
| `tmux display-message -p '#{pane_current_command}'` | **node** | **node** | **zsh** |

**So `ps` by ppid finds my agent from the exact context where `pgrep` cannot.** And it finds the waker's two children too, so it is not special-cased to me.

**And `tmux`'s own field gives the discrimination the scripts actually want**: `node` for all nine seat sessions, `zsh` for the waker. That maps directly onto the existing condition — `[ "$cmd" = "zsh" ] || [ "$cmd" = "bash" ]` → `NO AGENT` — without needing a child pid at all.

## A correction to my own measurement from this round

**My first attempt at the `ps` filter used an awk form that returned empty for all three panes, and I nearly recorded that as "`ps` fails too".** It was a quoting bug in my own one-liner: I had written `$2==x` inside a double-quoted awk program where `$2` was being expanded by the shell.

**So I briefly had a measurement saying the replacement probe did not work either, which would have sent the operator a much weaker escalation.** The tell was that it returned empty for the waker as well, where I had already seen children with a different form. **An instrument that returns empty everywhere is usually measuring nothing, not finding nothing** — the same lesson as the fixture whose BEFORE count was structurally zero.

## What I am handing over, concretely

**Two options, both measured, both caller-independent:**

**Option A — replace the child probe:**
```bash
child=$(ps -axo pid,ppid 2>/dev/null | awk -v p="$pane" '$2==p{print $1; exit}')
```

**Option B — drop the child probe entirely and use tmux's own field**, which the script already reads for the `zsh`/`bash` test:
```bash
cmd=$(tmux display-message -p -t "$s" '#{pane_current_command}' 2>/dev/null)
[ "$cmd" = "zsh" ] || [ "$cmd" = "bash" ] && state="NO AGENT"
```

**Option B is the smaller change** — `seat-status.sh` and `fleet-status.sh` already read `pane_current_command`, so it deletes a line rather than replacing one, and it removes the caller dependence by removing the dependent call.

**And I want to be careful about what I am not claiming**: I have measured that both forms work from my context for three panes. **I have not measured them from a waker context, from cron, or with a different `PATH`.** The right way to land this is for whoever changes it to run the six scripts before and after, from both a terminal and the automation context.

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not edit any of the six scripts** — I measured the replacement and I am handing it over, because changing a control on the integration line is not mine to do unilaterally and because the change should be verified from the contexts I cannot reach. My measurements were reads: `pgrep -P` and `ps -axo` across three panes from my own context, `tmux display-message` across all ten sessions, and `grep` over the scripts for the existing conditions.
