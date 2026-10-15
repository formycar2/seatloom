"""S2.5 — pruning for the LLM stage (Part 2, S3). Token leverage, no LLM.

Pruned away before anything is sent to an LLM (packet §1):
  * tool_result BODIES — keep tool name + success + byte count + LINE RANGE
    (the L2 pointer back into raw/);
  * retry rounds — a failed tool call that was retried with identical
    (tool, arguments) before the next real user prompt: only the final
    attempt survives;
  * meta rows — counted, never sent.

RED LINE: pruning NEVER affects L2 pointers. Every surviving tool_result
compact record carries its raw-file line range, so any extracted memory can
still be traced back to the exact lines of the original transcript.
"""

from __future__ import annotations

import hashlib
import json
from typing import Any


def _args_fingerprint(turn: dict) -> str:
    return hashlib.sha256((turn.get("text") or "").encode("utf-8")).hexdigest()[:12]


def _compact_tool_result(turn: dict) -> dict:
    body = turn.get("text") or ""
    ok = not bool((turn.get("meta") or {}).get("is_error"))
    out = dict(turn)
    out["text"] = json.dumps({
        "tool": turn.get("tool"),
        "ok": ok,
        "bytes": len(body.encode("utf-8")),
        "l2": [turn.get("src_file"), turn.get("src_line")],
    }, ensure_ascii=False)
    out["refs"] = []
    return out


def prune_session(turns: list[dict[str, Any]]) -> tuple[list[dict], dict]:
    """Return (llm_turns, stats). llm_turns keep full provenance."""
    stats = {
        "canonical_turns": len(turns),
        "pruned_meta": 0,
        "pruned_retry_rounds": 0,
        "pruned_toolresult_body_bytes": 0,
        "llm_turns": 0,
    }

    # --- pass 1: find retried (tool, args) fingerprints to drop -------------
    # A failed call is a retry victim if the same (tool, fingerprint) is
    # called again later in the same work burst (before the next real user
    # prompt). Map: (tool, fp) -> list of turn indices.
    attempts: dict[tuple, list[int]] = {}
    last_prompt_idx = -1
    prompt_bounds: list[int] = []
    for i, t in enumerate(turns):
        if t.get("kind") == "message" and t.get("role") == "user":
            prompt_bounds.append(i)
    next_prompt = {p: (prompt_bounds[j + 1] if j + 1 < len(prompt_bounds) else len(turns))
                   for j, p in enumerate(prompt_bounds)}

    for i, t in enumerate(turns):
        if t.get("kind") != "tool_call":
            continue
        key = (t.get("tool"), _args_fingerprint(t))
        attempts.setdefault(key, []).append(i)

    drop_idx: set[int] = set()
    for (tool, _fp), idxs in attempts.items():
        if len(idxs) < 2:
            continue
        # walk pairs: a failed call followed by an identical call within the
        # same burst is a retry round -> drop the failed call AND its result
        for a, b in zip(idxs, idxs[1:]):
            prior = [p for p in prompt_bounds if p < a]
            burst_end = next_prompt.get(max(prior), len(turns)) if prior else len(turns)
            if b >= burst_end:
                continue
            if _result_is_error(turns, a):
                drop_idx.add(a)
                r = _find_result(turns, a)
                if r is not None:
                    drop_idx.add(r)
                stats["pruned_retry_rounds"] += 1

    # --- pass 2: emit -------------------------------------------------------
    out: list[dict] = []
    for i, t in enumerate(turns):
        if i in drop_idx:
            continue
        kind = t.get("kind")
        if kind == "meta":
            stats["pruned_meta"] += 1
            continue
        if kind == "tool_result":
            body = t.get("text") or ""
            stats["pruned_toolresult_body_bytes"] += len(body.encode("utf-8"))
            out.append(_compact_tool_result(t))
            continue
        out.append(dict(t))

    stats["llm_turns"] = len(out)
    return out, stats


def _result_is_error(turns: list[dict], call_idx: int) -> bool:
    r = _find_result(turns, call_idx)
    if r is None:
        return False
    return bool((turns[r].get("meta") or {}).get("is_error"))


def _find_result(turns: list[dict], call_idx: int) -> int | None:
    """Index of the tool_result matching the tool_call at call_idx (by
    call id when present, else the next tool_result turn)."""
    call = turns[call_idx]
    cid = (call.get("meta") or {}).get("call_id") or (call.get("meta") or {}).get("tool_use_id")
    if cid:
        for j in range(call_idx + 1, len(turns)):
            m = turns[j].get("meta") or {}
            if m.get("call_id") == cid or m.get("tool_use_id") == cid:
                if turns[j].get("kind") == "tool_result":
                    return j
            if turns[j].get("kind") == "tool_call":
                break
        return None
    for j in range(call_idx + 1, len(turns)):
        k = turns[j].get("kind")
        if k == "tool_result":
            return j
        if k == "tool_call":
            return None
    return None
