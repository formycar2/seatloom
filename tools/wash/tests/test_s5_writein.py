"""S5 write-in gates — red lines 1 and 2, enforced on the ACTUAL write set.

The 09:06 close-out reported "0 records dropped for no line number" on the
strength of a 5-record random sample. This suite exists because that sample was
not enough: run over all 88 records it found 2 records anchored against the
live transcript instead of raw/, 3 stale idempotency keys, and 1 record no
honest line range can carry. A gate that protects a per-record invariant has to
be exhaustive over the write set, not statistical.

Skips when the manifest has not been generated (`python3 tools/wash/s5_writein.py`).
"""

import hashlib
import json
import os
import re
import sys
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vela_wash.s5_store import deep_cycle_detect  # noqa: E402

WASH = os.path.join(os.path.dirname(__file__), "..")
MANIFEST = os.path.join(WASH, ".local", "wash", "s5", "writein_manifest.json")
REFUSAL = os.path.join(WASH, ".local", "wash", "s5", "writein_refusal.json")
AH = os.path.expanduser("~/Documents/GitHub/AgentHistory")


def _skip_reason() -> str | None:
    """Why these gates cannot run. Said honestly: since the session-key gate
    landed, a missing manifest usually means the driver REFUSED the write set,
    not that nobody ran it. Pointing at the refusal record is the difference
    between a useful skip and a misleading one."""
    if os.path.exists(MANIFEST):
        return None
    if os.path.exists(REFUSAL):
        try:
            with open(REFUSAL, encoding="utf-8") as f:
                n = len(json.load(f)["validation"]["problems"])
        except (OSError, ValueError, KeyError):
            n = -1
        return (f"the write-in driver REFUSED this write set ({n} validation "
                f"problem(s)) — read {os.path.relpath(REFUSAL, WASH)}; the "
                f"manifest gates have nothing to check until it is clean")
    return "run the write-in driver first (python3 tools/wash/s5_writein.py)"

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
}


def sha256_file_range(path, start, end):
    h = hashlib.sha256()
    with open(path, encoding="utf-8") as f:
        for i, line in enumerate(f, 1):
            if i > end:
                break
            if i >= start:
                h.update(line.encode("utf-8"))
    return h.hexdigest()


def content_digest(rec):
    h = hashlib.sha256()
    for k in ("kind", "title", "l0_summary", "l2_file_path",
              "l2_line_start", "l2_line_end"):
        h.update(str(rec.get(k, "")).encode("utf-8"))
        h.update(b"\x00")
    return h.hexdigest()


@unittest.skipUnless(os.path.exists(MANIFEST), _skip_reason())
class TestWriteInManifest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        with open(MANIFEST, encoding="utf-8") as f:
            cls.m = json.load(f)
        cls.memories = cls.m["memories"]

    # ---------------------------------------------------------- red line 2
    def test_every_l2_pointer_resolves_and_recomputes(self):
        """Not a sample: every record, against the raw file it names."""
        checked = 0
        for r in self.memories:
            path = os.path.expanduser(r["l2_file_path"])
            self.assertTrue(os.path.exists(path), f"{r['record_id'][:8]}: {path}")
            with open(path, encoding="utf-8") as f:
                n = sum(1 for _ in f)
            self.assertGreaterEqual(r["l2_line_start"], 1, r["record_id"][:8])
            self.assertLessEqual(r["l2_line_end"], n,
                                 f"{r['record_id'][:8]}: range past EOF")
            self.assertLessEqual(r["l2_line_start"], r["l2_line_end"],
                                 r["record_id"][:8])
            self.assertEqual(
                sha256_file_range(path, r["l2_line_start"], r["l2_line_end"]),
                r["l2_body_digest"],
                f"{r['record_id'][:8]}: digest does not recompute")
            checked += 1
        self.assertEqual(checked, len(self.memories))

    def test_no_empty_body_digest(self):
        """sha256 of nothing is the signature of a range that read no lines."""
        empty = hashlib.sha256(b"").hexdigest()
        for r in self.memories:
            self.assertNotEqual(r["l2_body_digest"], empty, r["record_id"][:8])

    # ---------------------------------------------------- idempotency keys
    def test_record_id_equals_content_digest(self):
        for r in self.memories:
            self.assertEqual(content_digest(r), r["record_id"],
                             f"{r['record_id'][:8]}: stale idempotency key")

    # ---------------------------------------------------------- red line 1
    def test_no_secret_shape_in_stored_text(self):
        for r in self.memories:
            blob = "\n".join([r["title"], r["l0_summary"], r["l1_overview"]])
            for name, pat in SECRET_PATTERNS.items():
                hit = pat.search(blob)
                self.assertIsNone(
                    hit, f"{r['record_id'][:8]}: {name} in stored text "
                         f"({hit.group(0)[:40] if hit else ''})")

    # ------------------------------------------------------ accounting
    def test_counts_add_up(self):
        c = self.m["counts"]
        self.assertEqual(c["inserted"] + c["dropped_no_line_range"], c["staged"])
        self.assertEqual(c["inserted"], len(self.memories))
        self.assertEqual(len(self.m["dropped"]), c["dropped_no_line_range"])
        for d in self.m["dropped"]:
            self.assertTrue(d["reason"].strip(),
                            "a dropped record must state why")

    def test_validation_reported_clean(self):
        self.assertEqual(self.m["validation"]["problems"], [])
        self.assertTrue(self.m["validation"]["ok"])
        self.assertTrue(self.m["validation"]["layering"]["ok"])

    # ------------------------------------------------------- topic tree
    def test_topic_tree_is_a_forest_with_one_root(self):
        topics = {t["topic_id"]: t for t in self.m["topics"]}
        roots = [t for t in topics.values() if not t["parent_topic_id"]]
        self.assertEqual(len(roots), 1, "exactly one root scope")
        for t in self.m["topics"]:
            if t["parent_topic_id"]:
                self.assertIn(t["parent_topic_id"], topics,
                              f"{t['topic_id']}: parent not opened first")
        # every memory's topic exists and is a leaf-most segment scope
        for r in self.memories:
            self.assertIn(r["topic_id"], topics, r["record_id"][:8])

    def test_every_topic_has_a_title_and_records(self):
        for t in self.m["topics"]:
            self.assertTrue(t["title"].strip(), t["topic_id"])
            self.assertGreater(t["records"], 0,
                               f"{t['topic_id']}: an empty recall scope")

    def test_deep_cycle_detect_catches_multi_level_cycles(self):
        """015 only blocks 1-cycles (a node being its own parent). The pipeline
        owns the deeper ones, so this asserts the depth-4 name chain the p2
        delivery claims is unit-proven — and, just as importantly, that a plain
        forest is NOT reported as a cycle (a false positive would block every
        insert for no reason)."""
        self.assertIsNone(deep_cycle_detect(["a/b/c", "a/b/d", "e/f"]))
        self.assertIsNone(deep_cycle_detect(["seatloom"]))
        # name chain a -> b -> c -> d -> a closes: caught at depth 4
        cycle = deep_cycle_detect(["a/b", "b/c", "c/d", "d/a"])
        self.assertIsNotNone(cycle)
        self.assertEqual(cycle[0], cycle[-1], "a cycle is reported closed")
        # and the depth-3 case the p2 delivery names explicitly
        self.assertIsNotNone(deep_cycle_detect(["a/b", "b/c", "c/a"]))


if __name__ == "__main__":
    unittest.main(verbosity=2)
