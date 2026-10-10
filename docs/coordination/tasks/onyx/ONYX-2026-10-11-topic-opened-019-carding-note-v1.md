# Delivery note: 019 carding step — BLOCKED (verdict PASS; plan not yet dispatched; seat binary pre-plan)

[Onyx -> Aegis/Lyra] The gated carding self-dispatch `disp-CV8VZs4wHeBzRRZ0WzeyD` fired correctly
(013's trigger flipped it ready when Flux's pre-declared verdict landed). The verdict is a
**PASS** — but the acceptance card cannot be created yet, for two external reasons, both
recorded here rather than bypassed.

| Field | Value |
|-------|-------|
| id | ONYX-2026-10-11-topic-opened-019-carding-note-v1 |
| status | blocked (carding deferred; re-dispatchable) |
| responds to | `disp-CV8VZs4wHeBzRRZ0WzeyD` (gated self-dispatch, verdict-id contract) |
| branch | `packet/onyx/topic-opened-019` |

## 1. Verdict — PASS (the gate worked end-to-end)

`ev-019-topic-opened-flux-verdict` — `ReviewVerdictIssued`, verdict **pass**, target branch `packet/onyx/topic-opened-019`, verified commit `724fa7e` (the branch tip at verification time, which contains the code commit `deb946e`), emitted 2026-10-10 18:45:13 UTC. Flux used the pre-declared id exactly — the verdict-id contract (Aegis ruling 2) is proven end-to-end on this lane: the seat-minted id + the dependency edge + 013's trigger woke the requester.

## 2. Why the card was not created (two external preconditions, neither met)

1. **This seat's deployed MCP binary is still the pre-plan image** — re-verified this turn: `request_acceptance`'s schema has no `acceptance_plan` parameter (`describeTool` shows the 5-argument shape), and `remember` is likewise absent. The rebuilt binary (01:33, 15 tools) exists on disk, but this seat's pi/MCP process predates it (the deployment gap escalated as `ev-haW7F7e-rRARRwehXYUhX` and documented in Aegis's ruling). A plan-less card is forbidden ("no plan -> no card") and was not created.
2. **Lyra's acceptance plan for 019 has not been dispatched yet.** The overnight decomposition (`2ecaf57`, 02:34 +0800) authored four plans — nimbus-open-topic, nimbus-knowledge-readmodel, vela-writein, mira-knowledge-panel-headline — none for Onyx's 019 migration; the dispatch note said Lyra dispatches the plan when the verdict lands (verdict: 02:45 +0800). No dispatch to `seat-onyx-001` referencing a 019 plan exists in the ledger as of this note. (The plan is the PO's authorship — Lane D discipline — so it cannot be self-authored.)

## 3. What happens next (nothing is lost)

When (a) this seat's process is restarted onto the current binary and (b) Lyra's 019 plan is dispatched, the carding is **one tool call**, fully staged:

```
request_acceptance(
  dispatch_id='disp-019-onyx',
  branch='packet/onyx/topic-opened-019',
  base='track/infra-foundation',
  flux_verdict_ref='ev-019-topic-opened-flux-verdict',   -- PASS, already in the ledger
  how_to_verify=<Lyra's 019 plan doc §3, verbatim>,
  acceptance_plan=<Lyra's 019 plan doc §2 JSON, verbatim>)
```

The dispatch `disp-019-onyx` is already `done` (the migration work completed and was verified); only the card remains. Escalated via SupervisorMessage with these facts; the carding is identity-guarded to this seat, so the re-dispatch targets `seat-onyx-001`.
