"""S0 — normalization: per-harness adapters -> canonical turn stream.

Output spec (packet §1): one JSON object per turn —
    {seq, ts, role, kind, text, tool, refs}
plus three provenance fields required by other red lines:
    cwd        — session cwd in effect (S2 cwd-change signal)
    src_file   — path relative to AgentHistory root (L2 pointer, raw/)
    src_line   — 1-based line number in the raw file (L2 pointer)
    meta       — structured extras for kind=meta/event rows only

Mapping policy
--------------
codex: the `response_item` stream is canonical (it is the model's actual I/O).
`event_msg` rows are the UI mirror: plaintext `agent_reasoning` is the only
readable reasoning (response_item/reasoning is encrypted) so it is kept; every
other event_msg row duplicates a response_item and is recorded as meta.
claude-code: user/assistant message rows are canonical; `attachment` rows are
hook/side-channel records -> meta. Sidechain (sub-agent) rows are kept — they
are real session work.
cursor: read the existing chat_dump.json (no sqlite).
gemini: register-only — one meta row, no extraction.

Tolerance: an unrecognized row is recorded as a meta turn and counted
(`unknown_rows`); a malformed JSON line likewise (`unparseable_rows`). No
adapter ever fails the whole file.
"""

from __future__ import annotations

import json
import os
import re
from typing import Any, Iterator

# ---------------------------------------------------------------------------
# canonical record helpers
# ---------------------------------------------------------------------------

_PATH_RE = re.compile(
    r"(?:[A-Za-z0-9_.@-]+/)*[A-Za-z0-9_.@-]+\.(?:md|rs|ts|tsx|toml|json|jsonl|sh|py|sql|yml|yaml|js|mjs|cjs|css|html|lock|txt|pem|key)"
)
_URL_RE = re.compile(r"https?://[^\s\"'<>)\]}]+")
_LINE_REF_RE = re.compile(r"\b[\w./-]+\.(?:md|rs|ts|tsx|py|sh|sql|toml|json):\d+\b")
_MAX_REFS = 24


def extract_refs(*texts: str | None) -> list[str]:
    """Heuristic reference extraction: repo paths, path:line refs, URLs.

    Bounded and best-effort — refs are a retrieval aid, not evidence. The L2
    pointer (src_file/src_line) is the evidence.
    """
    refs: list[str] = []
    seen: set[str] = set()
    for text in texts:
        if not text:
            continue
        for m in _LINE_REF_RE.findall(text):
            if m not in seen:
                seen.add(m)
                refs.append(m)
        for m in _PATH_RE.findall(text):
            if "/" in m and m not in seen and not m.startswith("http"):
                seen.add(m)
                refs.append(m)
        for m in _URL_RE.findall(text):
            if m not in seen:
                seen.add(m)
                refs.append(m)
        if len(refs) >= _MAX_REFS:
            break
    return refs[:_MAX_REFS]


def _turn(seq: int, ts: str | None, role: str, kind: str, text: str | None,
          tool: str | None, refs: list[str], cwd: str | None,
          src_file: str, src_line: int, meta: dict | None = None) -> dict[str, Any]:
    return {
        "seq": seq,
        "ts": ts,
        "role": role,
        "kind": kind,
        "text": text,
        "tool": tool,
        "refs": refs,
        "cwd": cwd,
        "src_file": src_file,
        "src_line": src_line,
        "meta": meta or {},
    }


def _new_stats() -> dict[str, int]:
    return {
        "total_lines": 0,     # raw lines in the file
        "parsed": 0,          # canonical turns with kind != meta
        "meta_lines": 0,      # canonical turns with kind == meta
        "unknown_rows": 0,    # rows whose shape/type is not in the mapping table
        "unparseable_rows": 0,
    }


def _emit_meta(seq, ts, cwd, src_file, src_line, raw_type, extra=None):
    return _turn(seq, ts, "system", "meta", None, None, [], cwd, src_file, src_line,
                 {"raw_type": raw_type, **(extra or {})})


# ---------------------------------------------------------------------------
# codex adapter
# ---------------------------------------------------------------------------

_CODEX_META_EVENTS = {
    "token_count", "task_started", "task_complete", "turn_aborted",
    "context_compacted", "error", "exec_command_end", "patch_apply_end",
    "web_search_end", "user_message", "agent_message",
}


