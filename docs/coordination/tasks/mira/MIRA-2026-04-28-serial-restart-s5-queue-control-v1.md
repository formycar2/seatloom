# MIRA-2026-04-28-Serial-Restart-S5-Queue-Control-v1

| Field | Value |
|---|---|
| Owner | Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | Immediate |
| Acceptance owner | Lyra |
| Stage | Serial restart / step 5 queue |
| Working project | `seatloom` |
| Working path | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom` |
| Execution mode | Queued micro-slices / max 3 active tasks |

## 1. Purpose

Stabilize the post-scan UI realignment into a small queued set of implementation slices.
This queue replaces ad-hoc broad patches with bounded packets that can be processed gradually.

## 2. Current acceptance state

- `S5A Handoff State Strip` is accepted as completed.
- The latent `Working` handoff label fix in `ui/src/utils/display.ts` is accepted as a safe build-unblocking exception.
- Do not reopen `S5A` unless Lyra explicitly asks.

## 3. Active queue

You now have these packets available:

1. `S5B` `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5b-workitem-review-tier-strip-v1.md`
2. `S5C` `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5c-session-prompt-blocked-surfaces-v1.md`
3. `S5D` `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5d-seat-card-capability-truth-v1.md`
4. `S5E` `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5e-session-continuity-preview-v1.md`
5. `S5F` `docs/coordination/tasks/mira/MIRA-2026-04-28-serial-restart-s5f-seeded-visibility-patch-v1.md`

## 4. Concurrency rule

- Hard maximum: **3 active packets at once**.
- Preferred mode: start with **1 active packet**, increase only if the seat remains stable.
- Never edit the same file from two active packets.
- If the CLI becomes unstable, drop back to strict single-packet execution immediately.

## 5. Dependency rule

- `S5B`, `S5C`, and `S5D` may run in parallel because they do not share file ownership.
- `S5E` depends on `S5C` completion because both touch `ui/src/components/SessionDetail.tsx`.
- `S5F` depends on `S5B` + `S5C` + `S5D` + `S5E` completion because it seeds proof states for the repaired surfaces.

## 6. Reply rule

After each packet:

1. stop,
2. run the packet validation,
3. send the tmux reply in that packet's exact format,
4. wait for Lyra acceptance before opening a fourth active slice or revising finished files.

## 7. Guardrails

- No broad redesign.
- No speculative data-model expansion.
- No hidden concurrency beyond the stated limit.
- Keep file ownership per packet intact.
- If a packet scope needs to expand, stop and report before changing extra files.
