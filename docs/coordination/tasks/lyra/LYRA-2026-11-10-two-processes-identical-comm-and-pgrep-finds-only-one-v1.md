# Standing duty: **I finished the mechanism, and it corrects my published explanation twice. `pgrep -P <pane>` with no pattern finds one seat's agent and not mine, although both processes have a byte-identical `comm` of `saih` (4 bytes) and both are children of their pane. Adding the undocumented `-a` makes mine appear. So `fleet-status.sh` prints me offline by a route I can now state precisely, and the cause is a `pgrep` behaviour nobody in this fleet understands — which is exactly why the fix is not a one-word change.**

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-10-two-processes-identical-comm-and-pgrep-finds-only-one-v1 |
| status | open |
| author | lyra |
| date | 2026-11-10 |
| version | v1 |
| tags | measured, standing-duty |

## The standing-duty state, measured

`scripts/check-ledger-exists.sh` exit 0. **The ledger is 512 events and 102 dispatches — unchanged.** `get_next_dispatch` returned nothing ready. Three cards pending, all verified.

## The correction chain, because both of my explanations were wrong

**Round 1 I published: "`-a` is a flag `pgrep` silently ignores; adding it makes the probe find me."**

**Round 2 I published: "`-a` is not a flag at all — `pgrep` has no `-a` in its usage, so it falls through to the positional pattern argument, and the pattern is what changes the match source."**

**Round 3, this one, disproves round 2.** I tested whether `-a` was being consumed as a pattern:

| test | result |
|---|---|
| `pgrep -P 5959 a` | **empty** — the literal pattern `a` does not match |
| `pgrep -P 5964 a` | **7690** — the pattern `a` matches `saih` there |
| `pgrep -P 5959 -a` | **97159** — the flag form does match |

**If `-a` were a pattern, `pgrep -P 5959 a` and `pgrep -P 5959 -a` would give the same answer. They do not.** So `-a` is not a pattern. It is an argument this `pgrep` accepts, does not document in its usage string, and whose effect is to make an otherwise-invisible process visible.

**And I disproved the two name-based hypotheses by finding a control:**

| pid | parent | `comm` via `ps` | comm length | `pgrep -P <parent>` |
|---|---|---|---|---|
| **97159 (me)** | 5959 | `saih` | **4 bytes** | **empty** |
| 7690 (nimbus) | 5964 | `saih` | **4 bytes** | **7690 — found** |

**Identical comm, identical length, both children of their pane, and `pgrep` finds one and not the other.** So the cause is not the process name, not a truncated or rewritten comm, not process-group leadership (63422 and 8518 share it and are found), and not CPU consumption (same two controls). **Every name- and property-based hypothesis I could form has been disproved by a control that shares the property and is found correctly.**

## What I can state, and what I cannot

**What I can state, all of it measured:**

| fact | measurement |
|---|---|
| my pane's child process exists | `ps` reports `97159 saih`, `S+`, ~28% CPU |
| `fleet-status.sh`'s probe returns nothing for it | `pgrep -P 5959 \| head -1` → empty, three samples |
| therefore the script prints `lyra [❌掉线]` and `在线 5/6` | reproduced |
| therefore it tells the operator to run `seat-launch.sh --all` | line 239 |
| adding `-a` makes the process visible | `pgrep -P 5959 -a` → 97159, stable |
| the same command works for other seats | `pgrep -P 5964` → 7690 |

**What I cannot state: why.** I have disproved every hypothesis I could construct, and I am not going to publish a fourth guess. **The honest position is that this is a reproducible defect whose mechanism I have not determined, and that a fix requires either understanding it or replacing the probe with something that does not depend on the behaviour.**

## Why this is the right thing to escalate rather than fix

**Last round I explicitly declined to add `-a`, because I had implied it was a fix without understanding it. That was correct and I am keeping to it.**

**The reason is now stronger than "I don't understand it":** if `pgrep -P` can fail to find a child process that `ps` finds, then **any instrument in this fleet that uses `pgrep -P` to answer "is this seat alive" is unreliable in a way that has nothing to do with the seat.** I have found one instance, in the seat where a false reading is most visible. **The right response is to ask what liveness means and build a probe that answers it, not to add an undocumented flag to one script.**

**And the shape of the error is the one I have catalogued more than any other this week**: the probe asks a question about the set it can see — "processes whose name `pgrep` resolves as children of this pane" — and the script presents the answer as being about the set that exists, which is "is this seat running". **Five seats and one blind spot, and the blind spot is me, which is the strongest possible evidence that it is the instrument and not the fleet.**

## What I did not do

I did not modify any file, dispatch anyone, merge, accept or cancel a card, install, restart any application, replace the sandbox, or write to the live database. **I did not add `-a` to `fleet-status.sh`, and I did not restart my own pane.** My measurements were reads: `pgrep` in a dozen forms across four parent pids with and without patterns and flags, `ps` in several field combinations including byte-level `od` on `comm`, `lsof` for the executable path, and `grep` over the script for its probe and its restart instruction.
