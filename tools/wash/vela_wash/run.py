"""run.py — orchestrate S0 -> S1 -> S2 -> S2.5 over the seatloom manifest.

Zero LLM. Read-only on AgentHistory/raw. All artifacts land in the scratch
directory (gitignored .local/wash by default) — Part 2 (S3/S5) consumes them.

Usage:
    python3 -m vela_wash.run [--out DIR] [--session SID]

Outputs per session (under OUT/<harness>__<session_id>/):
    s0.canonical.jsonl   raw canonical stream (scratch only, NEVER to an LLM)
    s1.redacted.jsonl    redacted canonical stream
    s25.llm.jsonl        pruned LLM-facing stream (Part 2 input)
    stats.json           per-stage stats for this session
Global (under OUT):
    stats.json           per-session + totals
    topics.json          topic tree (S2 output)
    topics.md            human-readable topic list (acceptance gate 1)
    s1_report.json       redaction counters + gate-2 post-scan result
"""

from __future__ import annotations

import argparse
import json
import os
import sys

from . import manifest
from .s0_normalize import adapt
from .s1_redact import redact_turn, scan_text
from .s2_segment import segment_session, topic_tree
from .s25_prune import prune_session


def _write_jsonl(path: str, rows) -> int:
    n = 0
    with open(path, "w", encoding="utf-8") as f:
        for r in rows:
            f.write(json.dumps(r, ensure_ascii=False) + "\n")
            n += 1
    return n


