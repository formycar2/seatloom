| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-11-09-2100-i-turned-the-measured-gap-into-a-dispatch-and-the-backend-needed-nothing |
| status | delivered |
| author | aegis |
| date | 2026-11-09 |
| version | v1 |
| tags | aegis, ui-gap, dispatch, scope-boundary, staleness |

# I turned the measured gap into a dispatch, and the backend needed nothing

## Last hour's measurement became this hour's work

I spent last hour measuring the interface gap: the operator's obligations live in three tables, the
interface reads two, and the third — **dispatches ready and over 30 minutes** — has no representation at
all. I closed that note by saying the gap was "now measured and dispatchable, and I have not started it".

Starting it would have meant writing UI in a heartbeat, which is not the right way to change the surface
the operator accepts on. So this hour I dispatched it: `disp-bouEIM-49hK0xR_-B8EMm`, priority 95, to
Mira.

## Why it is bounded, and that came out of the measurement

Before dispatching I checked whether the backend could carry it, because a dispatch that quietly needs a
schema change is not a bounded one.

**It can, and already does.** `cmd_list_dispatches` exists and takes `project_id`. `DispatchDto` at
`src-tauri/src/dto.rs:324` already carries `issued_at` as a String. So the age is already on the wire and
the staleness decision is a UI-side computation against the clock — exactly how `work-list.sh` computes
it.

That is the difference between a bounded deliverable and an open-ended one, and the only reason I could
tell is that I measured first. Had I dispatched from the phrase "the CLI tools are not integrated", the
ask would have implied a backend endpoint, a view, or a way for the app to run a shell script.

## What the dispatch asks for, and the part that makes it trustworthy

Three required directions, not one:

| case | required |
|---|---|
| a dispatch that became ready 5 minutes ago | must **not** appear |
| a dispatch that became ready 31 minutes ago | must appear |
| a dispatch that is `in_flight` or `done` | must never appear, at any age |

And the empty case has to say something true rather than nothing. **A section that only ever shows the
populated state cannot distinguish a broken query from a quiet system** — the same failure mode as a check
whose only result is green.

The rule itself is not mine to invent: `work-list.sh:232` implements it, and `notify-operator.sh:166-168`
already had it right in the notification channel with the honest framing that a stalled queue is a
**system** problem rather than the operator's task. The dispatch says to copy that framing rather than
improve on it.

## The scope boundary I deliberately did not cross

There is a second half of the gap and I left it out on purpose: neither `work-list.sh` nor
`prepare-verification-build.sh` has any way in from the interface. Whether the app should be able to
invoke a shell script at all is a **product decision**, not an implementation detail — and it is the
kind of decision I have been wrong about before by answering it in a dispatch. The dispatch names it as
out of scope in those words, so Mira is not pulled past it and nobody later mistakes the silence for an
oversight.

It also forbids a mock, with the reason attached: this codebase already recorded what happens when a
failed hydration renders three seed projects as real. A mock standing in for a dispatch row is that
failure again.

## Measured state

- Ledger: 35 tables. Six seats online. All six protocol checks exit 0.
- Two pending cards, both clickable, both merge clean into current track.
- One ready dispatch for Mira, which is the first piece of forward motion on the interface gap.
- Stale ready dispatches right now: **0** — so the new section would render its empty state, which is
  exactly the case the dispatch requires be distinguishable from broken.
- Salvage: 2 live patches round-tripping, 7 superseded. Nothing at risk.
- Push: local and remote both at `8eaba20`, confirmed. No new commits this hour.
- Sandbox: 95-byte ASCII text file, no backup. Still needs your authorisation.
- Main application: pid 90805, untouched.

## Needs you

- **Two cards are waiting**, and they are different work.
- **The sandbox decision unblocks five dispatches at once**, not one. That is the cheapest lever you
  have.
- **`documents` has zero rows**, so `l2_ref_kind=document` can never succeed. The failure is legible now,
  which is the precondition for answering it, not the answer.
- **Whether the app should be able to invoke a shell script at all** — the second half of the interface
  gap, deliberately not answered by me.
