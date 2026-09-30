# SeatLoom P0 Scope — Takeover and Capture

| Field | Value |
|---|---|
| template | T4 |
| subtype | design_proposal |
| id | review-2026-09-30-p0-scope-takeover-and-capture |
| status | draft — pending Mr. Zhang and Lyra review |
| author | aegis |
| date | 2026-09-30 |
| version | v1 |
| depends_on | `docs/coordination/reviews/2026-09-29-aegis-platform-design-macro-to-micro-v1.md`, `docs/PRODUCT_TRUTH.md`, `docs/prd-v0.5.md` |
| tags | p0, scope, capture, harness-agnostic, data-model, lineage |

## 0. What this document does

The platform design doc (the dependency above) measured the system and named the
abstraction changes. This document does one thing on top of it: **fix the P0
scope**, correcting two priority calls that document got wrong.

Authority: this reprioritization is a proposal from Aegis; product-scope truth is
Lyra's to ratify per `PRODUCT_TRUTH.md`.

## 1. Thesis

SeatLoom becomes the **control-and-record plane for agent work.** From now on,
new work is conducted through SeatLoom; SeatLoom captures what every harness and
every agent does into one harness-agnostic data model that is structured,
searchable, traceable, and internally linked.

Its job is not to *create* work in a product UI, and not to *reconstruct* the
history of past work. Its job is to **take over the live agent loop and record
it** — so that the record, not the harness, holds the truth.

## 2. Priority corrections against the platform design doc

| Design doc said | Corrected P0 |
|---|---|
| First milestone = "create a WorkItem by typing intent into IM" | **Wrong.** The first write is not a human filling a form; it is SeatLoom *capturing* what a live agent already does. See §7. |
| Historical ingestion of the ~16 GB corpus is the P0 main line | **P2.** Past sessions only need to be findable under their project; deep historical relationship reconstruction is deferred. |
| A7 budget as a first-class object | **P2.** No billing, no token accounting in P0. |
| A4 `external_ref` sequenced 5th | **P0 critical path, near the front** — it is the precondition for "harness-agnostic". |

The design doc's *analysis* stands (the write side does not exist; the ledger is
decorative; the abstraction changes A1–A6). Only its *priority framing* is
replaced by this document.

## 3. Core deliverable — the data-layer abstraction

Everything else rests on this. The abstraction must make all four true:

1. **Text becomes structure.** Any meaningful thing an agent does becomes a
   structured record, not a line of prose.
2. **Records are linked.** Every record carries its relations: who produced it,
   who consumed it, which session it came from, what it was based on upstream,
   what it affected downstream.
3. **Records belong to project and session, never to a harness.** Switching
   harness (Claude Code ↔ Codex ↔ Gemini) changes nothing about the record.
4. **Records are deterministically retrievable.** You find the exact record,
   section, or file — you do not ask a model to reread history.

## 4. P0 capabilities

1. **Takeover / capture.** SeatLoom non-destructively wraps a live agent session
   on any harness and captures its activity as canonical **semantic events** —
   which work it began, what artifact it produced, what it handed off — not just
   terminal bytes. Today SeatLoom mirrors bytes via `pipe-pane`; P0 mirrors
   meaning. Killing SeatLoom leaves the harness running (mirror, not replacement).
2. **Harness-agnostic identity and context.** A session's context belongs to
   SeatLoom, not the harness; switching harness does not lose it. Requires an
   explicit cross-system identity mapping (A4 `external_ref`).
3. **Project grouping.** New data is filed under its owning project
   automatically.
4. **Lineage.** Relations between records are traversable upstream (what was this
   based on) and downstream (what did this affect).
5. **Deterministic retrieval.** Find and locate the exact record / section /
   file.

## 5. P0 non-goals

- Billing, token budget, cost accounting — **P2**.
- Ingestion of the ~16 GB history and reconstruction of historical relationships
  — **P2** (past sessions need only be findable under their project).
- Unattended autonomy — out of scope.
- New L2 viewer surfaces — build capture and linkage first; views follow.

## 6. Takeover model (assumption, stated for rejection)

SeatLoom is the **control plane and record plane**; the harness is the
**execution plane**. SeatLoom drives and observes agents through the existing
tmux bridge (`send-keys` to drive, `pipe-pane` and file watchers to capture) and
lifts terminal activity into semantic events. It is **non-destructive**: the
harness runs independently and survives a SeatLoom crash.

This is the existing "mirror, not replacement" principle extended from bytes to
meaning. If the intended takeover is stronger (SeatLoom as the only launch path,
harnesses unable to run outside it) or weaker (pure passive observation with no
driving), this section is where that decision is recorded.

## 7. First milestone (redefined)

**Not** "a human types intent and a WorkItem is created."

**Instead:** a real agent session, on some harness, does its work — begins a
piece of work, produces an artifact, hands off to another seat — and those acts
are captured by SeatLoom as typed events, filed under the correct project,
retrievable and traceable, with no dependence on which harness produced them.

Concretely: the seat-to-seat coordination the team performs today by hand
(`tmux send-keys` dispatch, markdown deliveries) should begin flowing into
SeatLoom's data model on its own. The manual loop becomes the captured loop.

## 8. Technical basis (from platform design §A, reprioritized)

Carried into P0, in dependency order:

1. **A2** — typed, versioned event envelope (hand-written `FromStr`/`Display`).
2. **A3** — `event_object_refs.role` (`subject` / `actor` / `context`): the
   lineage join fabric (capability §4.4).
3. **A4** — `external_ref` (`pinned` / `derived`): harness-agnostic identity
   (capability §4.2). Elevated to the front of the path.
4. **A1** — event as the only write primitive; success test: projections
   rebuildable from the ledger. This is how capture stays trustworthy.
5. **A5 / A6** — dual-authority resolution and typing de-duplication: cleanups
   that ride along.

Dropped from P0: **A7 budget** → P2.

## 9. Acceptance framing

P0 is met when this is true and was not before:

> A record exists in SeatLoom because an agent, on some harness, did something —
> and that record is filed under its project, linked to what it came from and
> what it produced, findable by exact search, and unchanged by which harness was
> used.

Today zero records satisfy this: all 34 ledger events and all domain rows come
from seed SQL, and the one live writer only appends supervisor messages.

## 10. Open questions for Mr. Zhang and Lyra

1. **Takeover strength (§6).** Control-plane-over-tmux-bridge, non-destructive —
   accept as the P0 model? Or is a stronger "SeatLoom is the only launch path"
   intended?
2. **Capture granularity.** What is the smallest agent act worth a semantic
   event in P0 — session start/end, work begun, artifact produced, handoff? A
   short closed set beats an open-ended one.
3. **Harness coverage for P0.** Start with the harness all six seats run today
   (stepcode Claude Code over tmux), then generalize? Or must two harnesses be
   captured before P0 is called done, to prove harness-agnosticism is real?
