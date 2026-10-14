---
template: T5
subtype: delivery
doc_id: 2026-10-14-lyra-disp-history-sessions-in-ui-contract-delivery
title: Delivery — the historical-sessions-in-UI contract, its four slices, and the data-model ruling Aegis owes
status: active
author: lyra
doc_date: 2026-10-14
version: v1
depends_on: AEGIS-2026-10-14-history-sessions-in-ui-contract.md (dispatch payload), LYRA-2026-10-14-history-sessions-in-ui-contract-v1.md
tags: delivery, lyra, history-sessions, archive, projects, decomposition
---

# Delivery — `disp-history-sessions-in-ui-contract`

## What was delivered

The three-layer product contract at
`docs/coordination/tasks/lyra/LYRA-2026-10-14-history-sessions-in-ui-contract-v1.md`
(`10504bc` on `packet/lyra/milestone-summary`), plus four dispatched slices.

| PRD hard rule | Where satisfied |
|---|---|
| User story | §2 |
| Interaction flow | §3, with §3.2 as the empty/missing-state table |
| Screen / UI surface definition | §4 |
| Pain / omission cost / behavior change | §5 |
| Out of scope with reasons | §7 |
| Acceptance criteria in the operator's language | §6 |

The packet path named in the dispatch payload
(`docs/coordination/tasks/aegis/AEGIS-2026-10-14-history-sessions-in-ui-contract.md`) does not
exist in any worktree or on any branch. I worked from the payload, which was complete, and
recorded that in the contract.

## Three measurements that changed the contract

**1. cursor is 14 sessions, not 13.** `CURSOR-HOMES.csv` has 13 *directories*; one directory
(`private_knowledge_base__fbb8b86a…`, home `…/private_knowledge_base/step-product/mind/code/brainpp-main`)
holds **two** sessions, labelled `(1/2)` and `(2/2)` in the CSV. The brief's own parenthetical
says exactly this and then states 13 anyway. The arithmetic is now written into the contract so
it is checkable at a glance: **150 + 18 + 14 + 1 = 183**. The operator has already corrected a
count in this dataset once, so this is the second time a count would have reached him wrong.

**2. All 14 cursor sessions have no timestamps at all.** Cursor's `chat_dump.json` carries
none, so a "when" column cannot be filled for them from the source. The contract makes
`时间未记录` an explicit UI state, forbids inventing a date, and forbids sorting the undated
sessions as if they were the oldest — which is what a naive date sort would do.

**3. The 38 home paths are not a clean tree.** The bare `/Users/jyxc-dz-0100609/Documents/GitHub`
holds 8 sessions and is a path-prefix ancestor of 37 of the others; `OfficeBench`, `ai-insight`,
`private_knowledge_base` and `sf-code` are ancestors too. So there is no safe
nearest-ancestor rule, and those 8 sessions get an explicit unclassified group with the reason
stated — never a guessed parent, never dropped.

## The data-model ruling Aegis owes

The brief proposed 38 `projects` rows plus 183 `sessions` rows pointing at one archive seat. I
recommended against it, on schema evidence rather than preference:

- **`archived_sessions`** (migration `011`) was designed for exactly this corpus —
  harness-agnostic, `PRIMARY KEY (harness, id)`, free-text project slug, `cwd`, `session_date`,
  `bytes`, `lines`, `sha256`, `raw_path`, `src_path` — and its own header states **"NO FK to
  seats (archived sessions have no seat) and NO FK to projects"** plus **"Never store
  transcript bodies (may contain secrets)"**. It currently holds zero rows: a designed, empty
  home.
- **`v_archived_sessions_normalized`** (migration `015`) already maps slug → project tree with
  R4 *unknown → unmapped, reported, never guessed* and R5 *`(root)` → unmapped + escalated to
  Aegis*. The normalization already exists and already refuses to guess.
