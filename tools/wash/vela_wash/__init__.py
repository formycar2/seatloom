"""Vela washing pipeline — Part 1 (S0/S1/S2/S2.5, zero LLM).

SeatLoom corpus-washing seat. Normalizes historical agent transcripts of the
`seatloom` project into a canonical turn stream, redacts secrets, segments
topics, and prunes for LLM consumption.

Stages:
  S0  normalize  — per-harness adapters -> canonical turn stream
                   {seq, ts, role, kind, text, tool, refs} (+ provenance)
  S1  redact     — secret scrubbing BEFORE anything reaches an LLM or a table
  S2  segment    — topic segmentation (self-built heuristic, weakest link)
  S2.5 prune     — token leverage for the LLM stage (Part 2, S3)

Hard rules (AGENTS.md red lines):
  * Redaction happens after normalization and BEFORE any LLM call / table write.
  * Every canonical turn carries its raw-file line range (L2 pointer); pruning
    never touches those pointers.
  * raw/ is read-only. Nothing here writes to AgentHistory.
"""

__version__ = "1.0.0"
