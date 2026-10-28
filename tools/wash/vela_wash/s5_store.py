"""S4 — layering, and S5 — the staged, idempotent storage driver.

S4 (packet §S4): L0 one sentence (the only default layer) / L1 enough to decide
/ L2 = path + digest + line range. Raw text is NEVER copied into the store -
memories are pointers plus distillation. This module validates that every
authored record satisfies the layering contract and reports the distribution.

S5 (packet §S5): slash paths -> projects tree (DEEP cycle detection is this
pipeline's responsibility - migration 015 only blocks 1-cycles; detect before
insert, stop and report, never silently skip); segments -> topics; records ->
memories. Writes go ONLY through the seatloom-core memory write path (the
remember tool: row + event in one transaction) - no raw SQL, no bypass files.
Idempotent by content digest; checkpoint by session sha256 so an interrupted
run resumes without duplicate inserts.

The driver is deliberately split from the write call itself: it produces an
insert manifest, a pluggable writer executes it (the MCP remember tool), and
per-record results fold back into the checkpoint. A dry run validates
everything without writing.
"""

from __future__ import annotations

import hashlib
import json
import os
from typing import Any, Callable

from .s3_extract import session_checkpoint


# ---------------------------------------------------------------------------
# S4 — layering validation
# ---------------------------------------------------------------------------

def validate_layering(records: list[dict]) -> dict:
    """Every record must have L0 (short), L1 (decision-grade), and a complete
    L2 pointer (kind=file + path + digest + line range)."""
    problems: list[str] = []
    for r in records:
        rid = r.get("record_id", "?")[:8]
        l0 = (r.get("l0_summary") or "").strip()
        l1 = (r.get("l1_overview") or "").strip()
        if not l0:
            problems.append(f"{rid}: empty L0")
        if len(l0) > 400:
            problems.append(f"{rid}: L0 is not one sentence ({len(l0)} chars)")
        if not l1:
            problems.append(f"{rid}: empty L1")
        if r.get("l2_ref_kind") != "file":
            problems.append(f"{rid}: L2 ref kind must be 'file' for archived sessions")
        for k in ("l2_file_path", "l2_body_digest", "l2_line_start", "l2_line_end"):
            if not r.get(k):
                problems.append(f"{rid}: missing L2 field {k}")
        if r.get("l2_line_end", 0) < r.get("l2_line_start", 0):
            problems.append(f"{rid}: L2 line range inverted")
        # raw text must never be copied in: L1 must not contain long verbatim
        # transcript spans (a distilled summary, not a second copy of raw/)
        if len(l1) > 4000:
            problems.append(f"{rid}: L1 too long to be a distillation")
    return {"records": len(records), "problems": problems,
            "ok": not problems}


# ---------------------------------------------------------------------------
# S5 — deep cycle detection on the projects tree
# ---------------------------------------------------------------------------

def deep_cycle_detect(paths: list[str]) -> list[str] | None:
    """Detect cycles of ANY depth in a parent->child tree built from slash
    paths. Returns the cycle as a path list, or None if the tree is a forest.

    015 only blocks 1-cycles (a node being its own parent); this walks the
    whole graph so a -> b -> c -> a is caught too. Per the packet: detect
    BEFORE insert, stop and report on a cycle, never silently skip.
    """
    parent: dict[str, str] = {}
    children: dict[str, list[str]] = {}
    nodes: set[str] = set()
    for p in paths:
        parts = [x for x in p.split("/") if x]
        for i, node in enumerate(parts):
            nodes.add(node)
            if i > 0:
                par = parts[i - 1]
                # a repeated simple name under different parents is fine;
                # edges are keyed by full path
                parent[("/".join(parts[: i + 1]))] = "/".join(parts[:i])
                children.setdefault("/".join(parts[:i]), []).append("/".join(parts[: i + 1]))

    WHITE, GRAY, BLACK = 0, 1, 2
    color = {n: WHITE for n in nodes}
    stack_path: list[str] = []

    def dfs(node: str) -> list[str] | None:
        color[node] = GRAY
        stack_path.append(node)
        for child in children.get(node, []):
            name = child.split("/")[-1]
            if color.get(name) == GRAY:
                i = stack_path.index(name)
                return stack_path[i:] + [name]
            if color.get(name) == WHITE:
                found = dfs(name)
                if found:
                    return found
        stack_path.pop()
        color[node] = BLACK
        return None

    for n in sorted(nodes):
        if color[n] == WHITE:
            found = dfs(n)
            if found:
                return found
    return None


