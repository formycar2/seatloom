# Aegis vs Lyra Alignment on Multica Benchmark

| Field | Value |
|---|---|
| Owner | Lyra |
| Date | 2026-04-28 |
| Status | Accepted |
| Purpose | Compare Aegis benchmark conclusions with Lyra's benchmark conclusions and produce an executable SeatLoom position |
| Related artifacts | `docs/coordination/reviews/2026-04-28-multica-product-benchmark.md` |
| Active SeatLoom contract | `docs/archive/product-history/prd-v0.4.md`, `docs/archive/product-history/interaction-spec-v1.0.md`, `docs/archive/product-history/acceptance-spec-v1.0.md` |

## 1. Overall verdict

Aegis and Lyra are aligned on the core product judgment:

1. Multica and SeatLoom are adjacent, not identical.
2. Multica is issue/workspace/agent-management centric.
3. SeatLoom is continuity/ledger/local-runtime centric.
4. SeatLoom should borrow execution rigor and runtime transparency, but must not drift into a generic issue tracker or notification center.

The main differences are not about direction, but about **where to bind the borrowed patterns into the SeatLoom contract** and **how much autonomy to give seats without breaking governance**.

## 2. Point-by-point comparison

| Aegis point | Lyra position | Alignment | SeatLoom decision |
|---|---|---|---|
| Seat as a first-class member in UI | Agree with the direction. Seat should feel like an accountable operator, not a hidden config row. | Strong agreement | Adopt as P1. Add avatar/icon, online state, runtime, recent activity, and direct actions from the Seat card. Avoid over-anthropomorphizing with social/profile-heavy UI in the baseline. |
| Add `claimed` to the WorkItem lifecycle | Agree with the need for explicit ownership transitions, but not necessarily as a new visible state in the frozen contract. | Partial agreement | Do **not** insert `claimed` into the active v0.4 state machine immediately. Prefer a `claim` event and clearer `ready -> active` gating first. Revisit only if ownership ambiguity remains after baseline. |
| Proactive blocker reporting from the seat | Agree on the user need, but disagree that it should become a separate parallel Inbox object type. | Partial agreement | Keep the active Inbox contract intact. Implement blocker reporting through structured evidence that drives existing states: `WorkItem -> blocked` or `Session -> input_required`, with required reason and suggested next action. This is P0-compatible. |
| Skill system / reusable capability definitions | Agree. This matches Lyra's own Skill Pack recommendation. | Strong agreement | Plan as P1. Extend ContextPack/LaunchPack into reusable `SkillPack` / `Seat Pack` assets, tied to execution surfaces rather than a generic marketplace. |
| Runtime health dashboard | Agree strongly. This is one of the highest-value borrowings. | Strong agreement | Split into two layers: P0 product requirement = capability truth must be visible; P1 UI delivery = full Runtime Health view with online state, concurrency, run duration, estimated cost, and queue/load signals. |
| Live activity feed / Timeline live mode | Agree on real-time visibility, but Timeline must remain an evidence/replay surface first. | Partial agreement | Add an optional Live Mode instead of turning Timeline into a chat stream by default. Preserve filters, object linking, and replay semantics from the active contract. |
| Seat limited autonomy: create sub-WorkItems, artifacts, handoff drafts | Agree in principle, but this is governance-sensitive. | Partial agreement | Allow earlier autonomy for `artifact creation` and `handoff draft creation`. Treat auto-created sub-WorkItems as proposals or drafts that require human acceptance. Keep this at P2 until governance is stable. |

## 3. Areas where Aegis and Lyra strongly converge

### 3.1 Product positioning

Both views converge that:

- Multica = agent PM/workspace product.
- SeatLoom = continuity and execution-trace product.
- Local-first continuity remains SeatLoom's differentiator.

### 3.2 Runtime truth matters

Both views push SeatLoom toward stronger runtime visibility:

- runtime identity
- runtime capability truth
- runtime health
- runtime accountability

Lyra adds one important clarification: capability truth should be visible not only in a dedicated dashboard, but directly in `Wrap`, `Attach`, `Switch Runtime`, and session detail.

### 3.3 Reusable operator capability matters

