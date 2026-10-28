"""S2 — topic segmentation. THE WEAKEST LINK (self-built heuristic, no external
validation — Aegis's own words in the packet §1).

Output is a two-level tree (packet: "topic 树, parent→child"):
  level 1 topic     — coarse work theme, split by the STRONG signals:
                      silence > 30 min, cwd change, explicit task markers
                      (codex task_started/task_complete/turn_aborted/
                      context_compacted/compacted)
  level 2 exchange  — one user prompt + the work it triggered, split by
                      substantial user prompts and tool-call clustering
                      (>=CLUSTER_MIN consecutive tool turns = one burst)

UI noise (`<local-command-caveat>`, `/exit` echoes, `[Request interrupted by
user]`, `<task-notification>`) never splits anything — it attaches to the
current exchange. Micro-exchanges (<2 real turns, no tool calls) merge into
their neighbour.

If the output looks wrong, the packet says REPORT, do not force it. The topic
list goes to the operator as acceptance gate 1 — this module only produces
the candidate list.
"""

from __future__ import annotations

import datetime as _dt
import re
from typing import Any

SILENCE_MIN = 30
CLUSTER_MIN = 6  # consecutive tool turns that mark a work burst

# user messages that are UI plumbing, not work intent
_NOISE_PROMPT = re.compile(
    r"^\s*(<local-command-caveat>|<command-name>|<command-message>|"
    r"<local-command-stdout>|<local-command-stderr>|<task-notification>|"
    r"\[Request interrupted by user\]|\[Request interrupted by user for tool use\]|"
    r"Caveat: The messages below)",
)


def _parse_ts(ts: str | None) -> _dt.datetime | None:
    if not ts:
        return None
    try:
        return _dt.datetime.fromisoformat(ts.replace("Z", "+00:00"))
    except ValueError:
        return None


def _is_task_marker(turn: dict) -> bool:
    raw = (turn.get("meta") or {}).get("raw_type", "")
    return raw in ("event_msg/task_started", "event_msg/task_complete",
                   "event_msg/turn_aborted", "event_msg/context_compacted",
                   "compacted")


def _is_substantial_prompt(turn: dict) -> bool:
    if turn.get("kind") != "message" or turn.get("role") != "user":
        return False
    text = (turn.get("text") or "").strip()
    if not text:
        return False
    return not _NOISE_PROMPT.match(text)


def _label(turns: list[dict]) -> str:
    """Human label for eyeball review: first substantial prompt (else first
    assistant text) + tool profile."""
    prompt = ""
    for t in turns:
        if _is_substantial_prompt(t):
            prompt = (t.get("text") or "").strip().replace("\n", " ")
            break
    if not prompt:
        for t in turns:
            if t.get("kind") == "message" and t.get("role") == "assistant":
                prompt = (t.get("text") or "").strip().replace("\n", " ")
                break
    if len(prompt) > 90:
        prompt = prompt[:87] + "..."
    if not prompt:
        prompt = "(no user prompt)"
    tools: dict[str, int] = {}
    for t in turns:
        if t.get("kind") == "tool_call" and t.get("tool"):
            tools[t["tool"]] = tools.get(t["tool"], 0) + 1
    top = ", ".join(f"{k}x{v}" for k, v in
                    sorted(tools.items(), key=lambda kv: -kv[1])[:4])
    return prompt


def _materialize(segs: list[dict], prefix: str) -> list[dict]:
    out = []
    for i, seg in enumerate(segs):
        ts_list = [t.get("ts") for t in seg["turns"] if t.get("ts")]
        real = [t for t in seg["turns"] if t.get("kind") != "meta"]
        out.append({
            "id": f"{prefix}{i:03d}",
            "seq_start": seg["turns"][0]["seq"],
            "seq_end": seg["turns"][-1]["seq"],
            "ts_start": ts_list[0] if ts_list else None,
            "ts_end": ts_list[-1] if ts_list else None,
            "cwds": seg["cwds"],
            "turns": len(real),
            "label": _label(seg["turns"]),
            "signals": sorted(seg["signals"]),
        })
    return out


