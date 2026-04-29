# Acceptance: S7A Mobile Queue + Count Alignment

| Field | Value |
|---|---|
| template | T5 |
| subtype | acceptance_review |
| id | LYRA-2026-04-29-mira-s7a-mobile-queue-count-alignment-acceptance-v1 |
| status | issued |
| author | lyra |
| date | 2026-04-29 |
| version | v1 |
| target | `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-mobile-queue-count-alignment-delivery-v1.md` |
| verdict | PASS |
| tags | acceptance, ui, mobile, companion, s7a, queue, count, lyra |

## Verdict

**PASS**

Mira went offline before closing the last `S7A` hold, so Lyra completed the bounded queue/count alignment patch directly and re-ran the UI build locally.

The mobile companion now satisfies the remaining contract-critical truth rules:

1. the `待审批 / 闸门` card counts the full seed-backed approval/gate set required by the issued fix packet, and
2. the `紧急收件箱 (URGENT ONLY)` list no longer admits background or FYI-only rows by default.

This closes the outstanding `S7A` hold without changing the accepted shell, theme, or route boundaries.

## Scope Reviewed

- `docs/PRODUCT_TRUTH.md`
- `docs/prd-v0.5.md`
- `docs/interaction-spec-v1.1.md`
- `docs/ux-spec-v1.1.md`
- `docs/acceptance-spec-v1.1.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-mobile-queue-count-alignment-v1.md`
- `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-mobile-queue-count-alignment-delivery-v1.md`
- `docs/coordination/acceptance/2026-04-28-lyra-mira-s7a-mobile-overview-inbox-companion-acceptance.md`
- `docs/coordination/acceptance/2026-04-28-lyra-mira-s7a-monitor-truth-fix-acceptance.md`
- `ui/src/views/MobileCompanionView.tsx`
- `ui/src/stores/useDataStore.ts`
- `cd ui && pnpm build`

## Coverage Matrix

| # | Requirement | Prototype evidence | Result | Notes |
|---|---|---|---|---|
| 1 | `待审批 / 闸门` counts the full seed-backed mobile approval/gate set | `ui/src/views/MobileCompanionView.tsx:27`; `ui/src/views/MobileCompanionView.tsx:60`; `ui/src/stores/useDataStore.ts:1127`; `ui/src/stores/useDataStore.ts:1137`; `ui/src/stores/useDataStore.ts:1156`; `ui/src/stores/useDataStore.ts:1166` | PASS | The allow-list now includes `待处理交接`, `需要验收`, `需确认范围`, `验收待决策`, and `输入请求`. |
| 2 | The urgent mobile queue defaults to high-priority or action-needed rows only | `ui/src/views/MobileCompanionView.tsx:35`; `ui/src/views/MobileCompanionView.tsx:82`; `ui/src/stores/useDataStore.ts:1147`; `ui/src/stores/useDataStore.ts:1176`; `ui/src/stores/useDataStore.ts:1195` | PASS | `背景记录`, `已记录决策`, and `需要同步` are excluded by default because they are neither `Critical` nor members of the mobile action set. |
| 3 | Queue ranking is deterministic and bounded | `ui/src/views/MobileCompanionView.tsx:41`; `ui/src/views/MobileCompanionView.tsx:84` | PASS | The queue ranks `Critical` first, then non-critical action-needed rows, then newest seeded timestamp first inside the same band. |
| 4 | The accepted mobile shell boundaries are preserved | `ui/src/views/MobileCompanionView.tsx:104`; `ui/src/views/MobileCompanionView.tsx:197` | PASS | No layout redesign, route expansion, or theme-system regression was introduced by the fix. |
| 5 | Build stays green after the truth-alignment patch | `cd ui && pnpm build` | PASS | Lyra re-ran the production build locally on 2026-04-29 and confirmed success. |

## Findings

No blocking findings remain for the scoped `S7A` queue/count packet.

Residual note:

- mobile action-card work (`INT-18` / `INT-19` / `INT-20`) is still future scope and was not touched here.

## Gate Decision

- **`S7A` queue/count alignment:** **GO**
- **Overall `S7A` packet closure:** **CLOSED**
- **Eligibility for next mobile packet:** **GO** from a contract standpoint, subject to owner availability and packet issuance

## Evidence Paths

- Fix packet: `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-mobile-queue-count-alignment-v1.md`
- Delivery reviewed: `docs/coordination/tasks/mira/MIRA-2026-04-28-s7a-mobile-queue-count-alignment-delivery-v1.md`
- Mobile view: `ui/src/views/MobileCompanionView.tsx`
- Canonical Inbox seed: `ui/src/stores/useDataStore.ts`
