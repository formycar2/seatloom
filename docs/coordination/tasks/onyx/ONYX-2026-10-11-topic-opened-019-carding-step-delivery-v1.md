# Delivery: 019 carding step — plan now on file; call still blocked by this seat's pre-plan binary (report as deliverable)

[Onyx -> Aegis] `disp-019-onyx-card`: Lyra's acceptance plan for 019 has landed
(`docs/coordination/acceptance/2026-10-11-lyra-onyx-019-topic-opened-acceptance-plan-v1.md`,
commit `adbdfcd`), and the staged `request_acceptance` call is verified parameter-by-parameter
against it. The call was **not** made: this seat's MCP process still runs the pre-plan binary
(`request_acceptance` has no `acceptance_plan` parameter — re-verified this turn). Per the
dispatch's instruction, this report is the deliverable; the restart is Aegis's patrol action,
and the carding is one staged call once this seat's process is current.

| Field | Value |
|-------|-------|
| id | ONYX-2026-10-11-topic-opened-019-carding-step-delivery-v1 |
| status | delivered (report; carding deferred on the seat-binary restart) |
| responds to | governed dispatch `disp-019-onyx-card` (019 建卡步) |
| branch | `packet/onyx/topic-opened-019` |

## 1. Preconditions — now both documented, one still operationally open

| precondition | state |
|---|---|
| Flux verdict PASS | ✅ `ev-019-topic-opened-flux-verdict` (commit-pinned `724fa7e`, which contains `deb946e`) — in the ledger since 2026-10-10 18:45 UTC |
| Lyra's acceptance plan | ✅ landed: `2026-10-11-lyra-onyx-019-topic-opened-acceptance-plan-v1.md` (`adbdfcd`), shape v1, §2 JSON + §3 call parameters |
| This seat's binary accepts `acceptance_plan` | ❌ still the pre-plan image (re-verified this turn via `describeTool`: the 5-argument schema; `remember` likewise absent) |

## 2. The staged call, verified parameter-by-parameter against the plan doc §3

```
request_acceptance(
  dispatch_id='disp-019-onyx',
  branch='packet/onyx/topic-opened-019',
  base='track/infra-foundation',
  flux_verdict_ref='ev-019-topic-opened-flux-verdict',
  how_to_verify='scratch 两遍 apply 第二遍零 catalog 写入;event_type 值域 28 值含 TopicOpened,与 Rust EventType::ALL 逐字一致,memory 四类型未被 revert;apply-db.sh 注册就位',
  acceptance_plan={<plan doc §2 JSON, verbatim>})
```

Every parameter matches the plan doc §3 verbatim; the §2 JSON is carried in full in the plan doc (plan_version 1 / scenario / preconditions / 6 steps / 5 expected). No parameter is guessed, no plan-less card is created.

## 3. The ask (Aegis's patrol action)

Restart this seat's MCP process onto the rebuilt binary (the 01:33 image with the 15 tools — the one already on disk). The same restart unblocks: this card, the three staged 015/016/017 cards, and every seat's forward-recording `remember` obligation. Escalated as a SupervisorMessage with this note (`decision_needed_from: Aegis`); the carding is identity-guarded to `seat-onyx-001`, so the re-dispatch targets this seat and takes one call.

## 4. Completion

`complete_dispatch(dispatch_id="disp-019-onyx-card", outcome="done")` — the report is this branch's deliverable, per the dispatch's instruction. The seat never merges its own work; the card merges only on the operator's Accept click.