def segment_session(turns: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Split a session's canonical turns into a two-level topic tree.

    Returns level-1 topics; each carries an `exchanges` list (level 2).
    """
    # ---- pass 1: coarse topics (strong signals only) ----------------------
    topics: list[dict] = []
    cur: list[dict] = []
    cur_signals: set[str] = set()
    pending: str | None = None
    cur_cwds: list[str] = []
    prev_ts: _dt.datetime | None = None
    prev_cwd: str | None = None

    def flush_topic():
        nonlocal cur, cur_signals, pending
        if any(t.get("kind") != "meta" for t in cur):
            topics.append({"turns": cur, "signals": set(cur_signals),
                           "cwds": cur_cwds[:]})
        cur, cur_signals = [], set()
        if pending:
            cur_signals.add(pending)
            pending = None

    for t in turns:
        ts = _parse_ts(t.get("ts"))
        cwd = t.get("cwd")
        kind = t.get("kind")

        boundary = None
        if prev_ts and ts and (ts - prev_ts) > _dt.timedelta(minutes=SILENCE_MIN):
            boundary = f"silence>{SILENCE_MIN}min"
        elif cwd and prev_cwd and cwd != prev_cwd:
            boundary = "cwd-change"
        elif kind == "meta" and _is_task_marker(t):
            boundary = "task-marker"

        if boundary:
            pending = boundary
            flush_topic()

        if cwd and cwd not in cur_cwds:
            cur_cwds.append(cwd)
        cur.append(t)
        prev_ts = ts or prev_ts
        prev_cwd = cwd or prev_cwd
    flush_topic()

    # ---- pass 2: exchanges within each topic (fine signals) ---------------
    out_topics: list[dict] = []
    for topic in topics:
        segs: list[dict] = []
        cur = []
        cur_signals: set[str] = set()
        pending: str | None = None
        tool_run = 0
        saw_output = False

        def flush_ex():
            nonlocal cur, cur_signals, pending
            if cur:
                segs.append({"turns": cur, "signals": set(cur_signals),
                             "cwds": []})
            cur, cur_signals = [], set()
            if pending:
                cur_signals.add(pending)
                pending = None

        for t in topic["turns"]:
            kind = t.get("kind")
            if kind in ("tool_call", "tool_result"):
                tool_run += 1
            elif kind in ("message", "reasoning"):
                if kind == "message" and t.get("role") == "assistant":
                    saw_output = True
                tool_run = 0

            boundary = None
            if _is_substantial_prompt(t) and (saw_output or tool_run >= CLUSTER_MIN):
                boundary = "user-prompt" if saw_output else "user-prompt-after-cluster"

            if boundary:
                pending = boundary
                flush_ex()
                saw_output = False

            cur.append(t)
        flush_ex()

        # merge micro-exchanges (<2 real turns, no tool calls) into neighbour
        merged: list[dict] = []
        for seg in segs:
            real = [t for t in seg["turns"] if t.get("kind") != "meta"]
            has_tool = any(t.get("kind") == "tool_call" for t in real)
            if merged and len(real) < 2 and not has_tool:
                merged[-1]["turns"].extend(seg["turns"])
                merged[-1]["signals"] |= seg["signals"]
            else:
                merged.append(seg)

        exchanges = _materialize(merged, "e")
        # inherit cwds from the parent topic
        for ex in exchanges:
            ex["cwds"] = topic["cwds"]
        meta = _materialize([topic], "t")[0]
        meta["id"] = f"t{len(out_topics):03d}"
        out_topics.append({**meta, "exchanges": exchanges})
    return out_topics


def topic_tree(session_segments: dict[str, list[dict]]) -> dict:
    """parent->child tree: project -> session -> topic -> exchange."""
    return {
        "project": "seatloom",
        "sessions": [
            {"session_id": sid, "topics": segs}
            for sid, segs in session_segments.items()
        ],
    }
