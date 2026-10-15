"""S0 adapter tests — tolerance, stats honesty, and canonical shape.

Runs against the REAL seatloom raw files (read-only) plus synthetic rows for
the tolerance paths (unknown row shapes must never fail a file).
"""

import json
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vela_wash import manifest  # noqa: E402
from vela_wash.s0_normalize import (adapt, adapt_claude_code, adapt_codex,  # noqa: E402
                                    adapt_cursor, adapt_gemini)


def _consume(gen):
    return list(gen)


class TestAdaptersOnRealSessions(unittest.TestCase):
    def _run(self, harness, sid, relpath):
        path = manifest.session_path(relpath)
        if not os.path.exists(path):
            self.skipTest(f"raw not present: {path}")
        gen, stats = adapt(harness, path, manifest.session_rel(relpath), sid)
        turns = _consume(gen)
        return turns, stats

    def test_every_manifest_session_adapts(self):
        for harness, sid, relpath, _note in manifest.SESSIONS:
            with self.subTest(session=sid):
                turns, stats = self._run(harness, sid, relpath)
                # stats honesty: parsed + meta == canonical turns; unknown and
                # unparseable are counted, never fatal
                self.assertEqual(stats["parsed"] + stats["meta_lines"],
                                 len(turns), f"{sid}: stats do not balance")
                if harness == "gemini":
                    self.assertTrue(stats.get("registered_only"))
                    self.assertEqual(stats["parsed"], 0)
                else:
                    self.assertGreater(stats["parsed"], 0, f"{sid}: nothing parsed")

    def test_total_lines_match_wc_l(self):
        for harness, sid, relpath, _note in manifest.SESSIONS:
            path = manifest.session_path(relpath)
            if not os.path.exists(path) or harness in ("cursor", "gemini"):
                continue  # cursor is a json dump; gemini is register-only
            with self.subTest(session=sid), open(path, encoding="utf-8") as f:
                wc = sum(1 for _ in f)
            _t, stats = self._run(harness, sid, relpath)
            self.assertEqual(stats["total_lines"], wc, f"{sid}: line count mismatch")

    def test_canonical_shape(self):
        turns, _ = self._run("claude-code", "b0100cff-2426-4786-b165-45827c9e2713",
                             "claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl")
        required = {"seq", "ts", "role", "kind", "text", "tool", "refs",
                    "cwd", "src_file", "src_line"}
        for t in turns[:200]:
            self.assertTrue(required <= set(t), f"missing fields: {required - set(t)}")
            self.assertIn(t["role"], ("user", "assistant", "system"))
            self.assertIn(t["kind"], ("message", "tool_call", "tool_result",
                                      "reasoning", "event", "meta"))
        # seq is dense and ordered; src_line points back into the raw file
        self.assertEqual([t["seq"] for t in turns], list(range(len(turns))))
        for t in turns[:200]:
            self.assertGreaterEqual(t["src_line"], 1)

    def test_codex_tool_results_resolve_tool_names(self):
        turns, _ = self._run("codex", "019ddc3e-06d0-73c3-a80e-0d49f38ffe80",
                             "codex/rollout-2026-04-30T10-35-45-019ddc3e-06d0-73c3-a80e-0d49f38ffe80.jsonl")
        results = [t for t in turns if t["kind"] == "tool_result"]
        self.assertTrue(results)
        self.assertTrue(all(r["tool"] != "unknown_tool" for r in results),
                        "unresolved call_id -> unknown_tool")

    def test_claude_code_tool_results_resolve_tool_names(self):
        turns, _ = self._run("claude-code", "b0100cff-2426-4786-b165-45827c9e2713",
                             "claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl")
        results = [t for t in turns if t["kind"] == "tool_result"]
        self.assertTrue(results)
        unresolved = [r for r in results if r["tool"] == "unknown_tool"]
        self.assertLess(len(unresolved), len(results) * 0.01,
                        "too many unresolved tool_use_ids")