def process_session(harness: str, session_id: str, relpath: str, out_root: str) -> dict:
    src_file = manifest.session_rel(relpath)
    path = manifest.session_path(relpath)
    session_dir = os.path.join(out_root, f"{harness}__{session_id}")
    os.makedirs(session_dir, exist_ok=True)

    # -- S0 ------------------------------------------------------------------
    gen, s0_stats = adapt(harness, path, src_file, session_id)
    canonical = list(gen)
    s0_stats["canonical_turns"] = len(canonical)
    _write_jsonl(os.path.join(session_dir, "s0.canonical.jsonl"), canonical)

    # -- S1 ------------------------------------------------------------------
    counters: dict[str, int] = {}
    redacted = [redact_turn(t, counters) for t in canonical]
    _write_jsonl(os.path.join(session_dir, "s1.redacted.jsonl"), redacted)

    # gate-2 post-scan: the redacted stream must contain no surviving hits
    survivors = 0
    survivor_samples: list[str] = []
    for t in redacted:
        for field in ("text",):
            v = t.get(field)
            if v and scan_text(v):
                survivors += 1
                if len(survivor_samples) < 5:
                    survivor_samples.append(f"{src_file}:{t.get('src_line')}")
        for r in t.get("refs") or []:
            if scan_text(r):
                survivors += 1

    # -- S2 ------------------------------------------------------------------
    segments = segment_session(redacted)

    # -- S2.5 ----------------------------------------------------------------
    llm_turns, s25_stats = prune_session(redacted)
    _write_jsonl(os.path.join(session_dir, "s25.llm.jsonl"), llm_turns)

    raw_bytes = os.path.getsize(path)
    llm_bytes = os.path.getsize(os.path.join(session_dir, "s25.llm.jsonl"))
    stats = {
        "harness": harness,
        "session_id": session_id,
        "src_file": src_file,
        "raw_bytes": raw_bytes,
        "s0": s0_stats,
        "s1": {"redactions": counters, "post_scan_survivors": survivors,
               "post_scan_samples": survivor_samples},
        "s2": {"topics": len(segments),
               "exchanges": sum(len(t.get("exchanges", [])) for t in segments)},
        "s25": {**s25_stats, "llm_bytes": llm_bytes,
                "bytes_kept_pct": round(100.0 * llm_bytes / raw_bytes, 2) if raw_bytes else 0.0},
    }
    with open(os.path.join(session_dir, "stats.json"), "w", encoding="utf-8") as f:
        json.dump(stats, f, ensure_ascii=False, indent=2)

    return {"stats": stats, "segments": segments}


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description="vela washing pipeline, part 1 (zero LLM)")
    ap.add_argument("--out", default=os.path.join(".local", "wash"))
    ap.add_argument("--session", help="process only this session id")
    args = ap.parse_args(argv)

    out_root = os.path.abspath(args.out)
    os.makedirs(out_root, exist_ok=True)

    results = []
    for harness, sid, relpath, _note in manifest.SESSIONS:
        if args.session and sid != args.session:
            continue
        path = manifest.session_path(relpath)
        if not os.path.exists(path):
            print(f"WARN missing raw file: {path}", file=sys.stderr)
            continue
        print(f"[wash] {harness}/{sid}")
        results.append(process_session(harness, sid, relpath, out_root))

    # global rollups
    totals = {
        "sessions": len(results),
        "raw_lines": sum(r["stats"]["s0"]["total_lines"] for r in results),
        "canonical_turns": sum(r["stats"]["s0"]["canonical_turns"] for r in results),
        "parsed_turns": sum(r["stats"]["s0"]["parsed"] for r in results),
        "meta_turns": sum(r["stats"]["s0"]["meta_lines"] for r in results),
        "unknown_rows": sum(r["stats"]["s0"]["unknown_rows"] for r in results),
        "unparseable_rows": sum(r["stats"]["s0"]["unparseable_rows"] for r in results),
        "raw_bytes": sum(r["stats"]["raw_bytes"] for r in results),
        "llm_turns": sum(r["stats"]["s25"]["llm_turns"] for r in results),
        "llm_bytes": sum(r["stats"]["s25"]["llm_bytes"] for r in results),
        "pruned_toolresult_body_bytes": sum(r["stats"]["s25"]["pruned_toolresult_body_bytes"] for r in results),
        "pruned_retry_rounds": sum(r["stats"]["s25"]["pruned_retry_rounds"] for r in results),
        "s1_post_scan_survivors": sum(r["stats"]["s1"]["post_scan_survivors"] for r in results),
    }
    redactions: dict[str, int] = {}
    for r in results:
        for k, v in r["stats"]["s1"]["redactions"].items():
            redactions[k] = redactions.get(k, 0) + v

    with open(os.path.join(out_root, "stats.json"), "w", encoding="utf-8") as f:
        json.dump({"totals": totals, "redactions": redactions,
                   "sessions": [r["stats"] for r in results]}, f,
                  ensure_ascii=False, indent=2)

    tree = topic_tree({r["stats"]["session_id"]: r["segments"] for r in results})
    with open(os.path.join(out_root, "topics.json"), "w", encoding="utf-8") as f:
        json.dump(tree, f, ensure_ascii=False, indent=2)

    with open(os.path.join(out_root, "s1_report.json"), "w", encoding="utf-8") as f:
        json.dump({"redactions_by_type": redactions,
                   "post_scan_survivors": totals["s1_post_scan_survivors"]}, f,
                  ensure_ascii=False, indent=2)

    with open(os.path.join(out_root, "topics.md"), "w", encoding="utf-8") as f:
        f.write("# seatloom — S2 topic tree (acceptance gate 1: operator eyeball)\n\n")
        f.write("Level-1 topics split on STRONG signals: silence>30min / cwd-change / "
                "task-marker. Level-2 exchanges split on substantial user prompts / "
                "tool-call clustering. UI noise never splits.\n\n")
        for sess in tree["sessions"]:
            f.write(f"## session {sess['session_id']} — {len(sess['topics'])} topics\n\n")
            for tp in sess["topics"]:
                sig = ",".join(tp["signals"]) or "-"
                f.write(f"### {tp['id']} [{tp['turns']} turns, {tp['ts_start']} -> "
                        f"{tp['ts_end']}] signals: {sig}\n")
                f.write(f"  {tp['label']}\n")
                for ex in tp["exchanges"]:
                    f.write(f"  - {ex['id']} [{ex['turns']} turns] {ex['label']}\n")
            f.write("\n")

    print(json.dumps(totals, ensure_ascii=False, indent=2))
    print(f"[wash] artifacts under {out_root}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
