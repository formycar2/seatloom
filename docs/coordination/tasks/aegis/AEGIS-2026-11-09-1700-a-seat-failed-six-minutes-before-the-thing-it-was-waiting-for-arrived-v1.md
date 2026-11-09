| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-09-1700-a-seat-failed-six-minutes-before-the-thing-it-was-waiting-for-arrived |
| status | delivered |
| author | aegis |
| date | 2026-11-09 |
| version | v1 |
| tags | aegis, failed-dispatch, verdict-timing, own-mechanism-error |

# A seat failed six minutes before the thing it was waiting for arrived, and it was right to fail

## The sequence, to the minute

| time (UTC) | event |
|---|---|
| 08:10 | I dispatch Nimbus: your fix is real, open the card |
| 08:15:42 | Nimbus opens a topic and records a memory |
| 08:15:58 | Flux emits `ev-nRH3MeASZN_sK3E6Up1Ku` — **verdict `pass`, commit `41da35b`** |
| 08:16:19 | Nimbus sends its response: the card is NOT open, blocked on a verdict that does not exist yet |
| 08:16:41 | Nimbus marks the dispatch **failed** |

Nimbus failed **43 seconds** after the verdict it was waiting for landed. It had already done the work:
synced the branch to `ea511ae` (15 ahead, 0 behind), verified `memory.rs` and the test script are
byte-identical to `41da35b` after the merge, the memory suite 5/5, `cargo test --workspace` 387 passed
/ 0 failed, the sandbox suite 63/0.

**And it was right to fail.** Its words: *"the card is NOT open. It is blocked on a verdict that does not
exist yet, and I am reporting that rather than opening a card against an empty criterion."*

That is the correct call. `request_acceptance` refuses a `flux_verdict_ref` that names no existing
event — so the only way to "succeed" at that moment was to invent a verdict id, or to open a card
against nothing. A seat that fails a dispatch rather than fabricate the precondition has the right
priorities, and it is worth saying so in the same breath as noting it missed by 43 seconds.

## Flux's verdict, and the correction it made to me

The verdict that landed carries six criteria, all met: the caller-visible message names both production
constraints through the real tool with table and SQLSTATE; the refused write still refuses with rows=0;
a valid write still succeeds with rows=1; the assertion goes through `let rejection: MemoryRejection =
err.into()`, the same `From<DbError>` that `?` uses; line 416 reads `db_chain(&other)` while line 321 is
unchanged with its comment explaining why; and **`mutation_of_416_is_red`** — reverting the wiring makes
the suite fail with the defect's own message.

**And it corrected my mechanism.** I had told Nimbus that mutating line 321 leaves the suite green. Flux
measured it: mutating 321 to `db_chain(&other)` **does not compile** — `From<DispatchRejection>` receives
a `DispatchRejection`, not a `DbError`, so `db_chain` does not type-check, and the suite exits 1 with
E0308 while its own gate reports that not a single test ran. Flux redid it type-legally with
`format!("{}", other)`, which is semantically identical, and the suite stayed green.

I verified the correction myself: line 321 sits inside `impl From<DispatchRejection>` (starts line 297)
and line 416 inside `impl From<DbError>` (starts line 370), and `db_chain` takes `&DbError`. **So my
outcome was right and my reason was wrong.** Same conclusion, wrong mechanism — which matters, because
anyone repeating the experiment on my description would hit a compile error and conclude something
different.

## What this says about the dispatch protocol

A dispatch that can only be satisfied by a verdict that arrives after the dispatch is a dispatch that
invites fabrication. The seat handled it correctly by failing, but the shape is worth naming: **the
window between "work is done" and "verdict exists" is a window where the only successful action is a
lie.** Nimbus's own report says it warned Flux about the two-arm trap in the verification dispatch, so
the information flow was working; the timing was not.

I have re-dispatched with the verdict id in hand, at higher priority, and the branch already satisfies
containment so nothing further needs syncing.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- Two failed dispatches, both Nimbus, both correctly reported rather than faked. `disp-OupuAHH6tLwlzqXnswXV8`
  is the older one, already recorded.
- Nimbus's branch `packet/nimbus/candidate-final` at `ea511ae`, 15 ahead, 0 behind, worktree clean.
- Salvage: 2 live patches round-tripping, 7 superseded. Nothing at risk.
- Push: local and remote both at `565d0bd`, confirmed.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **One card is waiting on `packet/vela/washing-pipeline`**, and a second is being opened now for
  Nimbus's `db error` fix.
- **`l2_ref_kind=document` while `documents` is empty.** Flux named it as uncertified and escalated it.
  The failure is now legible — the caller is told which constraint — which is the precondition for
  answering it, not the answer.
- **The sandbox rebuild**, still the only thing between seven verified deliverables and you.
