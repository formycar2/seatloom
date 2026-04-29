# FLUX-2026-04-28-UI-Contract-Realignment-Review-v1

| Field | Value |
|---|---|
| Owner | Flux |
| Acting role | Temporary UI/UX alignment seat replacing Mira |
| Issued by | Lyra |
| Status | Active |
| Deadline | 2026-04-29 10:30 |
| Acceptance owner | Lyra |
| Stage | Product Baseline Freeze / pre-SG-01 |
| Execution mode | Need-to-know / token-efficient |

## 1. Micro-brief

1. Do **not** run SG-01 verification on the current prototype yet.
2. The active v0.5 contract set has moved ahead of the current UI; the next needed output is a **UI contract realignment review**, not a baseline verdict.
3. Work as acting Mira for contract-to-surface alignment only. Do not redefine product meaning.
4. Preserve already accepted baselines: typed Artifact objectization, Chinese high-density demo data, drift-signal noise policy, and the readability floor on touched surfaces.
5. Publish the review artifact in English; keep terminal/tmux progress summaries in Chinese.

## 2. Decisions already frozen

- `docs/PRODUCT_TRUTH.md` is the only entrypoint.
- Active authority set:
  - `docs/prd-v0.5.md`
  - `docs/interaction-spec-v1.1.md`
  - `docs/ux-spec-v1.1.md`
  - `docs/acceptance-spec-v1.1.md`
- No SG-01 re-verification should happen until a post-v0.5 UI realignment artifact exists.
- Do not use archived v0.4/v1.0 product docs as primary authority for this packet.
- If a required UI behavior appears to conflict with the active contract, escalate the clause conflict to Lyra instead of inventing a compromise in code.

## 3. Contract pack

Mandatory reads, in this order:

1. `docs/PRODUCT_TRUTH.md`
2. `docs/prd-v0.5.md`
3. `docs/interaction-spec-v1.1.md`
4. `docs/ux-spec-v1.1.md`
5. `docs/acceptance-spec-v1.1.md`
6. `docs/coordination/AI_NATIVE_WORKFLOW_PRINCIPLES.md`
7. `docs/coordination/COORDINATION_RULES.md`
8. `docs/coordination/reviews/2026-04-28-lyra-review-response.md`
9. `docs/coordination/reviews/2026-04-28-lyra-pre-implementation-freeze-decision.md`
10. `docs/coordination/acceptance/2026-04-28-lyra-artifact-objectization-baseline-acceptance.md`

## 4. Need-to-know scope

Read only:

- this packet
- the contract pack above
- the exact UI files needed to judge current coverage

Do not consume by default:

- archived product docs as primary authority
- unrelated architecture delivery threads
- old SG-01 verifier packets
- broad repo history

If blocked, return one compact blocker note with the exact clause and file path.

## 5. Required work

| Priority | Requirement | Output expectation |
|---|---|---|
| P0 | Publish a UI contract realignment matrix | `required flow/surface -> current status -> exact gap -> target files -> blocking/non-blocking` |
| P0 | Cover the current high-risk slices first | shell/navigation, project switching and memory, Inbox/detail/action loop, Timeline replay/filter/detail, WorkItem/Handoff lifecycle, default Detail states |
| P0 | Include new v0.5 interaction additions that the old UI baseline could not have covered | review-change routing (`L1/L2/L3` + `change_tier_record`), prompt-blocked/session assist surface, typed Artifact detail/filter presence, capability-truth visibility at execution points |
| P0 | Separate already-accepted UI baselines from unaligned areas | explicitly mark what must be preserved vs what must be redesigned |
| P1 | Recommend the minimum implementation sequence | ordered packets/phases with dependency notes and acceptance boundary suggestions |
| P1 | Flag any contract clause that is still too ambiguous to implement safely | cite the exact doc/section and propose the smallest decision needed from Lyra |

## 6. Non-goals

- Do not publish a PASS/FAIL SG-01 verdict.
- Do not produce a verifier evidence pack.
- Do not patch product code in this packet unless Lyra explicitly reissues this work as an implementation packet.
- Do not redesign product IA beyond what the active contract requires.

## 7. Done definition

All items below must be true:

- A delivery artifact is published at the required path.
- The delivery artifact uses the active v0.5 contract set only.
- The matrix clearly distinguishes `already aligned`, `needs repair`, and `needs Lyra decision`.
- File targets are concrete enough for a follow-up implementation packet.
- Accepted baselines that must not regress are called out explicitly.

## 8. Required delivery artifact

Publish completion as:

- `docs/coordination/tasks/flux/FLUX-2026-04-28-ui-contract-realignment-review-delivery-v1.md`

Required sections:

1. Micro-brief
2. Coverage / realignment matrix
3. Preserved baselines
4. Required implementation packets
5. Blockers / decision requests
6. Suggested verification boundary

## 9. Reporting rules

- Durable artifact: English only.
- Terminal/tmux progress summary: Chinese only.
- Keep outputs delta-first and short.
- Use exact file paths and contract references.
