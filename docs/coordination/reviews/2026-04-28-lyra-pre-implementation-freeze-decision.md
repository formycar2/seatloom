# Lyra Decision - Design Clarity Before Implementation

| Field | Value |
|---|---|
| Owner | Lyra |
| Date | 2026-04-28 |
| Status | Active |
| Decision type | Execution sequencing / implementation gate |
| Applies to | P0, P1, P2 product scope planning |

## 1. Decision

Implementation should **not** begin until the product story is clear enough to express SeatLoom's value without contract churn.

This means we must clarify, in files, before implementation starts:

- the user value of each P0, P1, and P2 capability
- the end-to-end data flow for each capability
- the story/scenario coverage for each capability
- the boundary between what is implemented now and what is intentionally deferred

## 2. Lyra position

I agree with the design-first sequencing request.

For SeatLoom's current stage, implementation before cross-priority clarity would create three risks:

1. P0 flows get implemented against unstable assumptions.
2. P1/P2 later force rework because upstream shell, object model, or interaction choices were underspecified.
3. The team optimizes screens before the product value chain is fully legible.

The correct order is:

1. clarify product value
2. clarify scenario truth
3. clarify data movement and state transitions
4. freeze the baseline
5. start implementation

## 3. Required clarity baseline before implementation

### P0

P0 must be fully explicit at contract level:

- value statement
- happy path and failure path scenarios
- user actions and system responses
- data objects touched
- state transitions
- acceptance criteria

### P1

P1 does not need implementation-ready UI detail yet, but it must be clear enough to prevent P0 design mistakes:

- why it exists
- what user/job it serves
- what data it reads and writes
- what P0 surfaces must leave room for it
- what is intentionally not in P0

### P2

P2 must also be stated, even if lightly, so we do not accidentally close future doors:

- future value hypothesis
- likely entry points in the product
- expected data dependencies
- non-goals for the current cycle

## 4. Exit criteria for implementation start

Implementation may start only when all of the following are true:

- P0/P1/P2 value map is written and reviewed
- P0/P1/P2 scenario map is written and reviewed
- major data flows and object transitions are written and reviewed
- cross-document conflicts are closed or explicitly waived
- Lyra marks `Product Baseline Freeze` as ready
- Aegis confirms the stage gate as `GO`

## 5. What this does not mean

This decision does **not** mean:

- every P1/P2 screen must be pixel-complete before P0 starts
- every future edge case must be solved now
- implementation is blocked on speculative architecture work

It **does** mean:

- no implementation against ambiguous product meaning
- no hidden scope in chat-only decisions
- no "build first, reinterpret later" loop

## 6. Immediate next outputs

Before implementation, Lyra should drive three baseline artifacts:

1. P0/P1/P2 value map
2. cross-priority data-flow map
3. scenario coverage matrix with happy/failure/deferred cases

These artifacts should become the operational baseline for Mira, Nimbus, and Flux packets.
