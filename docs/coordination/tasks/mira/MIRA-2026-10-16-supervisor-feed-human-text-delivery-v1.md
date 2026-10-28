# Delivery: 79 % of the Supervisor panel's rows rendered as a bare event-type name

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-16-supervisor-feed-human-text-delivery-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-16 |
| version | v1 |
| dispatch | disp-QBVu3LeruaTavYLIN5BCI |
| tags | supervisor-panel, event-translation, defect, delivery, mira |

---

## 0. The one-paragraph version

The Aegis Supervisor column showed real data rendered as
`• SeatDispatchIssued`, `• MemoryRecalled`, `• SeatDispatchDelivered` — one English
type name per row, no content. Measured on the 400 newest rows: **321 of them
(80 %)** fell through to `default: content = \`• ${etype}\``. The 16 event types the
database actually writes and the switch never learned are now translated into
Chinese carrying their meaningful fields, the translator is a pure tested module,
and the default branch itself now says what an unrecognised type carried — so a
bare type name can no longer appear.

## 1. Re-measured rather than taken on trust

The dispatch claimed 319/400 (79 %). I measured it myself over the same 400 newest
rows the column loads (`query_ledger`, limit 200 twice):

| | |
|---|---|
| rows | 400 |
| rows carrying `payload.content` (a real person's words) | 8 |
| rows the old switch rendered as a bare type name | **321 (80 %)** |

Composition: `MemoryRecalled` 106 · `MemoryCreated` 68 · `SeatDispatchIssued` 49 ·
`SeatDispatchDelivered` 33 · `SeatDispatchCompleted` 29 · `SeatDispatchCancelled` 11 ·
`AcceptanceRequested` 7 · `TopicOpened` 6 · `AcceptanceGranted` 6 ·
`AcceptanceCancelled` 3 · `MemoryForgotten` 1 · `SeatDispatchFailed` 1 ·
`SeatDispatchRetried` 1. The count matches the dispatch's to within two rows (the
ledger moved between its measurement and mine), and the composition is the same.

I also verified the two vocabulary claims independently:

- **The 16 to add are real.** `query_ledger` rejects an unknown type by printing the
  whole vocabulary; all 16 are in it.
- **The 9 dead branches are dead.** `SessionCompleted`, `SessionFailed`,
  `SessionInterrupted`, `HandoffDrafted`, `HandoffReceived`, `HandoffWorking`,
  `HandoffReturned`, `CheckpointCreated`, `PromptDetected` are **not** in the
  vocabulary — the tool rejects each by name, so those cases could never match.

## 2. What was built

| File | Change |
|---|---|
| `ui/src/app-v2/lib/eventText.ts` | **new**. The pure translator: 28 event types -> one Chinese line each, plus an honest `default` |
| `ui/src/app-v2/lib/eventText.test.ts` | **new**. 53 assertions over every vocabulary type, using the payload shapes the writers actually emit |
| `ui/src/app-v2/hooks/useSupervisorData.ts` | the local `eventToMessage` is replaced by the shared translator; a new `useEventTextContext` resolves the ids the events carry |

### The id problem, and how it was solved without guessing

The dispatch's requirement was "which work, to whom, what status — not a string of
ids". But the events do not carry those: a `SeatDispatchIssued` row's entire payload
is `{dispatch_id}`. And the DTO the UI receives (`CanonicalEventDto`) has **no
`refs`** — id, type, occurredAt, actorRef, payload, nothing else — so the object
refs that name the seat and the dispatch are not on the wire at all.

So `useEventTextContext` performs the reads the line needs, once per panel load:

- `api.listDispatches(undefined, 500)` -> dispatch id -> `{title, targetSeatId,
  status}`. That is what turns `• SeatDispatchIssued` into a line reading, in the
  product's own language, *new work dispatched: "<the dispatch's title>" -> <seat
  name>, status <status>* (the exact strings are pinned in `eventText.test.ts`,
  which is where a literal belongs).