def adapt_codex(path: str, src_file: str) -> tuple[Iterator[dict], dict]:
    stats = _new_stats()
    cwd = None
    call_tools: dict[str, str] = {}   # call_id -> tool name (for outputs)

    def gen() -> Iterator[dict]:
        nonlocal cwd
        seq = 0
        with open(path, encoding="utf-8") as f:
            for lineno, line in enumerate(f, 1):
                stats["total_lines"] += 1
                line = line.strip()
                if not line:
                    stats["meta_lines"] += 1
                    continue
                try:
                    row = json.loads(line)
                except (json.JSONDecodeError, ValueError):
                    stats["unparseable_rows"] += 1
                    stats["meta_lines"] += 1
                    yield _emit_meta(seq, None, cwd, src_file, lineno, "<unparseable>")
                    seq += 1
                    continue

                rtype = row.get("type")
                payload = row.get("payload") if isinstance(row.get("payload"), dict) else {}
                ptype = payload.get("type")
                ts = row.get("timestamp")

                def meta(raw_type, extra=None):
                    nonlocal seq
                    stats["meta_lines"] += 1
                    t = _emit_meta(seq, ts, cwd, src_file, lineno, raw_type, extra)
                    seq += 1
                    return t

                # -- session / turn metadata -----------------------------------
                if rtype == "session_meta":
                    cwd = payload.get("cwd") or cwd
                    yield meta("session_meta", {"session_id": payload.get("id"),
                                                "originator": payload.get("originator")})
                    continue
                if rtype == "turn_context":
                    cwd = payload.get("cwd") or cwd
                    yield meta("turn_context", {"turn_id": payload.get("turn_id")})
                    continue

                # -- response_item: the canonical model I/O -------------------
                if rtype == "response_item":
                    if ptype == "message":
                        role = payload.get("role")
                        content = payload.get("content") or []
                        text = "\n".join(
                            part.get("text", "") for part in content
                            if isinstance(part, dict) and part.get("type") in
                            ("input_text", "output_text", "text")
                        ).strip()
                        if not text:
                            yield meta("response_item/message:empty")
                            continue
                        crole = {"user": "user", "assistant": "assistant"}.get(role, "system")
                        stats["parsed"] += 1
                        t = _turn(seq, ts, crole, "message", text, None,
                                  extract_refs(text), cwd, src_file, lineno)
                        seq += 1
                        yield t
                    elif ptype == "reasoning":
                        # encrypted_content only — unreadable; plaintext lives in
                        # event_msg/agent_reasoning (kept below).
                        yield meta("response_item/reasoning",
                                   {"encrypted": bool(payload.get("encrypted_content"))})
                    elif ptype == "function_call":
                        name = payload.get("name") or "unknown_tool"
                        call_tools[payload.get("call_id")] = name
                        args = payload.get("arguments") or ""
                        stats["parsed"] += 1
                        t = _turn(seq, ts, "assistant", "tool_call", args, name,
                                  extract_refs(args), cwd, src_file, lineno,
                                  {"call_id": payload.get("call_id")})
                        seq += 1
                        yield t
                    elif ptype == "function_call_output":
                        tool = call_tools.get(payload.get("call_id"), "unknown_tool")
                        out = payload.get("output")
                        text = out if isinstance(out, str) else json.dumps(out, ensure_ascii=False)
                        stats["parsed"] += 1
                        t = _turn(seq, ts, "user", "tool_result", text, tool, [],
                                  cwd, src_file, lineno,
                                  {"call_id": payload.get("call_id")})
                        seq += 1
                        yield t
                    elif ptype == "custom_tool_call":
                        name = payload.get("name") or "unknown_tool"
                        call_tools[payload.get("call_id")] = name
                        inp = payload.get("input") or ""
                        stats["parsed"] += 1
                        t = _turn(seq, ts, "assistant", "tool_call", inp, name,
                                  extract_refs(inp), cwd, src_file, lineno,
                                  {"call_id": payload.get("call_id")})
                        seq += 1
                        yield t
                    elif ptype == "custom_tool_call_output":
                        tool = call_tools.get(payload.get("call_id"), "unknown_tool")
                        out = payload.get("output")
                        text = out if isinstance(out, str) else json.dumps(out, ensure_ascii=False)
                        stats["parsed"] += 1
                        t = _turn(seq, ts, "user", "tool_result", text, tool, [],
                                  cwd, src_file, lineno,
                                  {"call_id": payload.get("call_id")})
                        seq += 1
                        yield t
                    elif ptype == "web_search_call":
                        action = payload.get("action") or {}
                        query = action.get("query") or ""
                        stats["parsed"] += 1
                        t = _turn(seq, ts, "assistant", "tool_call", query,
                                  "web_search", extract_refs(query), cwd,
                                  src_file, lineno, {"call_id": payload.get("call_id")})
                        seq += 1
                        yield t
                    else:
                        stats["unknown_rows"] += 1
                        yield meta(f"response_item/{ptype}")
                    continue

                # -- event_msg: UI mirror; only plaintext reasoning is unique --
                if rtype == "event_msg":
                    if ptype == "agent_reasoning":
                        text = (payload.get("text") or "").strip()
                        if text:
                            stats["parsed"] += 1
                            t = _turn(seq, ts, "assistant", "reasoning", text,
                                      None, extract_refs(text), cwd, src_file, lineno)
                            seq += 1
                            yield t
                            continue
                        yield meta("event_msg/agent_reasoning:empty")
                        continue
                    if ptype in _CODEX_META_EVENTS:
                        yield meta(f"event_msg/{ptype}")
                    else:
                        stats["unknown_rows"] += 1
                        yield meta(f"event_msg/{ptype}")
                    continue

                # -- compaction -------------------------------------------------
                if rtype == "compacted":
                    yield meta("compacted", {"replaced_turns": len(payload.get("replacement_history") or [])})
                    continue

                stats["unknown_rows"] += 1
                yield meta(str(rtype))

    return gen(), stats


