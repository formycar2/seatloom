# Delivery: the pending-acceptance count included voided cards — 7 was 5, because `cancelled` was not a settled state

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-16-acceptance-voided-count-delivery-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-16 |
| version | v1 |
| dispatch | disp-CEa11raWfi8W_UHiXGKBw |
| tags | acceptance, voided-card, count, defect, delivery, mira |

---

## 0. The one-paragraph version

`isDecided` knew two settled states out of four. `cancelled` was missing, so every
voided card stayed in the awaiting list **and** in the "N items awaiting your decision"
number. The
operator counted 7 where 5 were actually his to decide; the extra two had been
voided days earlier. `isDecided` now knows `cancelled`, voided cards moved to their
own collapsed **voided** area that carries the reason, and the number counts only
what is genuinely waiting.

## 1. Root cause, verified

The dispatch named `ui/src/lib/acceptance.ts:129-131`. Confirmed verbatim:

```ts
export function isDecided(status: string): boolean {
  return status === 'accepted' || status === 'rejected';
}
```

and the consumer at `AcceptanceSection.tsx`:

```ts
cards.filter((c) => !isDecided(c.status) && !cardState[c.id]?.dismissed)
```

The type's own comment (`types-dto.ts:223`) says
`'pending' | 'accepted' | 'rejected' | 'cancelled'` — so this was the front end
recognising three of four states, not the back end sending an extra one. Confirmed
independently: the two voided cards the operator was seeing really are `cancelled`,
and both carry a reason in `feedback`:

| card | status | reason (verbatim, from `get_acceptance_lineage`) |
|---|---|---|
| `acc-disp-me6Nm8e9D6ZRe4fKyVokY` | cancelled | base recorded as nonexistent track; content already bootstrap-merged by Aegis; voiding per ruling — the new feature gets a new card with lineage + structured plan |
| `acc-disp-supervisor-intent-ui-wiring` | cancelled | the original card's acceptance description was full of technical terms the operator could not follow, so it was re-opened in plain language as `-plain` |

**Re-measured the live set rather than trusting the dispatch's snapshot.** The
dispatch said 8 cards: 5 pending + 1 accepted + 2 cancelled. The ledger has moved
since (two of my own cards were requested and granted while this dispatch was being
worked). Derived from the ledger's acceptance events, cross-checked against
`get_acceptance_lineage` per id:

- 10 distinct cards ever requested, **6 accepted, 2 cancelled, 2 pending**.
- With the bug, the section would render **4 items awaiting decision instead of 2** —
  the defect is
  live right now, not historical.

## 2. The judgment call: hide them, or show them?

**Shown, collapsed, with the reason.** Three reasons, in order of weight:

1. **The operator counts.** His whole complaint was a number that did not add up.
   Hiding voided cards entirely changes the total again, and he has no way to
   reconcile it — the same class of confusion, one layer down.
2. **He has asked "where did that card go" before.** The dispatch says so
   explicitly. A voided card with its reason is the answer to that question, and the
   reason is genuinely interesting: one was withdrawn because its base branch did
   not exist, one because nobody could read its acceptance criteria.
3. **They are not work.** So they are collapsed by default and excluded from the
   count. Opening them is one click, and the header says in plain words that they
   are not counted.

No extra read was needed: `feedback` carries the reason on both currently-voided
cards, so the area renders from the same list the section already polls.

## 3. The second instance of the same bug, found and fixed

The dispatch asked whether anywhere else only recognises accepted/rejected. One
more place, in the same file:

```ts
// DecisionResult
const accepted = card.status === 'accepted' || Boolean(card.mergeCommit);
const rejected = card.status === 'rejected' || Boolean(card.reworkDispatchId);
```

Four states, two branches: a cancelled card matched neither and would have rendered
an empty result strip. It is now an explicit third branch (badge + the reason).
Unreachable through the awaiting list after the main fix, but it is the same shape
and it is now impossible rather than merely unused.

Checked and **not** the same bug: `SupervisorIntent.tsx:177` and
`HandoffDetail.tsx:28` gate on **handoff** statuses, a different domain.

## 4. One thing deliberately not worked around

The voided area shows the card's **creation time**, not when it was voided.
`AcceptanceCard` — the DTO the pending-list read returns — deliberately drops
`decided_by` / `decided_at` (`src-tauri/src/dto.rs` destructures them as `_`), and
`lib/types-dto.ts` is Nimbus's fence. Rather than edit the wire shape for a
timestamp on a collapsed record, the label says what the time actually is. The void
*instant* is in the ledger's `AcceptanceCancelled` event, which the Supervisor feed
now renders (the previous dispatch's fix) — and the area says so when a card carries
no reason either.

## 5. Gates

| Gate | Result |
|---|---|
| `tsc --noEmit` (whole tree) | clean |
| `vitest` | **159/159** (6 new) |
| `scripts/lint-changed.sh track/infra-foundation` | eslint 0 errors, prettier clean |
| `vite build` | green |
| `cargo test --workspace` | 251 passed / 0 failed (no Rust changed) |
| `scripts/verify-docs.sh` | this note clean |

New tests: the count excludes a voided card; the voided area is collapsed by default
and opens with title + reason; a card with no reason points at the ledger;
already-accepted and already-rejected cards stay out of the count exactly as before;
and `isDecided` / `isCancelled` / the voided badge are pinned in
`acceptance.test.ts`.

## 6. What is not proven

Nobody has looked at the rendered section with the real 10-card set. The tests
drive the component with a mocked list, and the live card states were measured
through the ledger and the lineage read — but "the number reads 2" in the running
app is unverified. His click is the first look.

## 7. Boundaries

No backend change. No other view touched (`isDecided` / `isCancelled` are new
exports in the file the dispatch named, consumed only by `AcceptanceSection`). No
skeleton rework: the voided area is a new collapsed block below the existing list,
built from the section's own primitives.

## 8. Artifact index

- Branch: `packet/mira/knowledge-panel`.
- Ledger: see `emit_event(ArtifactCreated)`.
- Acceptance card: `acc-disp-CEa11raWfi8W_UHiXGKBw`.
