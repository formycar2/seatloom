# Standing duty: **`fleet-status.sh` reports me `❌掉线` — offline — while I am demonstrably running and answering dispatches. I found the mechanism and it is an instrument defect, not a fleet condition: the script probes `pgrep -P "$pane"` without `-a`, and on my pane that form returns nothing while `pgrep -P "$pane" -a` returns my own pid. Reproduced three times, stable.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-fleet-status-says-i-am-offline-and-i-am-not-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. Three cards pending, all verified.

**And `bash scripts/fleet-status.sh` prints `lyra [❌掉线] 产品负责人 — 空闲`, with `在线 5/6`.**

## The claim is false, and I can prove it two ways

**First, the obvious way: I am executing this round.** I read the ledger, called `get_next_dispatch`, ran `git` and `psql`, and I am writing a document. A seat that is answering is not offline.

**Second, by measurement:**

| check | measurement |
|---|---|
| my tmux session | `Lyra-po-seatloom`, created 11-02, present |
| my pane pid | **5959** |
| `ps` children of 5959 | **`97159 saih`** — that is me |
| `ps -p 97159` | **`S+`, uid 501, tty ttys001, 19.3% CPU** |

**So the process exists, is a child of my pane, and is burning CPU right now.**

## The defect, isolated

**The script's probe, at `scripts/fleet-status.sh:95`:**

```bash
pane=$(tmux list-panes -t "$sess" -F '#{pane_pid}' 2>/dev/null | head -1)
proc=""; [ -n "$pane" ] && proc=$(pgrep -P "$pane" 2>/dev/null | head -1)
if [ -n "$proc" ]; then ... up="在线"; else up="❌掉线"; fi
```

**And the two forms disagree on my pane:**

| invocation | result |
|---|---|
| `pgrep -P 5959` | **empty** |
| `pgrep -P 5959 -a` | **`97159`** |

**Three consecutive samples, stable.** So the script's `proc` is empty, `alive` is not incremented, and I am printed as offline.

**And I confirmed it is not a general `pgrep` failure**: `pgrep -P 5958` returns 66 processes, and `pgrep -P 5964` returns 7690 correctly. **The probe works for other seats — which is why the other five are `在线` and only I am not.** Whatever the cause, it is specific to the process the probe is trying to find.

**I tried to pin the mechanism and I could not, and I am recording that rather than guessing.** I tested and discarded three hypotheses: that it was a zombie (it is `S+`), that it was a process-group leader (`7690` also has `pgid==pid` and is seen), and that it was a session leader (the `sess` field reads 0 for both). **The one remaining correlation is that mine is the process consuming CPU** — 19.3% against 0.0% for the one that is seen — and I have not established that as causal. **A hypothesis I have not tested is not a finding, and I would rather report the reproducible fact than a plausible cause.**

## Why this matters more than a cosmetic label

**The instrument prints `在线 5/6` and tells the operator to run `bash scripts/seat-launch.sh --all`.** So a false offline reading produces an instruction to take action on a seat that is working. **Reporting a problem that is not there pushes the operator to break what works**, and this one would have him relaunch a seat that is mid-conversation with him.

**And it is the same failure family as the ones I have catalogued all week: an instrument answering about the set it can see, presented as an answer about the set that exists.** `pgrep -P` without `-a` sees a subset; the script presents the subset as the seat's liveness.

**There is also a narrower reading worth stating**: the probe asks "does this pane have a child process", which is a proxy for "is the seat running". **For the five seats whose agent is a plain child of the shell the proxy works. For me it does not**, because of whatever property makes `pgrep` skip me. **A proxy that works for five of six is not a liveness check, it is a check that happens to work for the shape it was written against.**

## The fix is small and it is not mine

**Adding `-a` would make the probe find me.** That is a one-word change to a script on the integration line, which is not mine to make unilaterally — and it would also be papering over a `pgrep` behaviour nobody understands yet. **The better fix is to ask what liveness actually means for a seat**, given that at least two different process shapes are in use and the probe only recognises one.

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. My measurements were reads: `bash scripts/fleet-status.sh`, `tmux list-panes`, `ps` in several field combinations, `pgrep -P` with and without `-a` across several parents, and `grep` over the script for its probe. **I did not edit `fleet-status.sh`, did not restart my own pane, and did not relaunch anything.**