# ---------------------------------------------------------------------------
# claude-code adapter
# ---------------------------------------------------------------------------

_CC_META_TYPES = {
    "last-prompt", "mode", "permission-mode", "atis-latch", "pr-link",
    "queue-operation", "file-history-snapshot", "file-history-delta",
    "cost-state",
}


def adapt_claude_code(path: str, src_file: str) -> tuple[Iterator[dict], dict]:
    stats = _new_stats()
    tool_names: dict[str, str] = {}    # tool_use_id -> tool name

    def gen() -> Iterator[dict]:
        nonlocal tool_names
        seq = 0
        with open(path, encoding="utf-8") as f:
            for lineno, line in enumerate(f, 1):
                stats["total_lines"] += 1
                line = line.strip()
                if not line:
                    stats["meta_lines"] += 1
                    continue
                try:
                    row = json.loads(line)
                except (json.JSONDecodeError, ValueError):
                    stats["unparseable_rows"] += 1
                    stats["meta_lines"] += 1
                    yield _emit_meta(seq, None, None, src_file, lineno, "<unparseable>")
                    seq += 1
                    continue

                rtype = row.get("type")
                ts = row.get("timestamp")
                cwd = row.get("cwd")

                def meta(raw_type, extra=None):
                    nonlocal seq
                    stats["meta_lines"] += 1
                    t = _emit_meta(seq, ts, cwd, src_file, lineno, raw_type, extra)
                    seq += 1
                    return t

                if rtype == "user":
                    msg = row.get("message") or {}
                    content = msg.get("content")
                    if isinstance(content, str):
                        if content.strip():
                            stats["parsed"] += 1
                            t = _turn(seq, ts, "user", "message", content, None,
                                      extract_refs(content), cwd, src_file, lineno)
                            seq += 1
                            yield t
                        else:
                            yield meta("user:empty")
                        continue
                    if isinstance(content, list):
                        emitted = False
                        for block in content:
                            if not isinstance(block, dict):
                                continue
                            btype = block.get("type")
                            if btype == "text":
                                text = (block.get("text") or "").strip()
                                if text:
                                    stats["parsed"] += 1
                                    t = _turn(seq, ts, "user", "message", text,
                                              None, extract_refs(text), cwd,
                                              src_file, lineno)
                                    seq += 1
                                    yield t
                                    emitted = True
                            elif btype == "tool_result":
                                tool = tool_names.get(block.get("tool_use_id"), "unknown_tool")
                                c = block.get("content")
                                if isinstance(c, list):
                                    text = "\n".join(
                                        p.get("text", "") for p in c
                                        if isinstance(p, dict) and p.get("type") == "text")
                                else:
                                    text = c if isinstance(c, str) else json.dumps(c, ensure_ascii=False)
                                stats["parsed"] += 1
                                t = _turn(seq, ts, "user", "tool_result", text or "",
                                          tool, [], cwd, src_file, lineno,
                                          {"tool_use_id": block.get("tool_use_id"),
                                           "is_error": bool(block.get("is_error"))})
                                seq += 1
                                yield t
                                emitted = True
                        if not emitted:
                            if content:
                                stats["unknown_rows"] += 1
                            yield meta("user:no-usable-content")
                        continue
                    stats["unknown_rows"] += 1
                    yield meta("user:unknown-shape")
                    continue

                if rtype == "assistant":
                    msg = row.get("message") or {}
                    content = msg.get("content")
                    if not isinstance(content, list):
                        stats["unknown_rows"] += 1
                        yield meta("assistant:unknown-shape")
                        continue
                    emitted = False
                    for block in content:
                        if not isinstance(block, dict):
                            continue
                        btype = block.get("type")
                        if btype == "text":
                            text = (block.get("text") or "").strip()
                            if text:
                                stats["parsed"] += 1
                                t = _turn(seq, ts, "assistant", "message", text,
                                          None, extract_refs(text), cwd,
                                          src_file, lineno)
                                seq += 1
                                yield t
                                emitted = True
                        elif btype == "thinking":
                            text = (block.get("thinking") or "").strip()
                            if text:
                                stats["parsed"] += 1
                                t = _turn(seq, ts, "assistant", "reasoning", text,
                                          None, extract_refs(text), cwd,
                                          src_file, lineno)
                                seq += 1
                                yield t
                                emitted = True
                        elif btype == "tool_use":
                            name = block.get("name") or "unknown_tool"
                            tool_names[block.get("id")] = name
                            inp = block.get("input")
                            text = inp if isinstance(inp, str) else json.dumps(inp, ensure_ascii=False)
                            stats["parsed"] += 1
                            t = _turn(seq, ts, "assistant", "tool_call", text,
                                      name, extract_refs(text), cwd,
                                      src_file, lineno, {"tool_use_id": block.get("id")})
                            seq += 1
                            yield t
                            emitted = True
                    if not emitted:
                        yield meta("assistant:no-content")
                    continue

                if rtype == "system":
                    text = row.get("content")
                    if isinstance(text, str) and text.strip():
                        stats["parsed"] += 1
                        t = _turn(seq, ts, "system", "event", text.strip(), None,
                                  [], cwd, src_file, lineno)
                        seq += 1
                        yield t
                    else:
                        yield meta("system:empty")
                    continue

                if rtype == "attachment":
                    att = row.get("attachment") or {}
                    yield meta(f"attachment:{att.get('type')}")
                    continue

                if rtype in _CC_META_TYPES:
                    yield meta(rtype)
                    continue

                stats["unknown_rows"] += 1
                yield meta(str(rtype))

    return gen(), stats


