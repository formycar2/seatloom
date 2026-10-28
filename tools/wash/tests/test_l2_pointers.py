"""L2 pointer integrity — red line #2 (AGENTS.md): a record whose L2 cannot
point back to concrete raw-file lines must not enter the store.

For Part 1 the requirement is on the canonical stream itself: every turn's
(src_file, src_line) must resolve to the exact raw row it was derived from.
Verification is STRUCTURAL (re-read the raw file, match ids/shape), not
literal-substring, because raw jsonl escapes newlines.
"""

import json
import os
import random
import sys
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vela_wash import manifest  # noqa: E402

WASH_DIR = os.path.join(os.path.dirname(__file__), "..", ".local", "wash")
AGenthistory = os.path.expanduser("~/Documents/GitHub/AgentHistory")


def _raw_rows(rel_src_file: str) -> list[str]:
    with open(os.path.join(AGenthistory, rel_src_file), encoding="utf-8") as f:
        return f.readlines()


def _row_at(rel_src_file: str, lineno: int) -> dict:
    return json.loads(_raw_rows(rel_src_file)[lineno - 1])


class TestL2Pointers(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if not os.path.isdir(WASH_DIR):
            raise unittest.SkipTest("run the pipeline first (.local/wash missing)")
        cls.sessions = []
        for harness, sid, relpath, _note in manifest.SESSIONS:
            p = os.path.join(WASH_DIR, f"{harness}__{sid}", "s1.redacted.jsonl")
            if os.path.exists(p):
                with open(p, encoding="utf-8") as f:
                    cls.sessions.append((harness, sid, [json.loads(l) for l in f]))

    def test_provenance_fields_present_on_every_turn(self):
        for harness, sid, turns in self.sessions:
            for t in turns:
                self.assertIn("src_file", t, f"{sid}")
                self.assertIsInstance(t["src_line"], int, f"{sid}")
                self.assertGreaterEqual(t["src_line"], 0, f"{sid}")

    def test_sampled_pointers_resolve_structurally(self):
        rng = random.Random(20261010)
        checked = {"message": 0, "tool_call": 0, "tool_result": 0, "reasoning": 0}
        for harness, sid, turns in self.sessions:
            real = [t for t in turns if t.get("kind") != "meta" and t.get("text")]
            if not real:
                continue
            for t in rng.sample(real, min(15, len(real))):
                if harness == "cursor":
                    # cursor src_line is a 1-based logical index: prompts
                    # first, then generations (see adapt_cursor)
                    dump = json.load(open(os.path.join(
                        AGenthistory, t["src_file"]), encoding="utf-8"))
                    n_prompts = len(dump.get("prompts") or [])
                    idx = t["src_line"] - 1
                    if idx < n_prompts:
                        self.assertEqual(t["role"], "user")
                        self.assertEqual(dump["prompts"][idx]["text"], t["text"])
                    else:
                        self.assertEqual(t["role"], "assistant")
                        self.assertEqual(
                            dump["generations"][idx - n_prompts]["textDescription"],
                            t["text"])
                    checked["message"] += 1
                    continue

                row = _row_at(t["src_file"], t["src_line"])
                kind = t["kind"]

                if harness == "codex":
                    self.assertEqual(row.get("type"), "response_item"
                                     if kind in ("message", "tool_call", "tool_result")
                                     else "event_msg")
                    payload = row.get("payload") or {}
                    if kind == "tool_call":
                        self.assertEqual(payload.get("name"), t["tool"])
                        self.assertEqual(payload.get("call_id"),
                                         (t.get("meta") or {}).get("call_id"))
                    elif kind == "tool_result":
                        self.assertIn(payload.get("type"),
                                      ("function_call_output", "custom_tool_call_output"))
                        self.assertEqual(payload.get("call_id"),
                                         (t.get("meta") or {}).get("call_id"))
                    elif kind == "reasoning":
                        self.assertEqual(payload.get("type"), "agent_reasoning")
                    else:
                        self.assertEqual(payload.get("type"), "message")
                        # codex developer instructions map to role=system by
                        # design (see adapt_codex)
                        self.assertEqual(payload.get("role"),
                                         {"user": "user", "assistant": "assistant",
                                          "system": "developer"}[t["role"]])
                    checked[kind] += 1
                    continue

                # claude-code
                if kind == "tool_result":
                    blocks = (row.get("message") or {}).get("content") or []
                    ids = [b.get("tool_use_id") for b in blocks
                           if isinstance(b, dict) and b.get("type") == "tool_result"]
                    self.assertIn((t.get("meta") or {}).get("tool_use_id"), ids,
                                  f"{sid}: tool_result pointer mismatch at "
                                  f"{t['src_file']}:{t['src_line']}")
                elif kind == "tool_call":
                    blocks = (row.get("message") or {}).get("content") or []
                    uses = [b for b in blocks
                            if isinstance(b, dict) and b.get("type") == "tool_use"]
                    self.assertTrue(uses, f"{sid}: no tool_use at pointer")
                    self.assertEqual(uses[0].get("name"), t["tool"])
                    self.assertEqual(uses[0].get("id"),
                                     (t.get("meta") or {}).get("tool_use_id"))
                elif kind == "reasoning":
                    blocks = (row.get("message") or {}).get("content") or []
                    self.assertTrue(any(isinstance(b, dict) and b.get("type") == "thinking"
                                        for b in blocks),
                                    f"{sid}: no thinking block at pointer")
                else:
                    self.assertEqual(row.get("type"), "user" if t["role"] == "user"
                                     else "assistant")
                checked[kind] += 1

        for kind, n in checked.items():
            self.assertGreater(n, 0, f"no {kind} turns sampled")
        self.assertGreater(sum(checked.values()), 150,
                           "sanity: expected >150 structurally verified pointers")

    def test_gemini_registered_only(self):
        for harness, sid, turns in self.sessions:
            if harness != "gemini":
                continue
            self.assertEqual(len(turns), 1)
            self.assertEqual(turns[0]["kind"], "meta")
            self.assertEqual(turns[0]["meta"]["harness"], "gemini")
            self.assertEqual(turns[0]["meta"]["session_id"], "seatloom")


if __name__ == "__main__":
    unittest.main(verbosity=2)