- **`sessions`** requires `seat_id NOT NULL REFERENCES seats(id)` and (post-008)
  `project_id NOT NULL`, and `seats` is referenced by **six** tables —
  `project_role_bindings`, `seat_delegations`, `sessions`, `workitems`, `handoff_receipts`,
  `dispatches`. A synthetic archive seat would therefore appear in the Seat Registry and in
  dispatch targeting.

Cost of the recommendation: one new read command. Cost of the brief's model: a fiction inside a
load-bearing table. It is architecture-adjacent, so it is Aegis's call — but the two models
produce different migrations, so the ruling has to land before the data slice starts rather
than halfway through it. Escalated as a `SupervisorMessage`.

## The decomposition

| Slice | Seat | Dispatch | Edges |
|---|---|---|---|
| 0. Contract | Lyra | `disp-history-sessions-in-ui-contract` | — → emitted `ev-history-sessions-in-ui-contract-landed` |
| 1. Data | Onyx | `disp-history-sessions-data` | contract event → **ready** |
| 2. Read path | Nimbus | `disp-history-sessions-read` | `ev-history-sessions-data-landed` → blocked |
| 3. UI | Mira | `disp-history-sessions-ui` | `ev-history-sessions-read-live` → blocked |
| 4. Verification | Flux | `disp-history-sessions-verify` | both announcements → blocked, verdict id `ev-verdict-history-sessions` |

Edge reasoning, per the standing rule that a knowable-unmet precondition is an edge and never
prose: slice 2 waits for the data because a read path verified against an empty table proves
nothing; slice 3 waits on the **measured** `-live` gate rather than the announcement, because it
is the one that calls the capability; slice 4 waits on both announcements, because a verifier
needs the code and the rows to exist and nothing more. All five expected event ids and their
required payload shapes are written into the contract's §8.1, so no seat has to remember what to
emit.

Every slice carries the hard prohibitions in its payload: zero rows into `sessions`, zero new
seats, no transcript bodies in the database, no invented dates, no guessed parents, and no
merging of the 49 `OfficeBench` short sessions (which are one benchmark ops line sliced into
many command sessions, and are correct as they are).

## Acceptance criteria, in the operator's language

Contract §6 follows the plain-language rule from `disp-acceptance-cards-plain-language-v2`:
every sentence in Chinese, no English jargon, no self-invented abbreviations, no dispatch
titles, and each step says what to do → what to see → what passes → what gets rejected. The
exact arithmetic is written in — 38 projects, 183 sessions, 36,680 messages, `OfficeBench` 49,
`seatloom` 14, cursor 14 all undated — so he can check it with his own eyes. Two of the eight
steps are deliberately the ones that catch guessing: the cursor timestamps must read
"时间未记录" rather than a synthesized date, and the repository-root sessions must appear as an
explicit unclassified group rather than folded into a neighbour.

## Blockers

- **The data-model ruling above** is the only thing waiting on a human, and it gates slice 1's
  migration shape. Nothing else is blocked on a decision.
- Slices 2–4 are correctly blocked on referents that do not exist yet.

## Artifact paths

- `docs/coordination/tasks/lyra/LYRA-2026-10-14-history-sessions-in-ui-contract-v1.md`
- `docs/coordination/tasks/lyra/2026-10-14-lyra-disp-history-sessions-in-ui-contract-delivery.md` (this file)

## Ledger

- `ev-history-sessions-in-ui-contract-landed` — `ArtifactCreated`, the contract announcement,
  carrying the three corrections and the data-model recommendation.
- `ev-AFDhmwIfNNAZkdXoyOye8`, `ev-_vLcSPwmAFyjbvi_3mnz2`, `ev-PoUX_ROtK4ee2wfwnwWn_`,
  `ev-J0Kd_m8y1perofk4HQ6BF` — `ArtifactCreated` for the four slices, each recording why its
  edges are what they are.
- `ev-pd2gB6fRYe1xqVr1q3dN4` — `SupervisorMessage` to Aegis with the measurements and the
  data-model ruling request.
