"""S5 write-in — the real insert into the live knowledge layer.

Runs AFTER the environment precondition is met (the seat's MCP process must
advertise `open_topic` and the 018-typed `remember`). This script never writes
to the database itself: it produces the exact, validated write manifest that
the sanctioned writers (`open_topic`, `remember`) execute, plus a repair log
for every record whose staged L2 anchor did not resolve against raw/.

Why a repair pass exists at all (found at write-in time, 2026-10-11):
  * 3 records cited line ranges past the end of their raw file. Their
    `l2_body_digest` was the sha256 of an EMPTY input (`e3b0c442...`) — the
    tell-tale of `_sha256_file_range` reading nothing because `start` was
    beyond EOF. Root cause: those anchors were numbered against the LIVE
    session transcript (~/.claude/projects/... , 13,596 lines) instead of the
    archived `raw/` copy (7,044 lines). The live transcript is NOT the corpus
    of record — raw/ is (AGENTS.md red line 3).
  * 3 records carried a `record_id` that no longer equalled the content digest
    of their own fields — a stale idempotency key, which would let a re-run
    insert a duplicate.
  * 1 record's single claim spans four disjoint evidence regions, so no
    honest line range exists for it. Dropped, not force-anchored.

Red lines enforced here, before any write:
  * every L2 range resolves inside raw/ and its sha256 recomputes exactly;
  * every record's idempotency key equals the digest of what is written;
  * no secret shape survives in the text about to enter the table;
  * a record that cannot be anchored is dropped and counted, never stored.

Usage:  python3 tools/wash/s5_writein.py            # validate + emit manifest
        python3 tools/wash/s5_writein.py --check     # validate only, no files
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

from vela_wash.s3_extract import content_digest  # noqa: E402
from vela_wash.s5_store import validate_layering  # noqa: E402

AH = os.path.expanduser("~/Documents/GitHub/AgentHistory")
RECORDS = os.path.join(HERE, ".local", "wash", "s3", "records.jsonl")
TOPICS = os.path.join(HERE, ".local", "wash", "topics.json")
OUT_DIR = os.path.join(HERE, ".local", "wash", "s5")
MANIFEST = os.path.join(OUT_DIR, "writein_manifest.json")

# ---------------------------------------------------------------------------
# Repairs — each one is an evidence-backed re-anchor, verified by re-reading
# raw/ before it is accepted. Nothing is re-anchored on intuition.
# ---------------------------------------------------------------------------
REANCHOR = {
    # "AgentHistory archive had silently gone stale for a week"
    # evidence: the archive-gap probe at 6994 ("关键缺口:它已经过期一周"),
    # 7000 (the conclusion), 7031 ("raw/claude-code 全是 9/30,确认停滞"),
    # 7043 ("远超 9/30 的 14M 旧快照").
    "0dcce59c": (6994, 7044),
    # "The 2026-09-30 AgentHistory archive is a hand-off-oriented corpus"
    # evidence: 6981 (raw+summaries+by-project+manifest+README inventory),
    # 6993 and 7000 (the hand-off-oriented reading, README's stated purpose).
    "7ea2faad": (6981, 7000),
}

# Claims that span several disjoint evidence regions: no single line range can
# carry them honestly. Dropped rather than force-anchored.
DROP = {
    "56ccb8df": (
        "Codex history is 247 files / 14GB on disk with a ready index; Claude "
        "transcripts live under stepgo launch dirs",
        "claim spans four disjoint raw regions (336 .codex 14G du; 484-501 "
        "rollout count + session_index; 572-579 stepgo launch homes; "
        "7024-7044 seats really write under ~/.stepgo launch dirs) and its L1 "
        "is about the $200/day codex credit, not about file locations — no "
        "single line range supports the record as written; original anchor "
        "6834-7085 was numbered against the live transcript, not raw/",
    ),
}

# Secret-shape gate — the same shapes S1 enforces, re-run on the DISTILLED
# text because that text is about to enter a table (AGENTS.md red line 1:
# redact before anything is stored, not only before the LLM).
SECRET_PATTERNS = {
    "stepplan_key65": re.compile(r"\b[0-9a-f]{65}\b"),
    "jwt": re.compile(r"\beyJ[A-Za-z0-9_\-]{8,}\.[A-Za-z0-9_\-]{8,}\.[A-Za-z0-9_\-]{8,}"),
    "bearer": re.compile(r"(?i)\bbearer\s+[A-Za-z0-9._\-]{12,}"),
    "ak_sk": re.compile(r"\b(?:ak|sk|rk)-[A-Za-z0-9]{16,}\b"),
    "ghp": re.compile(r"\bghp_[A-Za-z0-9]{20,}\b"),
    "AKIA": re.compile(r"\bAKIA[0-9A-Z]{12,}\b"),
    "pem": re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    "intranet_domain": re.compile(r"\b(?:[a-z0-9-]+\.)+(?:internal|intranet|local|lan|corp|intra)\b"),
    "basic_auth": re.compile(r"://[^/\s:@]+:[^/\s:@]+@"),
    "redaction_marker": re.compile(r"⟪REDACTED:[a-z_]+:[0-9a-f]{6}⟫"),
}

ABBREV = {"claude-code": "cc", "codex": "cx", "cursor": "cu", "gemini": "ge"}


def sha256_file_range(path: str, start: int, end: int) -> str:
    """sha256 of the exact raw lines [start, end] — gate-5 recomputable.
    Accepts either the corpus-relative path or the ~-rooted form the write set
    carries, so the same helper validates both the staged and the rewritten
    pointer."""
    if path.startswith("~"):
        path = os.path.expanduser(path)
    h = hashlib.sha256()
    with open(path, encoding="utf-8") as f:
        for i, line in enumerate(f, 1):
            if i > end:
                break
            if i >= start:
                h.update(line.encode("utf-8"))
    return h.hexdigest()


def load_records() -> list[dict]:
    with open(RECORDS, encoding="utf-8") as f:
        return [json.loads(l) for l in f if l.strip()]


def load_segments() -> dict:
    """(harness, session_id, topic_id) -> segment metadata from S2."""
    with open(TOPICS, encoding="utf-8") as f:
        data = json.load(f)
    out = {}
    for s in data["sessions"]:
        for t in s["topics"]:
            out[(s["session_id"], t["id"])] = t
    return out


LABEL_NOISE = ("", "(no user prompt)", "<turn_aborted>", "please continue",
               "please continue.", "hi")


def _descriptive(harness: str, sid: str, seg_id: str, seg: dict,
                 records: list[dict]) -> str:
    """No usable prompt label: state the span and the record tags instead of
    inventing a theme. A missing label is an S2 limitation, not a topic."""
    ab = ABBREV.get(harness, harness[:2])
    tags: dict[str, int] = {}
    for r in records:
        for t in r.get("tags", []):
            tags[t] = tags.get(t, 0) + 1
    top = ", ".join(sorted(tags, key=lambda k: -tags[k])[:3]) or "no tags"
    turns = seg.get("turns", "?")
    return (f"{ab} {sid[:8]} · {seg_id} — {turns} turns, no usable user-prompt "
            f"label (S2 gap); record tags: {top}")


def segment_title(harness: str, sid: str, seg_id: str, seg: dict,
                  records: list[dict]) -> str:
    """A title for a segment topic, built from the S2 label when that label is
    a real prompt. Continued sessions resume mid-thread, so their first
    "substantial prompt" is a sentence fragment — recognised here and replaced
    by the descriptive form rather than published as a broken title."""
    ab = ABBREV.get(harness, harness[:2])
    label = re.sub(r"\s+", " ", (seg.get("label") or "").strip())
    # leading list markers / stray punctuation left by the label extraction
    label = re.sub(r"^[\-•*\d):\s]+", "", label).strip()
    # tmux pane capture glued onto a prompt collapses to a single marker
    label = re.sub(r"[\u23bf\u2502\u251c\u2514]+\s*", "· ", label)
    label = re.sub(r"(· ){2,}", "· ", label).strip()
    if label.lower() in LABEL_NOISE or label.startswith("<"):
        return _descriptive(harness, sid, seg_id, seg, records)
    # a leading lowercase ASCII letter means we are mid-sentence: fragment
    if label[:1].isascii() and label[:1].islower():
        return _descriptive(harness, sid, seg_id, seg, records)
    if len(label) > 92:
        cut = label[:92].rsplit(" ", 1)[0]
        label = cut.rstrip(" ,;:—-") + " ..."
    return f"{ab} {sid[:8]} · {seg_id} — {label}"


def main() -> int:
    check_only = "--check" in sys.argv
    records = load_records()
    segments = load_segments()

    repairs: list[dict] = []
    dropped: list[dict] = []
    kept: list[dict] = []

    for r in records:
        rid8 = r["record_id"][:8]
        if rid8 in DROP:
            title, why = DROP[rid8]
            dropped.append({"record_id": r["record_id"], "kind": r["kind"],
                            "title": title, "reason": why})
            continue

        # (a) detect the stale idempotency key on the ORIGINAL fields, before
        # anything is touched — otherwise the path normalisation below would
        # make every record look like a repair and hide the real defect.
        if content_digest(r) != r["record_id"]:
            repairs.append({
                "record_id": r["record_id"], "title": r["title"],
                "kind": r["kind"], "field": "record_id",
                "old_range": None, "new_range": None,
                "why": ("stale idempotency key: record_id did not equal the "
                        "content digest of the record's own fields, so a "
                        "re-run would have computed a fresh digest and "
                        "inserted a duplicate"),
            })

        # (b) normalise the L2 path BEFORE any digest is (re)computed, so the
        # idempotency key is a function of exactly the bytes that get written.
        r["l2_file_path"] = "~/Documents/GitHub/AgentHistory/" + r["l2_file_path"]

        if rid8 in REANCHOR:
            new_start, new_end = REANCHOR[rid8]
            old = (r["l2_line_start"], r["l2_line_end"])
            r["l2_line_start"], r["l2_line_end"] = new_start, new_end
            r["l2_body_digest"] = sha256_file_range(
                r["l2_file_path"], new_start, new_end)
            repairs.append({
                "record_id": r["record_id"], "title": r["title"],
                "kind": r["kind"], "field": "l2_line_start/l2_line_end",
                "old_range": list(old), "new_range": [new_start, new_end],
                "new_digest": r["l2_body_digest"],
                "why": ("anchor was numbered against the live session "
                        "transcript, not the archived raw/ copy; re-anchored "
                        "to the raw window that actually carries the claim"),
            })

        # (c) re-key over the final fields, whatever changed above
        r["record_id"] = content_digest(r)

        kept.append(r)

    # ---------------------------------------------------------------- gates
    problems: list[str] = []

    layering = validate_layering(kept)
    if not layering["ok"]:
        problems += ["layering: " + p for p in layering["problems"]]

    line_cache: dict[str, int] = {}

    def nlines(path: str) -> int:
        if path not in line_cache:
            with open(path, encoding="utf-8") as f:
                line_cache[path] = sum(1 for _ in f)
        return line_cache[path]

    unanchored = 0
    for r in kept:
        p = os.path.expanduser(r["l2_file_path"])
        if not os.path.exists(p):
            problems.append(f"{r['record_id'][:8]}: L2 file missing {r['l2_file_path']}")
            unanchored += 1
            continue
        n = nlines(p)
        if not (1 <= r["l2_line_start"] <= r["l2_line_end"] <= n):
            problems.append(
                f"{r['record_id'][:8]}: L2 {r['l2_line_start']}-{r['l2_line_end']} "
                f"outside {r['l2_file_path']} ({n} lines)")
            unanchored += 1
            continue
        if sha256_file_range(p, r["l2_line_start"], r["l2_line_end"]) != r["l2_body_digest"]:
            problems.append(f"{r['record_id'][:8]}: L2 digest does not recompute")
            unanchored += 1
        if content_digest(r) != r["record_id"]:
            problems.append(f"{r['record_id'][:8]}: record_id != content digest")

    # redaction gate on the text that is about to be stored
    secret_hits: dict[str, list] = {}
    for r in kept:
        blob = "\n".join([r.get("title", ""), r.get("l0_summary", ""),
                          r.get("l1_overview", "")])
        for name, pat in SECRET_PATTERNS.items():
            for m in pat.finditer(blob):
                secret_hits.setdefault(name, []).append(
                    {"record_id": r["record_id"][:8], "match": m.group(0)[:48]})
    for name, hits in secret_hits.items():
        problems.append(f"redaction: {len(hits)} {name} hit(s) in stored text")

    # ------------------------------------------------------- topic tree
    pairs = sorted({(r["harness"], r["session_id"], r["topic_id"]) for r in kept})
    topic_plan: list[dict] = []
    seg_of: dict[tuple, str] = {}
    for harness, sid, seg_id in pairs:
        ab = ABBREV.get(harness, harness[:2])
        short = sid[:8]
        seg_topic = f"wash-seg-{ab}-{short}-{seg_id}"
        seg_of[(harness, sid, seg_id)] = seg_topic
        seg = segments.get((sid, seg_id), {})
        recs = [r for r in kept if r["harness"] == harness
                and r["session_id"] == sid and r["topic_id"] == seg_id]
        topic_plan.append({
            "topic_id": seg_topic,
            "parent_topic_id": f"wash-sess-{ab}-{short}",
            "title": segment_title(harness, sid, seg_id, seg, recs),
            "records": len(recs),
        })

    sessions = sorted({(h, s) for h, s, _ in pairs})
    session_plan = []
    for harness, sid in sessions:
        ab = ABBREV.get(harness, harness[:2])
        short = sid[:8]
        n_seg = sum(1 for h, s, _ in pairs if s == sid)
        n_rec = sum(1 for r in kept if r["session_id"] == sid)
        session_plan.append({
            "topic_id": f"wash-sess-{ab}-{short}",
            "parent_topic_id": "wash-root",
            "title": (f"{harness} session {sid} — {n_seg} S2 segment(s) "
                      f"contributing, {n_rec} distilled records"),
            "records": n_rec,
        })

    root = {
        "topic_id": "wash-root",
        "parent_topic_id": None,
        "title": ("SeatLoom agent-history wash — 16 sessions, "
                  "2026-09-24 to 2026-10-10, distilled 2026-10-11"),
        "records": len(kept),
    }

    # ------------------------------------------------------ write manifest
    mem_plan = []
    for r in sorted(kept, key=lambda r: (r["session_id"], r["record_id"])):
        key = (r["harness"], r["session_id"], r["topic_id"])
        mem_plan.append({
            "record_id": r["record_id"],
            "kind": r["kind"],
            "confidence": r["confidence"],
            "title": r["title"],
            "l0_summary": r["l0_summary"],
            "l1_overview": r["l1_overview"],
            "topic_id": seg_of[key],
            "source_kind": r["source_kind"],
            "source_ref": r["source_ref"],
            "l2_ref_kind": "file",
            # raw/ is the corpus of record and lives OUTSIDE this repo, at the
            # AgentHistory root. 015/018 say "repo-relative"; a bare
            # raw/claude-code/... path would look repo-relative while resolving
            # to nothing here, i.e. an unresolvable pointer — which red line 2
            # forbids. The ~-rooted form names the corpus root and resolves.
            "l2_file_path": r["l2_file_path"],
            "l2_body_digest": r["l2_body_digest"],
            "l2_line_start": r["l2_line_start"],
            "l2_line_end": r["l2_line_end"],
            "tags": r["tags"],
        })

    manifest = {
        "generated_by": "tools/wash/s5_writein.py",
        "project_id": "seatloom",
        "raw_root": AH,
        "counts": {
            "staged": len(records),
            "inserted": len(mem_plan),
            "dropped_no_line_range": len(dropped),
            "repaired": len(repairs),
            "topics": 1 + len(session_plan) + len(topic_plan),
        },
        "topics": [root] + session_plan + topic_plan,
        "memories": mem_plan,
        "repairs": repairs,
        "dropped": dropped,
        "secret_hits": secret_hits,
        "validation": {"problems": problems, "ok": not problems,
                       "layering": layering},
    }

    # ------------------------------------------------------------- report
    print(f"staged records            : {len(records)}")
    print(f"to insert                 : {len(mem_plan)}")
    print(f"dropped (no line range)   : {len(dropped)}")
    print(f"repairs applied           : {len(repairs)}")
    print(f"topics to open            : {manifest['counts']['topics']}"
          f" (1 root + {len(session_plan)} session + {len(topic_plan)} segment)")
    print(f"validation problems       : {len(problems)}")
    for p in problems:
        print("   !", p)
    for d in dropped:
        print(f"   - dropped {d['record_id'][:8]} {d['kind']}: {d['title']}")
    for rp in repairs:
        print(f"   ~ repaired {rp['record_id'][:8]} {rp['kind']}: {rp['title'][:60]}")

    if problems:
        print("\nREFUSING to emit a manifest: the write set is not clean.")
        return 1

    if not check_only:
        os.makedirs(OUT_DIR, exist_ok=True)
        with open(MANIFEST, "w", encoding="utf-8") as f:
            json.dump(manifest, f, ensure_ascii=False, indent=1)
        staged_out = os.path.join(OUT_DIR, "records.writein.jsonl")
        with open(staged_out, "w", encoding="utf-8") as f:
            for r in sorted(kept, key=lambda r: (r["session_id"], r["record_id"])):
                f.write(json.dumps(r, ensure_ascii=False) + "\n")
        print(f"\nmanifest -> {MANIFEST}")
        print(f"staged   -> {staged_out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
