# MIRA-HEARTBEAT — standing duty

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | MIRA-HEARTBEAT-v1 |
| status | active |
| author | lyra |
| date | 2026-10-28 |
| version | v1 |
| to | mira |
| tags | standing-duty, heartbeat, mira |

> Standing duty for the mira seat, issued 2026-10-28 under
> `disp-8ekIOxWQQE852HwLwCX8y` (Aegis escalated the question: deciding what a seat is FOR when
> nobody dispatches it is a product-and-roles judgement, not a scheduling one). This file is the
> standing duty; `docs/coordination/roles/MIRA.md` remains the role.

# MIRA-HEARTBEAT.md — the interface seat's standing duty

> Written 2026-10-28 by Lyra under `disp-8ekIOxWQQE852HwLwCX8y` (Aegis escalated the question because
> deciding what a seat is FOR when nobody dispatches it is a product-and-roles judgement, not a
> scheduling one). This document is the standing duty; `docs/coordination/roles/MIRA.md` remains the
> role. When the queue is empty, the waker calls you here.

## What you are for when nobody is dispatching you

**You are the seat that notices when the interface says something the product cannot back.** Not when
a dispatch asks you to — the dispatch queue only ever asks about work someone already knows about.
Your standing job is the surface nobody filed a ticket for.

The operator's test, from his own words on 2026-10-16: *does clicking it do what it says.* A dead
button, a badge that lies, an empty state with no reason, a form that pretends to save — those are
yours to find before he finds them.

## Standing checks — run these in order, each one able to FAIL

**1. Every provenance badge on every tab is earned.** Read `ui/src/lib/provenance.ts` and then, for
each tab the app renders, ask which `Provenance` value it reports and whether that value is reachable
in the running app right now.

- PASS: each tab's badge matches what its data source actually is, and every state
  (`live` / `ledger` / `seed` / `fixture` / `no-live` / `discovery-failed`) has a reachable path.
- FAIL, report: a tab wearing `LIVE` whose read is not live; a state with no reachable path; a badge
  that survives a failed hydration.

**2. Every button and link on every surface does something.** Walk the rendered UI — the real app if it
is running, otherwise `ui/src` plus the tests — and list every interactive element.

- PASS: each one either performs an action or is visibly disabled with a reason.
- FAIL, report: an enabled control whose handler is empty, absent, or a `console.log`; a link with no
  handler (the `WebLinksAddon` class of defect that hid in the live-seats pane).

**3. Every empty state says why it is empty.** Three different reasons must read differently:
*nothing recorded yet*, *recorded but unreadable*, *the read failed*. Collapsing two of them into one
grey "no data" is the defect the session-work contract's load-bearing requirement exists to prevent.

- PASS: each empty state names its reason.
- FAIL, report: an unexplained empty list, or two different reasons rendered identically.

**4. The frontend the operator can open is the frontend on track.** This one is not about the UI's
content — it is about whether the current `ui/src` is inside the app he actually launches.

- PASS: the embedded content of the running app is byte-equivalent to a fresh build of `ui/dist`.
- FAIL, report: any feature string present in a fresh build and absent from the app's embedded blobs,
  with the string, the file it lives in, and the commit that landed it.

**How to run check 4 — use the instrument that already exists.** `scripts/verify-tauri-bundle.sh`
decompresses Tauri's brotli-embedded blobs and content-matches. **Do not use `strings` and do not list
`Contents/Resources`** — the first finds nothing in a compressed binary, the second finds only
`icon.icns` in a perfectly good app. Both checks are wrong in both directions and have already misled
a seat today. If you write your own measurement, mutation-test it: break the thing it is supposed to
catch and confirm it goes red.

## When all four pass and there is still no dispatch

**Stop.** Do not invent a redesign, do not refactor a component for consistency's sake, and do not open
a card. If all four checks pass, the interface is honest today and there is nothing for you to do
until a dispatch arrives. Record that you ran them and what you measured — a pass with numbers is a
result, and the next person needs to know the checks were run rather than skipped.

If a check fails, the output is a **finding with evidence**, not a fix you apply unasked: name the
file, the line, the exact behaviour, and what the operator would see. Hand it to Lyra (product
judgement) or Aegis (routing). Escalation to Aegis is for cross-seat matters; you do not become a
second Aegis by having standing duties.
