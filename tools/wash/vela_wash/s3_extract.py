"""S3 — extraction: author six typed records from the LLM-facing stream.

This is the ONLY LLM stage of the pipeline. The washing seat (unlimited
stepplan-wash budget) reads the pruned, redacted per-topic streams and authors
records; this module provides the rendering (what to read), the record shape
(the 018 value-domain contract verbatim), idempotent persistence (content
digest + per-session sha256 checkpoint) and stats.

Record shape (VALUE-DOMAIN-2026-10-10-memory-hardening-018-v1.md):
  kind       decision | failure | fact | preference | artifact | unresolved   (G1)
  confidence quoted | inferred                                            (G4)
  l2         l2_ref_kind='file', source_kind='archived_session' =>
             l2_file_path + l2_line_start/end REQUIRED                    (G5)
             l2_file_path is relative to the AgentHistory root
             (raw/<harness>/<file>); l2_body_digest = sha256 of the exact
             raw line range, so gate 5 can recompute it.

RED LINE: a record whose L2 line range cannot be recovered is NOT stored.
"""

from __future__ import annotations

import hashlib
import json
import os
from typing import Any

KINDS = ("decision", "failure", "fact", "preference", "artifact", "unresolved")
CONFIDENCES = ("quoted", "inferred")


def _clip(s: str, n: int) -> str:
    s = s.replace("\n", " ⏎ ")
    return s if len(s) <= n else s[: n - 1] + "…"


def render_turn(t: dict, text_clip: int = 700, args_clip: int = 300) -> str:
    """One readable line per turn, carrying the L2 pointer inline."""
    head = (f"[{t['seq']:>5} {str(t.get('ts'))[5:16]} {t['role']:<9}"
            f"{t['kind']:<12} src:{t['src_line']}")
    kind = t["kind"]
    if kind == "tool_call":
        refs = ",".join(t.get("refs") or [])[:120]
        return f"{head}] TOOL {t.get('tool')} {_clip(t.get('text') or '', args_clip)} refs=[{refs}]"
    if kind == "tool_result":
        return f"{head}] {_clip(t.get('text') or '', 160)}"
    return f"{head}] {_clip(t.get('text') or '', text_clip)}"


def load_session_turns(wash_root: str, harness: str, session_id: str) -> list[dict]:
    path = os.path.join(wash_root, f"{harness}__{session_id}", "s25.llm.jsonl")
    with open(path, encoding="utf-8") as f:
        return [json.loads(l) for l in f]


def render_topic(turns: list[dict], topic: dict, **kw) -> str:
    """Render one topic's exchange window (seq range from the topic tree)."""
    lo, hi = topic["seq_start"], topic["seq_end"]
    window = [t for t in turns if lo <= t["seq"] <= hi]
    lines = [render_turn(t, **kw) for t in window]
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# records
# ---------------------------------------------------------------------------

def _sha256_file_range(path: str, start: int, end: int) -> str:
    """sha256 of the exact raw line range (gate-5 recomputable)."""
    h = hashlib.sha256()
    with open(path, encoding="utf-8") as f:
        for i, line in enumerate(f, 1):
            if i > end:
                break
            if i >= start:
                h.update(line.encode("utf-8"))
    return h.hexdigest()


def content_digest(rec: dict) -> str:
    """Idempotency key: same title+kind+l0+l2 range => same record."""
    h = hashlib.sha256()
    for k in ("kind", "title", "l0_summary", "l2_file_path",
              "l2_line_start", "l2_line_end"):
        h.update(str(rec.get(k, "")).encode("utf-8"))
        h.update(b"\x00")
    return h.hexdigest()


class RecordStore:
    """Append-only JSONL store with content-digest idempotency."""

    def __init__(self, path: str, agenthistory_root: str):
        self.path = path
        self.root = agenthistory_root
        self.seen: set[str] = set()
        self.records: list[dict] = []
        if os.path.exists(path):
            with open(path, encoding="utf-8") as f:
                for line in f:
                    r = json.loads(line)
                    self.seen.add(r["record_id"])
                    self.records.append(r)

    def add(self, *, kind: str, confidence: str, title: str, l0_summary: str,
            l1_overview: str, session_id: str, harness: str, topic_id: str,
            l2_file_path: str, l2_line_start: int, l2_line_end: int,
            evidence_seqs: list[int], tags: list[str] | None = None) -> dict:
        if kind not in KINDS:
            raise ValueError(f"kind {kind!r} not in G1 vocab")
        if confidence not in CONFIDENCES:
            raise ValueError(f"confidence {confidence!r} not in G4 vocab")
        if l2_line_end < l2_line_start:
            raise ValueError("l2_line_end < l2_line_start")
        rec = {
            "kind": kind,
            "confidence": confidence,
            "title": title,
            "l0_summary": l0_summary,
            "l1_overview": l1_overview,
            "session_id": session_id,
            "harness": harness,
            "topic_id": topic_id,
            "source_kind": "archived_session",
            "source_ref": f"{harness}:{session_id}",
            "l2_ref_kind": "file",
            "l2_file_path": l2_file_path,
            "l2_line_start": l2_line_start,
            "l2_line_end": l2_line_end,
            "evidence_seqs": evidence_seqs,
            "tags": tags or [],
        }
        rec["record_id"] = content_digest(rec)
        if rec["record_id"] in self.seen:
            return rec  # idempotent no-op
        rec["l2_body_digest"] = _sha256_file_range(
            os.path.join(self.root, l2_file_path), l2_line_start, l2_line_end)
        self.seen.add(rec["record_id"])
        self.records.append(rec)
        with open(self.path, "a", encoding="utf-8") as f:
            f.write(json.dumps(rec, ensure_ascii=False) + "\n")
        return rec

    def stats(self) -> dict:
        by_kind: dict[str, int] = {}
        by_conf: dict[str, int] = {}
        by_session: dict[str, int] = {}
        for r in self.records:
            by_kind[r["kind"]] = by_kind.get(r["kind"], 0) + 1
            by_conf[r["confidence"]] = by_conf.get(r["confidence"], 0) + 1
            by_session[r["session_id"]] = by_session.get(r["session_id"], 0) + 1
        return {"total": len(self.records), "by_kind": by_kind,
                "by_confidence": by_conf, "by_session": by_session}


def session_checkpoint(wash_root: str, harness: str, session_id: str) -> str:
    """sha256 of the session's LLM-facing stream — the S5 resume key."""
    path = os.path.join(wash_root, f"{harness}__{session_id}", "s25.llm.jsonl")
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()
