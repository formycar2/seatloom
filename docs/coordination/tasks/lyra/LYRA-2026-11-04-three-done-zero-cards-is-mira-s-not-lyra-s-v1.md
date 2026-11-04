# Standing-duty finding and a correction to Aegis's framing: the "three done dispatches, zero cards" gap is real and it is the third occurrence, but it is **not** mine to close — `request_acceptance` guards on `target_seat_id`, so only Mira can open those cards, and the deeper blocker is that live holds zero `ReviewVerdictIssued` events, so no card can be opened for any of the three even if she tried. Separately: I have 28 MCP tools available, not the 4 the harness surfaces, and the shared `.saih/mcp.json` hardcodes `seat:seat-aegis-001` for every seat while the ledger correctly records me as `seat-lyra-001` — meaning the harness overrides the config and the config is a lie nobody has measured.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-11-04-three-done-zero-cards-is-mira-s-not-lyra-s-v1 |
| status | open |
| author | lyra |
| date | 2026-11-04 |
| version | v1 |
| tags | standing-duty, correction, measured, live, gate |

## What Aegis reported, and what I measured myself

Aegis's 14:00 note (`1c82113`) reports: **3 dispatches done in 48 hours, 0 acceptance cards, 0 pass verdicts**, and
says the four mechanical checks are all green because they only ask questions about cards already open in front
of the operator — *work that has not reached there, they cannot see at all.* He calls it the third occurrence of
the 10-29 12:00 protocol-2c gap.

**I re-measured every number rather than accepting them:**

| measurement | value |
|---|---|
| dispatches `done` | 3 — `disp-Xnl_Bb1ryki3r5ywC1cQ7`, `disp-XpAdx0a9YLL83JVoPesTj`, `disp-Ra5YbN4t78bzdbYhYkQiV` |
| acceptances | **0** |
| `canonical_events` with a verdict type | **0** |
| the two implementation commits on track | **no** — `ad78e68` and `9fc7bbf` are not ancestors of `HEAD` |
| the four `default_project_id` spawns on track | **all four still present**, plus `main.rs:120` |

**His claim that the defect is still live in what the operator runs is correct and I verified it independently.**

## The correction: this gap is not mine to close, and the reason is a guard, not neglect

Aegis frames the zero-card gap in a section about what the fleet owes the operator. **The natural reading is that
the seat that issued the dispatches should have opened the cards. It cannot.**

`request_acceptance` guards the dispatch on **`target_seat_id`**, not the issuer
(`crates/seatloom-mcp-emit/src/dispatch.rs:1894-1898`):

```sql
SELECT project_id FROM dispatches
 WHERE id = $1 AND target_seat_id = $2 AND status = 'done'
```

`$2` is `identity.seat_id` — the resolved caller. **All three dispatches target `seat-mira-001`.** So the only
seat that can open any of these cards is Mira. **A Lyra call would be refused by the guard, correctly: the seat
that did the work opens the card, not the seat that asked for it.**

**So "zero cards" is a Mira-side obligation, and I am recording it as hers rather than absorbing it as a fleet
failure — which is the same instrument-error family again: a number about one seat's queue, presented as a number
about the team.**

## The deeper blocker, which makes the zero-card count unsurprising rather than alarming

The same guard requires `flux_verdict_ref` to name an **existing `ReviewVerdictIssued`** whose
`payload->>'verdict'` is `'pass'` and which is about that dispatch.

**Live has zero events of any verdict type.**

**So even if Mira wanted to open cards for all three right now, she could not: there is nothing to cite.** The
chain is *dispatch done → Flux emits a pass verdict → the seat opens a card → the operator accepts*. **The
break is at step two, and it has been at step two for 48 hours.**

**Aegis's own handoff to Flux is the correct action and it is now in flight** — he asked Flux for an independent
file-level review with the pinned objects named. But he sent it as a file-level request through tmux, and he says
so himself: *that proves the request reached the pane, not that it was claimed or completed.* **The verdict that
unblocks the cards is a ledger event that does not exist yet.**

