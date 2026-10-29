# RULING — cold start lands on Dispatch, always. And the red tests must not stay red on track.

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-first-screen-ruling-v2 |
| status | active |
| author | lyra |
| date | 2026-10-29 |
| version | v2 |
| tags | ruling, product-behaviour, first-screen, red-tests, supersedes-v1-in-part |

## The ruling

**A. A cold start lands on the Dispatch page, always — regardless of which project was last open or
which page it was on. Per-project tab memory continues to work *inside* a session.**

This is the ruling I gave in `LYRA-2026-10-29-first-screen-behaviour-ruling-v1.md`. This dispatch
brought me two measurements I did not have, and **both confirm A rather than changing it.** I am
restating it with the new evidence rather than issuing a second, different ruling.

## Why A, from the operator's own words rather than from taste

**1. The first screen is not my design. It is his, written down on 2026-10-16.**

`ui/src/App.tsx:142` carries his words verbatim:

> [operator, 2026-10-16] The first screen is: what the whole fleet is doing, what is stuck, and
> what needs my decision.

And it records why the Dispatch page is the answer: it is the only surface in the product read from real
tables, and it already answers that question directly.

**2. My contract §6.2 slice 1 said the change was one line.**

> **Slice 1 (Mira) — one line.** `App.tsx:92` becomes `'dispatches'`. Nothing else changes: no re-layout,
> no new navigation, no new component, the mirror keeps its tab.

Mira did exactly that, correctly. What defeated it is an effect that already existed and that the
one-line framing did not account for. **So A is not a new interaction design — it is the contracted
design, finally working.** The restore-overriding behaviour was never designed; it is what happened when
a persisted store met a default.

**3. His second instruction cuts the same way.** improve on the existing SeatLoom design, do not do new interaction design. **B is the new behaviour**: it
would replace a 13-day-old written contract with a different one, because an accident defeated the
original. A is the existing design, completed.

**4. Dispatch is the only page that answers his question.** The Mirror page is tmux panes — the least
answer-shaped surface in the product. A cold start that lands there makes him hunt for the thing he
opened the app to see.

## What this does not do

- **It does not delete the restore.** Per-project tab memory is a feature he asked for and it stays.
  Switching from project A to project B inside a session still puts you on B's remembered page. That is
  the second half of the ruling and it is not optional — it is the thing that makes A safe.
- **It does not re-layout anything, add a component, or add a route.** Mira's `do_not` list stands.

## Flux's new measurement, and what it changes

Flux measured that **today**, on this machine, the active project is `seatloom` and its saved tab is
`dispatches` — so the app already opens on Dispatch. **The defect is latent, not visible.** I verified it
myself: the persisted store now reads `activeProjectId = seatloom`, saved tab `dispatches`.

That is worth stating precisely, because it cuts both ways and neither direction is "do nothing":

- **It lowers the risk of A.** On the current state, A changes nothing the operator sees. It is a guard
  against a state he has not hit yet.
- **It does not lower the case for A.** The state that breaks it is one click away — leave any project on
  the Mirror page, and every cold start after that lands on Mirror. A latent defect that silently
  reverses a written contract is still a defect; it is just one that has not bitten yet.

And it explains something I should be honest about: **my earlier measurement said `project_delivery/QL`
on `live-seats`, and that was true when I took it.** The state changed at 11:30. My finding was correct
about the mechanism and correct about the risk; it was a snapshot of a moving value, not a permanent
property. That is the same lesson as the stale dispatch failure reasons — **a measurement is a
measurement of a moment.**

## The red tests, which is the part that must not be left as it is

**Two tests are red on `track/infra-foundation` right now.** I ran them myself:

```
ui/src/App.restartRestore.test.tsx
  x a COLD START lands on the Dispatch page even when the recorded project has a saved tab
  × switching projects inside a session still restores THAT project's tab
  Tests  2 failed | 8 passed (10)
```

Those two are exactly the two halves of this ruling. They are red because A is not implemented. **A
permanently red test is worse than a missing one** — it is a check that has already failed once and been
ignored, which teaches everyone to stop reading it. It merged on a pass-with-named-defect verdict, which
is a legitimate outcome for a delivery, but it cannot be the resting state of the integration line.

**So the ruling has two obligations, and the second is not optional:**

1. **Implement A**, with the structure split Mira identified — not another condition on the same effect.
   The distinguishing signal already exists in state: `loadedUIProjectId` is `useState<string | null>(null)`
   at `App.tsx:138`, so `null` means first load and a real id means a project switch.
2. **Make those two tests green, or remove them.** Nothing else is acceptable. A red test on track is the
   exact failure this project has paid for repeatedly — a control that has stopped controlling.

## What this means for the card the operator already accepted

`acc-disp-first-screen-default-tab` was accepted at 05:12:37 on an assertion that was false when he read
it. **With A implemented, that assertion becomes true.** So the card was not wrong — it was **early**. It
named the intended behaviour and asked him to verify it before the behaviour existed.

That distinction matters, and I want it on the record rather than folded into "false premise": the card
was premature, not incorrect. The work it verifies is real, the behaviour it names is now ruled, and the
obligation is to make the assertion true rather than to argue about whether it was.

## What I did not do

I did not implement anything. I did not touch `App.tsx`, the test file, or the store. I did not cancel or
re-open the accepted card — decided rows are immutable. I did not dispatch a seat; Mira is already on the
rework and Aegis owns the integration line.
