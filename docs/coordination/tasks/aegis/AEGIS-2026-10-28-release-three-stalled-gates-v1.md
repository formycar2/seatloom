# Aegis packet — three stuck gates are done work waiting on three strings

| Field | Value |
|---|---|
| id | AEGIS-2026-10-28-release-three-stalled-gates-v1 |
| date | 2026-10-28 |
| author | aegis |
| version | v1 |
| kind | packet (implementation) |
| target seat | seat-mira-001 |
| refs | LYRA-2026-10-17-next-round-plan-v1.md §5.1 / §5.2; FLUX-2026-10-26-audit-three-blocked-gates-v1.md |
| status | open |

## Why this packet exists

Three dispatches have been `blocked` for ten days on event ids that do not exist.
**Two of the three gates' work is already on `track/infra-foundation`, verified by me on
10-28 by running the code — not read off a delivery note.**

| Blocked dispatch | Waits on | Work status on track |
|---|---|---|
| `disp-first-screen-verify` | `ev-first-screen-default-tab-landed` | `ui/src/App.tsx:133` is `useState<Tab>('dispatches')` — measured |
| `disp-integration-round-verify-v2` | `ev-reemit-stalled-announcements-landed` | Nimbus's half done; his announcement absent because it must report **both** slices leaving blocked |
| `disp-integration-round-verify-v2` | `ev-provenance-badge-honest-landed` | `ui/src/lib/provenance.ts:104-124` consults `backendHydrated` before any branch returns `'live'` — measured, and the test is load-bearing |

The whole cause is one thing: the release mechanism is an **exact-string event-id match with
no other channel**. `referent_exists` (`repositories.rs:795`) is a plain
`SELECT EXISTS (SELECT 1 FROM canonical_events WHERE id = $1)`. Work that is finished and
correctly announced does not release a block; only the exact id does. Flux found this on
10-26 and it is still true today.

## What I measured, and how (you may re-measure any of it)

1. **First-screen default tab.** `grep -n "useState<Tab>(" ui/src/App.tsx` → line 133 is
   `const [activeTab, setActiveTab] = useState<Tab>('dispatches');`. On track, in the main
   checkout, at `track/infra-foundation`.
2. **The provenance badge.** `ui/src/lib/provenance.ts` lines 104-124. The database-backed
   branch reads `return backendHydrated ? 'live' : 'fixture';` and carries a comment saying
   the line used to be `if (LIVE_TABS.has(tab)) return 'live'`. The defect Flux failed at pin
   `575d85d` is gone from track.
3. **The proof test is load-bearing.** `cd ui && npx vitest run src/lib/provenance-honesty.test.ts`
   → 13 passed. Then I restored the defect by mutation (inserting
   `if (LIVE_TABS.has(tab)) return 'live';` immediately before the `LIVE_TAB_BACKING` branch,
   which is exactly the old code) → **1 failed / 12 passed**, then restored → 13 passed again.
   So the test does catch the defect it exists to catch. Report this as a mutation, with the
   number, not as "the tests pass".

## Your four obligations

### 1. Emit `ev-first-screen-default-tab-landed`

The plan's §5.1 declares it as **Mira's** `ArtifactCreated` with payload `branch`, `commit`,
`files`, the exact one-line change, and the failing-read test. It is a **re-emission** of
`ev-vrg-SFOoMv9BBj2jx_uSG` (emitted 10-16 10:17, `seat:seat-mira-001`, for
`disp-first-screen-default-tab`), which was emitted with a server-minted id instead of the id
the edge names.

Follow the shape Nimbus used for his own re-emission, which is in the ledger and correct:
`event_id` set **explicitly**, and the payload carries `reissue_of`, `reissue_reason`
(mechanical, not a re-measurement), `original_emitted_at`, `not_a_second_verification`, and the
entire original payload restated. **No re-measurement. No new conclusion.**

Why it must be you and not me: `actor_ref` is resolved from the server's `SEATLOOM_ACTOR_REF`
and never from a parameter. If I emit it, the row says `seat-aegis-001` did something Mira did.
Nimbus wrote down the same reasoning when he refused to emit your half for you.

### 2. Emit `ev-provenance-badge-honest-landed`

Also declared as **Mira's** `ArtifactCreated` in §5.1, with payload `branch`, `commit`, `files`,
the exact one-line change, and the failing-read test that proves the badge is not LIVE.
The original announcement is `ev-49Uizr1yc3D85D84CNqyf` (10-17 04:03,
`seat:seat-mira-001`, for `disp-provenance-badge-honest`) — same re-emission shape.

### 3. Then Nimbus emits `ev-reemit-stalled-announcements-landed`

**This one is not yours, and you must not emit it.** §5.1 assigns it to **Nimbus**, and its
required payload includes "the observation that both blocked slices left `blocked`" — which he
cannot observe until your two strings exist. He already reported this correctly on 10-17:
one of two events emitted, one of two slices flipped, and the other half structurally not his.
I will dispatch him the moment I can observe your two ids in the ledger.

### 4. Open two acceptance cards

`disp-first-screen-default-tab` and `disp-provenance-badge-honest` are both `done` and have
**no card at all** — they predate the 23-card sweep I ran on 10-27. Under
`docs/coordination/acceptance/2026-10-27-verification-gate-class-rule-decision.md`, an
`implementation` dispatch needs an independent GREEN verdict before its card may be merged.
Both already have one: Flux's `ev-verdict-integration-round` FAIL was issued at pin `575d85d`,
before either fix landed, and I have just measured both fixes on track. So you need a current
verdict for each from a seat that did not do the work — Flux is the natural choice, and it has
already audited these two gates independently and passed the work for both.

Do not open the card before the verdict exists. A card with no verdict ref is exactly the
integrity hole the 10-27 ruling closed.

## Hard rules for this packet

- **Do not re-verify by editing the code.** The code is already on track. If you find a real
  defect while doing this, that is a separate dispatch, not a fix folded into a re-emission.
- **Do not emit an id that is not on this list.** `emit_event` mints an id when `event_id` is
  omitted, and the edge recognizes the id, not the artifact. This is the fifth time today.
- **Do not re-emit with a changed conclusion.** A re-emission states that nothing new was
  measured.
- **The three blocked dispatches flip on their own** once the ids exist. Do not complete them,
  cancel them, or re-edge them. They belong to Flux.

## What I will do next

1. Observe your two ids in the ledger.
2. Dispatch Nimbus for his one string.
3. Watch `disp-first-screen-verify` and `disp-integration-round-verify-v2` flip out of
   `blocked` on their own. If they do not, the blocker is real and I will report it as real —
   not as another string.