class TestTolerance(unittest.TestCase):
    """Unknown/malformed rows -> meta + counter, never a failed file."""

    def test_claude_code_tolerates_unknown_rows(self):
        rows = [
            {"type": "assistant", "timestamp": "2026-10-10T00:00:00Z",
             "message": {"role": "assistant", "content": [{"type": "text", "text": "hi"}]}},
            {"type": "totally-new-future-row", "payload": {"x": 1}},
            {"type": "user", "message": {"role": "user", "content": 42}},   # bad shape
            "this is not even json",
            {"type": "attachment", "attachment": {"type": "brand_new_attachment"}},
        ]
        with tempfile.NamedTemporaryFile("w", suffix=".jsonl", delete=False) as f:
            for r in rows:
                f.write(r if isinstance(r, str) else json.dumps(r))
                f.write("\n")
            path = f.name
        try:
            gen, stats = adapt_claude_code(path, "raw/test.jsonl")
            turns = _consume(gen)
        finally:
            os.unlink(path)
        self.assertEqual(stats["total_lines"], 5)
        self.assertEqual(stats["parsed"], 1)
        self.assertGreaterEqual(stats["unknown_rows"], 2)
        self.assertEqual(stats["unparseable_rows"], 1)
        self.assertEqual(stats["parsed"] + stats["meta_lines"], len(turns))

    def test_codex_tolerates_unknown_rows(self):
        rows = [
            {"type": "session_meta", "payload": {"id": "s1", "cwd": "/w"}},
            {"type": "response_item", "payload": {"type": "message", "role": "assistant",
                                                  "content": [{"type": "output_text", "text": "yo"}]}},
            {"type": "response_item", "payload": {"type": "future_thing"}},
            {"type": "brand_new_top_level", "payload": {}},
            "{broken json",
        ]
        with tempfile.NamedTemporaryFile("w", suffix=".jsonl", delete=False) as f:
            for r in rows:
                f.write(r if isinstance(r, str) else json.dumps(r))
                f.write("\n")
            path = f.name
        try:
            gen, stats = adapt_codex(path, "raw/test.jsonl")
            turns = _consume(gen)
        finally:
            os.unlink(path)
        self.assertEqual(stats["total_lines"], 5)
        self.assertEqual(stats["parsed"], 1)
        self.assertEqual(stats["unknown_rows"], 2)
        self.assertEqual(stats["unparseable_rows"], 1)
        self.assertEqual(stats["parsed"] + stats["meta_lines"], len(turns))

    def test_gemini_is_register_only(self):
        gen, stats = adapt_gemini("/nonexistent/path", "raw/gemini/seatloom", "seatloom")
        turns = _consume(gen)
        self.assertEqual(len(turns), 1)
        self.assertEqual(turns[0]["kind"], "meta")
        self.assertEqual(turns[0]["meta"]["harness"], "gemini")
        self.assertTrue(stats["registered_only"])
        self.assertEqual(stats["parsed"], 0)

    def test_cursor_reads_existing_dump(self):
        dump = {"prompts": [{"text": "hi", "commandType": 4}],
                "generations": [{"unixMs": 1778224775460, "type": "composer",
                                 "textDescription": "hello"},
                                {"unixMs": 1778224825957, "type": "composer",
                                 "textDescription": ""}]}
        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as f:
            json.dump(dump, f)
            path = f.name
        try:
            gen, stats = adapt_cursor(path, "raw/cursor/chat_dump.json")
            turns = _consume(gen)
        finally:
            os.unlink(path)
        self.assertEqual(stats["total_lines"], 3)
        self.assertEqual(stats["parsed"], 2)
        self.assertEqual(stats["meta_lines"], 1)
        self.assertEqual(turns[0]["role"], "user")
        self.assertEqual(turns[1]["role"], "assistant")
        self.assertTrue(turns[1]["ts"])  # unixMs converted to ISO ts


if __name__ == "__main__":
    unittest.main(verbosity=2)