Aegis's `Skill` view and Lyra's `Skill Pack` view are effectively aligned. Both suggest that SeatLoom should preserve and reuse successful execution patterns rather than only replay raw handoffs.

## 4. Areas where Lyra adds constraints or corrections

### 4.1 Inbox semantics must stay human-only

Aegis is right that seats should be able to report blockers. Lyra adds a stricter contract rule:

- Inbox remains a human action queue.
- Seats do not "consume Inbox".
- Agent-originated signals must still map into human-actionable states, not turn Inbox into a generic notification feed.

This preserves the v0.4 contract and avoids the same semantic drift that Multica intentionally embraces.

### 4.2 `Claimed` is likely an event before it is a state

Aegis is correctly identifying an ownership gap. Lyra's concern is implementation inflation:

- the current contract already has `ready -> active` with an owner gate;
- a new durable status adds UX, event, persistence, and acceptance complexity;
- the likely missing thing is explicit claim evidence, not necessarily a new lifecycle node.

Recommended interpretation: add a `claim` action and event first; only promote to a formal status if ambiguity remains.

### 4.3 Live feed must not erase replay quality

Aegis sees the value of a real-time stream. Lyra agrees, but the SeatLoom Timeline cannot degrade into a noisy event wall. The replay contract still requires:

- readable summaries
- object references
- filters
- drill-through into evidence

So the right adaptation is `Timeline + Live toggle`, not `Timeline becomes chat`.

### 4.4 Seat autonomy needs stronger governance boundaries

Aegis's "limited autonomy" is strategically correct. Lyra's additional constraint is that autonomy must be bounded by the ledger:

- seats may generate artifacts;
- seats may draft handoffs;
- seats may propose child work;
- humans still own acceptance, promotion, and closure rules.

This prevents accidental product drift into free-form agent self-management.

## 5. Important Lyra points not emphasized in the Aegis note

These are high-value borrowings that Lyra believes should remain visible in the next cycle:

| Lyra-only emphasis | Why it matters |
|---|---|
| Execution-history drilldown | SeatLoom needs a reviewable evidence layer between raw terminal output and object state. |
| Chinese mixed-script readability system | This is directly relevant to the current UI quality issue; Multica's typography work is practically useful for SeatLoom now. |
| Visible automation model | If SeatLoom adds scheduled or automatic actions, every run must leave an inspectable ledger trace. |
| Human-only Inbox rule | Prevents semantic drift from action queue into notification center. |

## 6. Final SeatLoom synthesis

If the two benchmark views are merged into one execution position, the result is:

### Adopt now / shape into requirements

1. Runtime capability truth
2. Runtime health visibility
3. Structured blocker evidence mapped into existing Inbox rules
4. Richer execution-history evidence
5. Chinese-first readability and mixed-script typography rules

### Plan next

1. Seat card upgrade
2. SkillPack / Seat Pack system
3. Timeline Live Mode
4. Runtime auto-discovery

### Defer carefully

1. Formal `claimed` state
2. Autonomous sub-WorkItem creation
3. Broader seat self-management powers

## 7. Practical recommendation to the team

- Accept Aegis's benchmark as directionally correct.
- Apply Lyra's contract guardrails before converting the ideas into implementation tasks.
- Treat runtime truth and readability as the highest-value near-term borrowings.
- Avoid importing Multica's issue-board gravity or notification-centered mental model.

## 8. Acceptance outcome

Aegis reviewed the alignment memo and accepted Lyra's tighter guardrails on all five previously open difference points:

1. `claimed` should be an event before it becomes a formal state.
2. blocker reporting should reuse the existing `blocked` / `input_required` contract instead of introducing Inbox-type sprawl.
3. Timeline live behavior should remain an optional toggle; replay semantics stay primary.
4. seat autonomy should allow artifact and handoff-draft generation earlier, while formal child WorkItems still require human confirmation.
5. runtime health should be layered: capability truth visible at P0, independent dashboard at P1.

Result: there is no remaining strategic disagreement between Aegis and Lyra on the Multica benchmark adaptation. This benchmark can now be treated as an aligned product input for downstream packets.