- `useDataStore`'s seats -> backend seat id -> display name, so `seat:seat-mira-001`
  renders as `Mira` and `human:supervisor` as the word for *operator*.
- A second, bounded pass: `api.getTopicMemories(topicId)` for **only the topics the
  loaded events actually mention** (typically a handful), so a `MemoryRecalled` row
  can show the memory's title instead of its id. Fetching every topic's memories on
  every panel open would have been tens of calls for a handful of rows.

Every miss is stated, never filled in — the line says the dispatch's *title was not
resolved*, the memory's *title was not resolved*, *no reason recorded*, *no actor
recorded*. A failed `listDispatches` leaves the panel rendering with the titles
unresolved rather than blanking it: losing the feed on a transient error would be a
worse defect than the one being fixed.

### The one thing deliberately not done

`AcceptanceGranted`'s payload is `{branch, base, merge_commit}` — no dispatch id, and
no refs to recover it from. The line therefore says *you accepted; merged into
<branch> at <short commit>* and, when the branch is absent too, *this one does not
record which work it was*. It does **not** strip `acc-` off an acceptance id to
derive one: that id is only deterministic when the caller omitted it, so deriving
would be a guess in exactly the shape this project has already paid for.

## 3. The 9 dead branches: removed, and why

Removed. They are not in the vocabulary, so they can never match; and a switch that
appears to cover 22 types when 9 of them are unreachable is the same failure as the
one being fixed, in the opposite direction — it makes the reader believe the
translation is more complete than it is. `SessionStarted` (which *is* in the
vocabulary) is kept and translated.

## 4. The default branch is now a safety net

```
<TypeName> — this type has no human translation yet; it carried these fields: foo, bar
```
(the literal is pinned in `eventText.test.ts`)

The next type added to the vocabulary degrades to "here is what it contained" rather
than back to a bare word. That is what stops this defect recurring silently — the
old default was the whole bug.

## 5. The two things the acceptance card says must not regress

- **A real person's words pass through byte for byte.** `SupervisorMessage` and
  `SeatResponse` return `payload.content` unchanged; an empty one renders
  *this one is empty*. Pinned by tests that compare against the exact string.
- **The emoji prefix style is preserved** — the original comment explains it is
  deliberate (the V2 MessageBubble needs a populated `card` for non-text types, and
  the prefix carries the category without card chrome). Every translated line still
  starts with one; a test asserts no line starts with `•`.

## 6. Gates

| Gate | Result |
|---|---|
| `tsc --noEmit` (whole tree) | clean |
| `vitest` | **153/153** (53 new) |
| `scripts/lint-changed.sh track/infra-foundation` | eslint 0 errors, prettier clean |
| `vite build` | green |
| `cargo test --workspace` | 251 passed / 0 failed (no Rust changed) |
| `cargo clippy` | clean (no Rust changed) |
| `scripts/verify-docs.sh` | this note clean |

Two pre-existing `any` annotations in `useSupervisorData.ts` (`roleKey`, and the
`target_seat_id` read) surfaced because the file is now changed; the ratchet
requires changed files to be clean, so both were tightened rather than left.

## 7. What is not proven

The 53 tests exercise the translator directly with the writers' real payload shapes,
and the measurement is against the live ledger. What is not proven is the rendered
panel: nobody has opened Cmd+K and read the column. In particular the memory-title
second pass is exercised only through the pure function — that the bounded
`getTopicMemories` fan-out resolves in the running app, and that a topic with more
than 200 memories degrades honestly, is unverified. His click is the first look.

## 8. Boundaries

- Sorting rules untouched (the backend takes newest-then-reverses, with id as the
  tie-break in both directions).
- Contact list source untouched.
- No other view changed; the new module is imported by `useSupervisorData` only.
- No backend change — the DTO's missing `refs` was worked around with reads, not by
  widening the command surface.

## 9. Artifact index

- Branch: `packet/mira/knowledge-panel`.
- Ledger: see `emit_event(ArtifactCreated)`.
- Acceptance card: `acc-disp-QBVu3LeruaTavYLIN5BCI`.
