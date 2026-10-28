"""S2 segmentation tests — synthetic streams with known ground truth.

S2 is the self-built weakest link; these tests pin the DOCUMENTED behavior
(signals, noise filtering, two-level tree, micro-exchange merging) so a future
change cannot silently alter the topic shape.
"""

import os
import sys
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vela_wash.s2_segment import segment_session  # noqa: E402


def turn(seq, ts=None, role="user", kind="message", text="hi", tool=None,
         cwd="/w", is_meta=False):
    t = {"seq": seq, "ts": ts, "role": role, "kind": kind, "text": text,
         "tool": tool, "refs": [], "cwd": cwd, "src_file": "raw/x.jsonl",
         "src_line": seq + 1, "meta": {}}
    if is_meta:
        t["kind"] = "meta"
        t["text"] = None
    return t


class TestSegmentation(unittest.TestCase):
    def test_silence_splits_topics(self):
        turns = [
            turn(0, "2026-10-10T00:00:00Z"),
            turn(1, "2026-10-10T00:10:00Z", role="assistant"),
            turn(2, "2026-10-10T01:00:00Z"),          # >30min silence
            turn(3, "2026-10-10T01:05:00Z", role="assistant"),
        ]
        topics = segment_session(turns)
        self.assertEqual(len(topics), 2)
        self.assertIn("silence>30min", topics[1]["signals"])

    def test_cwd_change_splits_topics(self):
        turns = [
            turn(0, "2026-10-10T00:00:00Z", cwd="/a"),
            turn(1, "2026-10-10T00:01:00Z", role="assistant", cwd="/a"),
            turn(2, "2026-10-10T00:02:00Z", cwd="/b"),  # cwd change
            turn(3, "2026-10-10T00:03:00Z", role="assistant", cwd="/b"),
        ]
        topics = segment_session(turns)
        self.assertEqual(len(topics), 2)
        self.assertIn("cwd-change", topics[1]["signals"])

    def test_ui_noise_never_splits_or_labels(self):
        turns = [
            turn(0, "2026-10-10T00:00:00Z"),
            turn(1, "2026-10-10T00:01:00Z", role="assistant"),
            turn(2, "2026-10-10T00:02:00Z", text="<local-command-stdout>Bye!</local-command-stdout>"),
            turn(3, "2026-10-10T00:03:00Z", text="[Request interrupted by user]"),
            turn(4, "2026-10-10T00:04:00Z", role="assistant"),
        ]
        topics = segment_session(turns)
        self.assertEqual(len(topics), 1)               # noise did not split
        self.assertEqual(len(topics[0]["exchanges"]), 1)

    def test_substantial_prompts_split_exchanges(self):
        turns = [turn(0, "2026-10-10T00:00:00Z", text="first task")]
        for i in range(1, 4):
            turns.append(turn(i, f"2026-10-10T00:0{i}:00Z", role="assistant"))
        turns.append(turn(4, "2026-10-10T00:04:00Z", text="second task"))
        turns.append(turn(5, "2026-10-10T00:05:00Z", role="assistant"))
        topics = segment_session(turns)
        self.assertEqual(len(topics), 1)
        self.assertEqual(len(topics[0]["exchanges"]), 2)
        self.assertEqual(topics[0]["exchanges"][0]["label"], "first task")
        self.assertEqual(topics[0]["exchanges"][1]["label"], "second task")

    def test_micro_exchanges_merge(self):
        turns = [turn(0, "2026-10-10T00:00:00Z", text="real task")]
        turns.append(turn(1, "2026-10-10T00:01:00Z", role="assistant"))
        turns.append(turn(2, "2026-10-10T00:02:00Z", text="[Request interrupted by user]"))
        turns.append(turn(3, "2026-10-10T00:03:00Z", text="another real task"))
        turns.append(turn(4, "2026-10-10T00:04:00Z", role="assistant"))
        topics = segment_session(turns)
        self.assertEqual(len(topics[0]["exchanges"]), 2)

    def test_task_marker_splits_topics(self):
        mk = turn(5, "2026-10-10T00:05:00Z", is_meta=True)
        mk["meta"]["raw_type"] = "event_msg/task_started"
        turns = [
            turn(0, "2026-10-10T00:00:00Z"),
            turn(1, "2026-10-10T00:01:00Z", role="assistant"),
            mk,
            turn(6, "2026-10-10T00:06:00Z"),
        ]
        topics = segment_session(turns)
        self.assertEqual(len(topics), 2)
        self.assertIn("task-marker", topics[1]["signals"])

    def test_ids_are_unique_and_dense(self):
        turns = []
        ts = 0
        for k in range(6):
            turns.append(turn(len(turns), f"2026-10-10T0{ts}:00:00Z", text=f"task {k}"))
            turns.append(turn(len(turns), f"2026-10-10T0{ts}:30:00Z", role="assistant"))
            ts += 1
        topics = segment_session(turns)
        ids = [t["id"] for t in topics]
        self.assertEqual(ids, [f"t{i:03d}" for i in range(len(topics))])
        for tp in topics:
            ex_ids = [e["id"] for e in tp["exchanges"]]
            self.assertEqual(ex_ids, [f"e{i:03d}" for i in range(len(ex_ids))])


if __name__ == "__main__":
    unittest.main(verbosity=2)
