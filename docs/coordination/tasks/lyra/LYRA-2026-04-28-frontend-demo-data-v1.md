# LYRA-2026-04-28-Frontend-Demo-Data-v1

| Field | Value |
|---|---|
| Owner | Lyra |
| Scope | Frontend demo seed for `seatloom-core` |
| Source basis | `docs/coordination/memory/2026-04-28.md`, Flux acting-Mira packet, Flux delivery note, Flux feedback recovery review |
| Target file | `ui/src/stores/useDataStore.ts` |
| Status | Seed authored |

## 1. Purpose

Seed the current frontend with one coherent `2026-04-28` story so the product surfaces read like the real coordination day instead of unrelated placeholder data.

## 2. Coverage map

| UI surface | Seeded story |
|---|---|
| Sidebar | Lyra / Nimbus / Mira / Flux seat state, including Mira paused and Flux active |
| Terminal | Lyra + Flux running sessions, Mira interrupted, Nimbus suspended |
| Inbox | Pending SG-01 acceptance, Inbox ID normalization defect, Mira limitation context, Nimbus handoff block |
| Timeline | Packet issue, seat interruption, acting-Mira repair session, delivery artifact, recovery note, repair note handoff |
| WorkItems | Repair, acceptance review, normalization fix, Nimbus handoff dependency, daily writeback |
| MorningDigest | 4 sessions, 5 goals, 3 handoffs, 1 interrupted session, 1 blocked goal |

## 3. Modeling rules

1. Keep the seed contract-focused, not generic productivity noise.
2. Reflect today's governance state: `SG-01` still on hold, Nimbus still blocked.
3. Represent Mira as paused/offline and Flux as the temporary acting-Mira executor.
4. Prefer exact IDs/timestamps so Inbox and Timeline interactions are demonstrable.
5. Keep references aligned so Inbox -> Detail and Sidebar -> Timeline remain testable.

## 4. Notes

- The seed is intentionally scenario-driven for review/demo use.
- This artifact does not change product meaning; it only replaces mismatched placeholder data with a day-consistent narrative.