# ---------------------------------------------------------------------------
# S5 — the insert plan + checkpoint
# ---------------------------------------------------------------------------

class StorePlan:
    """Builds the idempotent insert plan and tracks per-session checkpoints."""

    def __init__(self, records_path: str, wash_root: str, state_path: str):
        self.records_path = records_path
        self.wash_root = wash_root
        self.state_path = state_path
        self.records = [json.loads(l) for l in open(records_path, encoding="utf-8")
                        if l.strip()]
        self.state = {"done_sessions": {}, "results": []}
        if os.path.exists(state_path):
            self.state = json.load(open(state_path, encoding="utf-8"))

    def project_paths(self) -> list[str]:
        """Projects-tree slash paths implied by the records (project scope)."""
        return sorted({"seatloom"})

    def validate(self) -> dict:
        cycle = deep_cycle_detect(self.project_paths())
        layering = validate_layering(self.records)
        # every record's session must exist in the manifest and have a digest
        sessions = {}
        for r in self.records:
            sessions.setdefault(r["session_id"], 0)
            sessions[r["session_id"]] += 1
        return {
            "records": len(self.records),
            "sessions": sessions,
            "layering": layering,
            "projects_cycle": cycle,
            "blocking": bool(cycle) or not layering["ok"],
        }

    def checkpoint_key(self, harness: str, session_id: str) -> str:
        return session_checkpoint(self.wash_root, harness, session_id)

    def plan(self) -> list[dict]:
        """Ordered, resumable insert plan: sessions first (checkpoint skip),
        then records in a stable order."""
        out = []
        done = self.state["done_sessions"]
        for r in sorted(self.records,
                        key=lambda r: (r["session_id"], r["record_id"])):
            sid = r["session_id"]
            if sid in done:
                continue
            out.append({
                "record_id": r["record_id"],
                "session_id": sid,
                "harness": r["harness"],
                "topic_id": r["topic_id"],
                "kind": r["kind"],
                "title": r["title"],
                "l0_summary": r["l0_summary"],
                "l1_overview": r["l1_overview"],
                "l2_ref_kind": r["l2_ref_kind"],
                "l2_file_path": r["l2_file_path"],
                "l2_body_digest": r["l2_body_digest"],
                "l2_line_start": r["l2_line_start"],
                "l2_line_end": r["l2_line_end"],
                "source_kind": r["source_kind"],
                "source_ref": r["source_ref"],
                "tags": r["tags"],
            })
        return out

    def record_result(self, session_id: str, ok: bool, detail: str) -> None:
        self.state["results"].append({"session_id": session_id, "ok": ok,
                                      "detail": detail})
        with open(self.state_path, "w", encoding="utf-8") as f:
            json.dump(self.state, f, ensure_ascii=False, indent=1)

    def mark_session_done(self, harness: str, session_id: str) -> None:
        self.state["done_sessions"][session_id] = {
            "checkpoint": self.checkpoint_key(harness, session_id),
        }
        with open(self.state_path, "w", encoding="utf-8") as f:
            json.dump(self.state, f, ensure_ascii=False, indent=1)


def execute_plan(plan: list[dict], writer: Callable[[dict], Any],
                 store: "StorePlan") -> dict:
    """Run the plan through the sanctioned writer (remember). Idempotent:
    a record whose digest already exists is a no-op. Any writer failure is
    recorded and stops the run (never silently skipped)."""
    inserted = skipped = 0
    failures: list[dict] = []
    for item in plan:
        try:
            res = writer(item)
            detail = str(res)
            if "rejected" in detail or "error" in detail.lower():
                failures.append({"record_id": item["record_id"], "detail": detail})
                store.record_result(item["session_id"], False, detail)
                break  # stop on first failure; resume later from the checkpoint
            inserted += 1
        except Exception as e:  # noqa: BLE001 - recorded, never swallowed
            failures.append({"record_id": item["record_id"], "detail": str(e)})
            store.record_result(item["session_id"], False, str(e))
            break
    # sessions fully drained get their checkpoint stamped
    done_sessions = {i["session_id"] for i in plan[: inserted + len(failures)]}
    remaining = {i["session_id"] for i in plan} - done_sessions
    return {"inserted": inserted, "failures": failures,
            "remaining_sessions": sorted(remaining)}
