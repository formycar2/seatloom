# Dispatch: P0 step 4 — MCP emit server (capture substrate, end-state kickoff)

[Aegis -> Lyra] 奔终态 kickoff. Route to **Nimbus** (opus). This is the capture substrate the whole
end-state rides on (UM2 timeline, step-6 governed dispatch, step-5 first event all depend on it).

| Field | Value |
|---|---|
| template | T3 |
| subtype | dispatch |
| id | AEGIS-2026-10-08-p0-step4-mcp-emit |
| status | dispatched |
| author | aegis |
| date | 2026-10-08 |
| version | v1 |
| to | lyra |
| priority | P0 |
| milestone | P0 capture loop — events flow (end-state) |
| depends_on | migration 010 (vocab/role/schema_version), write API A1 (step 2, merged) |
| tags | p0, dispatch, mcp, capture, nimbus |

## Nimbus (PI/opus) — build the MCP emit server
A minimal MCP server exposing **`emit_event(type, payload, refs)`** that is the ONLY sanctioned way
a seat writes a governed event:
1. **Routes through the write API (A1)** — `append(event) → projections` in one transaction. Never
   mutate a projection directly. (Reuse step-2's append path.)
2. **Schema-validated against migration 010**, rejecting malformed events:
   - `type` ∈ the 12-value vocabulary (SessionStarted, WorkItemCreated, WorkItemStatusChanged,
     ArtifactCreated, HandoffSent, HandoffAccepted, HandoffCompleted, ReviewVerdictIssued,
     SeatDelegationIssued, SeatDelegationClosed, SupervisorMessage, SeatResponse).
   - `schema_version` set explicitly (per-type, ≥1).
   - `refs` carry `role` ∈ {subject, actor, context}; **exactly one `subject`** per event (010's
     partial-unique index guarantees at-most-one; the API enforces presence).
   - hand-written FromStr/Display at the Rust⇄PG TEXT boundary (ENGINEERING_STANDARDS).
3. **Seat connection:** a seat reaches it via pi's MCP (`pi mcp` / an entry in `~/.pi/agent/...`);
   confirm the wiring so a seat can call `emit_event`. Carry the verbatim value-domain contract.
4. Work in your worktree; zero-regression gates.

## Acceptance (Flux, on real Postgres)
A seat calls `emit_event(ArtifactCreated, …, refs=[subject=…,actor=…])` via MCP → it lands in
`canonical_events` via the write API, with a projection update in the same txn, and is retrievable.
A malformed event (bad type / zero subjects / missing schema_version) is REJECTED. Verify on a
scratch DB in a worktree.

## After (the end-state path continues)
step 6 governed dispatch (G19) rides on this → step 5 first real captured event → UM2 timeline UI.
Lyra: dispatch Nimbus now; on delivery, Flux-verify; on PASS, merge. Don't drop the hops — Aegis is
watching and will nudge, but drive it.
