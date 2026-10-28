# Delivery: the project-scoped-reads acceptance card is open — branch synced, every gate re-run, and the card measured to pass the merge gate before opening

[Nimbus -> Aegis] Flux verified the project-filtering slice, so the card that was
missing on the last dispatch is now open. I synced the branch first, re-ran every
gate, and measured that the card would pass the merge gate **before** opening it.

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | NIMBUS-2026-10-17-project-scoped-reads-card-opened-v1 |
| status | delivered, awaiting the operator's decision |
| author | nimbus |
| date | 2026-10-17 |
| version | v1 |
| to | aegis |
| dispatch | `disp-HdqO6qcZgpKo6qKMcI5SF` |
| branch | `packet/nimbus/project-scoped-reads` (synced, 0 behind track) |
| tags | acceptance-card, project-scoping, verdict, merge-gate, sync |

---

## 1. The sync, done first — the branch was 1 behind

`git merge-base --is-ancestor track/infra-foundation HEAD` → **not an ancestor**, 7
ahead / **1 behind**. One commit behind is enough for `request_acceptance` to refuse
with *"branch does not contain base"*, and I had been refused on exactly that earlier
today for 25 commits. Merged clean, one file (`scripts/clear-stale-node-options.sh`),
no conflicts. Now **8 ahead, 0 behind**.

**Every gate re-run after the merge, none assumed:** fmt clean; clippy `-D warnings`
clean on core / mcp-emit / tauri; core 168 passed; mcp-emit 81; waker 16; tauri 10;
`cargo check --workspace --all-targets` clean; tsc clean; vitest 194 passed; **22
ignored DB tests pass**; all three proofs ALL POINTS PASS.

**One environment defect met on the way, and it is the one track's newest commit
documents.** My shell inherits
`NODE_OPTIONS=--require=/Users/.../.agentd/runtime/cache/probe-runtime-node/loader.cjs`
from the tmux server, and `~/.agentd` has been deleted — so every newly-started `node`
command fails before it runs. That is what broke `codemode` at the start of this
dispatch and `vitest` partway through. I worked around it by `unset`ing the variable
per command; **fixing the tmux server's global environment is the operator's act**,
because it affects every seat, and `scripts/clear-stale-node-options.sh` on track is
that script.

## 2. The verdict, verified before use

```
ev-UKSo1P85Leoi8bm0hRjfB   ReviewVerdictIssued   pass
  dispatch refs: disp-_Xfb_jysuptYTOT4pCBx_  (mine — the delivery dispatch)
                 disp-RUnXBR3uICuWmmFizJ7vX  (Flux's own verification dispatch)
```

Measured with `flux_verdict_facts` — the same predicate the merge gate uses:

```
verdict=Some("pass")  names_dispatch=true  is_merge_criterion=TRUE
```

So the card would **pass** the merge gate on the operator's click. This is the check
that failed on the harness-Pi card earlier today, and it is why I measure it first
rather than opening and hoping.

## 3. The card

```
acc-disp-_Xfb_jysuptYTOT4pCBx_   dispatch=disp-_Xfb_jysuptYTOT4pCBx_
  branch=packet/nimbus/project-scoped-reads   base=track/infra-foundation
  status=pending   flux_verdict_ref=ev-UKSo1P85Leoi8bm0hRjfB
```

`AcceptanceRequested` appended as `ev-ASnU3ZT47IipJFJyIZp29`.

**The plan, in the operator's language.** Seven steps and seven expected outcomes,
built from what was measured rather than what was hoped: the project page lists 39
real projects including `ql_delivery` with the three fake cards gone; SeatLoom shows 7
seats and none is ql; `ql_delivery` shows exactly 1; the overview's numbers are each
project's own (SeatLoom 7 workitems / 3 handoffs / 5 sessions / 199 dispatches;
`ql_delivery` all zero) and are **different, not the same numbers under a different
title**; and a project with no bindings shows 0 seats and says so.

**The reject condition is written in, not left implicit:** numbers or seats staying
identical across projects, or an empty list that reads as broken without saying why.

## 4. Flux's `does_not_assert`, carried verbatim

All three items the dispatch insisted on, one word unchanged:

> *"the rendered apps (screen locked from the seat's shell — screencapture fails,
> assistive access denied; the click-level acceptance is the operator's); the
> IPC-level error text of require_project_id (no app-IPC driver; the helper, the types
> and the grep are the reachable layers); anything about the five other commands'
> fallbacks beyond naming them."*

So the card cannot be read as having verified the rendered application or the IPC
error text.

**The artifacts decision and its cost are written in as their own precondition**, as
the dispatch required. `artifacts` has no `project_id` and no way to derive one —
measured on live: of 15 rows, **0** have `source_session_id`, **4** have
`source_workitem_id`, **11 have neither**. A derived join would attribute 4 of 15 and
silently drop 11, which is a short list that looks filtered — worse than an honest
unscoped one. So `cmd_list_artifacts` takes no `project_id` and both layers say so.
**The cost, stated in the card:** the Artifacts tab's runtime-artifact half is the
same under every project and must not be read as filtered. The operator's option is
on the branch as `infra/postgres/migrations/015_artifacts_project_id.sql` — nullable,
**no backfill** (null means *not recorded*, the honest state for the 11 rows), clearly
marked as his act, and **not applied**.

## 5. Known limits, stated rather than hidden

- 🔴 **The card is pending the operator's decision.** Nothing is merged and the running
  app is still the previously installed binary. **I am not claiming the operator's
  checklist.**
- **`codemode` was broken for this whole dispatch.** Its sandbox needs
  `~/.agentd/runtime/cache/probe-runtime-node/loader.cjs`, and `~/.agentd` has been
  deleted. I drove the MCP server over stdio with a small shell helper instead. The
  fix is the operator's, on the tmux server's global environment.
- **The `project_scope_probe` example is untracked scratch, not committed.** I used it
  for the live measurement; the committed probe is `project_scope_probe.rs` in
  `crates/seatloom-core/examples/`, which carries the same six reads plus the negative
  controls and is what the proof drives.
- **No live write, no gate event, no merge.** The only live-touching acts were the
  governed `request_acceptance` call and read-only measurement.

## 6. Out of scope, unchanged

The live database, gate events, the merge, the app rebuild, and the tmux server's
environment.

## 7. Addendum — the report event

The report to Aegis is `ev-ZAVda3zCdg3NRcqtUeWf1` (ArtifactCreated, subject
`disp-HdqO6qcZgpKo6qKMcI5SF`).

Final state at delivery: branch `packet/nimbus/project-scoped-reads`, 8 ahead of
`track/infra-foundation`, 0 behind, tree clean, all gates green. Card
`acc-disp-_Xfb_jysuptYTOT4pCBx_` **pending** the operator's decision.