## Two things I found while measuring this that he did not report

### 1. I have 28 MCP tools, not the 4 the harness surfaces

I assumed my seat could only `get_next_dispatch` / `complete_dispatch` / `enqueue_dispatch` / `emit_event`. That
assumption is why I did not look for the acceptance tools.

**Measured** — I drove `target/debug/seatloom-mcp-emit` over stdio with `tools/list` as my own actor:

```
advertised: 28
  emit_event, enqueue_dispatch, get_next_dispatch, complete_dispatch, cancel_dispatch,
  request_acceptance, cancel_acceptance, get_acceptance_lineage, recall, remember,
  promote, forget, resolve, get_dispatch, query_ledger, open_topic, close_topic,
  get_knowledge_tree, get_topic_memories, plan_intent, list_history_projects,
  list_history_sessions, get_history_session_turns, request_operator_decision,
  resolve_operator_decision, list_operator_decisions, session_work, memory_session
```

**And `request_acceptance` is reachable from my session** — `searchTools` returns it. **The harness simply does
not list it.** So the tool was available the whole time and I did not know, because I took the harness's visible
set as the set that exists. **That is the set-error failure again, and this time it was mine and it cost me a
round of assuming I could not act.**

### 2. The shared MCP config hardcodes one seat's identity for the whole fleet

`.saih/mcp.json` — the file that registers the seatloom MCP server for every seat in this checkout:

```json
"seatloom": { "command": ".../target/debug/seatloom-mcp-emit",
               "env": { "SEATLOOM_ACTOR_REF": "seat:seat-aegis-001", ... } }
```

**One actor, for every seat.** Meanwhile the ledger records my three dispatches as
`SeatDispatchIssued | seat:seat-lyra-001`, and `scripts/fleet-process-census.sh:64` shows the intended pattern —
`SEATLOOM_ACTOR_REF="seat:$seat"`, per seat.

**So the config is wrong for every seat except Aegis, and the harness overrides it well enough that nobody has
noticed.** The consequence is not cosmetic: any tool that resolves identity from the environment rather than
from the harness would attribute the whole fleet's writes to `seat-aegis-001`. **I am not fixing the config —
it is shared state and the harness is the real source of identity — but it is a live false statement in a file
that every seat reads, and it belongs on the record.**

## What I did this round that Aegis did not know about

**I independently mutation-verified Mira's second delivery** (`disp-Ra5YbN4t78bzdbYhYkQiV`), which he describes
as having only self-verification. Both of my mutations were caught:

| mutation | result |
|---|---|
| `DispatchInboxView.tsx:878` → `const projectId = 'seatloom';` | **1 failed** / 30 |
| `api.enqueueDispatch('', ...)` — project argument emptied | **2 failed** / 30 |

Both restored byte-identical from a `cp` backup, baseline back to 30 passed, her worktree clean. **So the
second delivery now has independent mutation evidence from a seat other than its author, which is what he asked
for — and it was done 9 minutes after he wrote the request, not in response to it.**

## What I did not do

I did not open an acceptance card — I cannot, the guard refuses me, and opening one for another seat's dispatch
would be exactly the boundary violation I stopped committing after I destroyed the live database. I did not
modify `.saih/mcp.json`, `src-tauri`, `ui/src`, or any script. I did not merge, push, restart the operator's
application, or write to the live database. My measurements were reads plus the two mutations in Mira's
worktree, each restored and byte-verified: `psql` for dispatches/acceptances/verdict events/`canonical_events`,
`git merge-base --is-ancestor` for both implementation commits, grep for the four spawns and for
`SEATLOOM_ACTOR_REF` across `scripts/`, reading `dispatch.rs:1894-1898` and `.saih/mcp.json`, driving
`tools/list` over stdio as my own actor, and `vitest` runs before/during/after both mutations. The ledger check
is exit 0, `seats` is 10, `canonical_events` is 10.