# ---------------------------------------------------------------------------
# cursor adapter — read the existing chat_dump.json (no sqlite)
# ---------------------------------------------------------------------------

def adapt_cursor(path: str, src_file: str) -> tuple[Iterator[dict], dict]:
    stats = _new_stats()

    def gen() -> Iterator[dict]:
        seq = 0
        try:
            with open(path, encoding="utf-8") as f:
                dump = json.load(f)
        except (OSError, json.JSONDecodeError, ValueError):
            stats["unparseable_rows"] += 1
            stats["meta_lines"] += 1
            yield _emit_meta(seq, None, None, src_file, 1, "<unreadable chat_dump>")
            return

        prompts = dump.get("prompts") or []
        generations = dump.get("generations") or []
        stats["total_lines"] = len(prompts) + len(generations)

        import datetime
        for i, p in enumerate(prompts, 1):
            text = (p.get("text") or "").strip()
            if not text:
                stats["meta_lines"] += 1
                yield _emit_meta(seq, None, None, src_file, i, "prompt:empty")
                seq += 1
                continue
            stats["parsed"] += 1
            yield _turn(seq, None, "user", "message", text, None,
                        extract_refs(text), None, src_file, i)
            seq += 1
        for j, g in enumerate(generations, len(prompts) + 1):
            text = (g.get("textDescription") or "").strip()
            unix_ms = g.get("unixMs")
            ts = (datetime.datetime.fromtimestamp(unix_ms / 1000, datetime.timezone.utc)
                  .isoformat().replace("+00:00", "Z")) if unix_ms else None
            if not text:
                stats["meta_lines"] += 1
                yield _emit_meta(seq, ts, None, src_file, j, "generation:empty")
                seq += 1
                continue
            stats["parsed"] += 1
            yield _turn(seq, ts, "assistant", "message", text, None,
                        extract_refs(text), None, src_file, j)
            seq += 1

    return gen(), stats


# ---------------------------------------------------------------------------
# gemini adapter — register-only (packet §1: must not block the mainline)
# ---------------------------------------------------------------------------

def adapt_gemini(path: str, src_file: str, session_id: str) -> tuple[Iterator[dict], dict]:
    stats = _new_stats()
    stats["registered_only"] = 1
    stats["total_lines"] = 1   # the registration row itself; nothing extracted

    def gen() -> Iterator[dict]:
        # One meta row: harness, session id, path. No extraction this round.
        stats["meta_lines"] += 1
        yield _emit_meta(0, None, None, src_file, 0, "gemini:registered-only",
                         {"harness": "gemini", "session_id": session_id,
                          "path": path,
                          "note": "history/ holds .project_root only; extraction deferred"})

    return gen(), stats


# ---------------------------------------------------------------------------
# dispatcher
# ---------------------------------------------------------------------------

def adapt(harness: str, path: str, src_file: str, session_id: str):
    if harness == "codex":
        return adapt_codex(path, src_file)
    if harness == "claude-code":
        return adapt_claude_code(path, src_file)
    if harness == "cursor":
        return adapt_cursor(path, src_file)
    if harness == "gemini":
        return adapt_gemini(path, src_file, session_id)
    raise ValueError(f"no adapter for harness {harness!r}")
