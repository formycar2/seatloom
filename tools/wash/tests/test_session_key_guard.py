"""The session-key gate — both spellings, exhaustively, or the manifest is refused.

Why this suite exists as its own module: the L2 gates in
`test_s5_writein.py` check the *line anchors*, and they pass perfectly for a
record whose `source_ref` dangles. Nothing else in the driver looked at the
session key, so a memory went live pointing at a session that resolves against
no `archived_sessions` row — and the session then looks like it produced
nothing. That is the defect this gate closes, so it gets its own proof:

  * unit level — A1/A2 parsing, resolution, and every refusal reason;
  * end-to-end — the driver itself, on a synthetic write set, refusing a
    non-resolving reference and emitting a manifest when every key resolves.

The write set used here is synthetic and built in a temp dir (`--records`,
`--topics`, `--out`, `--index` exist for exactly this), so the suite does not
depend on the staged scratch records and does not need the live table.
"""

import hashlib
import json
import os
import subprocess
import sys
import tempfile
import unittest
from datetime import datetime, timedelta, timezone

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vela_wash.s3_extract import content_digest  # noqa: E402
from vela_wash.session_key import (  # noqa: E402
    INDEX_SCHEMA,
    IndexError_,
    SessionIndex,
    a1_parse,
    a2_parse,
    corpus_relative,
    validate_session_keys,
    write_index,
)

WASH = os.path.join(os.path.dirname(__file__), "..")
DRIVER = os.path.join(WASH, "s5_writein.py")
AH = os.path.expanduser("~/Documents/GitHub/AgentHistory")
RAW = os.path.join(AH, "raw", "claude-code", "b0100cff-2426-4786-b165-45827c9e2713.jsonl")

SESSION_ID = "b0100cff-2426-4786-b165-45827c9e2713"
REL_RAW = f"raw/claude-code/{SESSION_ID}.jsonl"
# The gate runs on the write set AFTER the driver has normalised l2_file_path
# to the ~-rooted corpus path (that normalisation happens before any digest is
# computed), so that is the form the records carry here.
ABS_RAW = f"~/Documents/GitHub/AgentHistory/{REL_RAW}"


def _index_rows(**over):
    row = {"harness": "claude-code", "id": SESSION_ID, "raw_path": REL_RAW,
           "project": "seatloom", "corpus_loaded": True}
    row.update(over)
    return [row]


def _record(**over):
    """A staged record whose L2 anchor is real, so the ONLY thing under test is
    the session key."""
    rec = {
        "kind": "decision",
        "confidence": "quoted",
        "title": "SeatLoom is local-first by definition, not by phase",
        "l0_summary": "SeatLoom must run on the machine where the agents live.",
        "l1_overview": "pty/mod.rs shells out to tmux directly and therefore "
                       "only sees tmux on the backend host.",
        "session_id": SESSION_ID,
        "harness": "claude-code",
        "topic_id": "t000",
        "source_kind": "archived_session",
        "source_ref": f"claude-code:{SESSION_ID}",
        "l2_ref_kind": "file",
        "l2_file_path": ABS_RAW,
        "l2_line_start": 1,
        "l2_line_end": 3,
        "l2_body_digest": "0" * 64,
        "tags": ["architecture"],
    }
    rec.update(over)
    return rec


class TestSpellings(unittest.TestCase):
    """A1 and A2, and what each one can and cannot see."""

    def test_a1_accepts_the_contract_spelling(self):
        self.assertEqual(a1_parse(f"claude-code:{SESSION_ID}"),
                         ("claude-code", SESSION_ID))
        self.assertEqual(a1_parse("codex:019dce05-fb23-7081-aaa6-40d60b4b33d2"),
                         ("codex", "019dce05-fb23-7081-aaa6-40d60b4b33d2"))
        self.assertEqual(a1_parse("cursor:seatloom__d73725d80aa3442b0d73b0e4a7911be0"),
                         ("cursor", "seatloom__d73725d80aa3442b0d73b0e4a7911be0"))

    def test_a1_rejects_anything_that_is_not_the_spelling(self):
        for bad in ["", "claude-code", "claude-code:", ":abc", "a:b:c",
                    "notaharness:xyz", f"claude-code:{SESSION_ID} ", None]:
            self.assertIsNone(a1_parse(bad), bad)

    def test_a2_codex_must_strip_the_rollout_timestamp(self):
        """The codex filename is rollout-<ts>-<uuid>. A path-derived id that
        keeps the timestamp is the pre-021 spelling, and it is not the session's
        own id — this is the trap A1 exists to avoid."""
        parsed = a2_parse("raw/codex/rollout-2026-04-27T16-19-51-"
                          "019dce05-fb23-7081-aaa6-40d60b4b33d2.jsonl")
        self.assertEqual(parsed["harness"], "codex")
        self.assertEqual(parsed["native_id"], "019dce05-fb23-7081-aaa6-40d60b4b33d2")
        self.assertEqual(parsed["stem"],
                         "rollout-2026-04-27T16-19-51-019dce05-fb23-7081-aaa6-40d60b4b33d2")
        self.assertEqual(parsed["shape"], "rollout-<ts>-<uuid>")

    def test_a2_plain_and_directory_shapes(self):
        p = a2_parse(REL_RAW)
        self.assertEqual((p["harness"], p["native_id"], p["stem"]),
                         ("claude-code", SESSION_ID, SESSION_ID))
        # cursor/gemini archive a DIRECTORY: the stem IS the id, no extension
        # to strip, and sha256 is NULL for dir-copies (011).
        c = a2_parse("raw/cursor/seatloom__d73725d80aa3442b0d73b0e4a7911be0")
        self.assertEqual((c["harness"], c["native_id"]),
                         ("cursor", "seatloom__d73725d80aa3442b0d73b0e4a7911be0"))
        g = a2_parse("raw/gemini/seatloom")
        self.assertEqual((g["harness"], g["native_id"]), ("gemini", "seatloom"))
        # the cursor row's raw_path points at the chat_dump INSIDE the archived
        # directory. Its last component's stem is 'chat_dump' — a FILE, not a
        # session — which is exactly why the gate resolves A2 by matching
        # raw_path and never by an id derived from the path.
        n = a2_parse("raw/cursor/seatloom__d73725d80aa3442b0d73b0e4a7911be0/chat_dump.json")
        self.assertEqual((n["harness"], n["native_id"]), ("cursor", "chat_dump"))
        self.assertNotEqual(n["native_id"], "seatloom__d73725d80aa3442b0d73b0e4a7911be0")

    def test_probe_plan_covers_both_spellings_per_session(self):
        """The refresh procedure probes what this returns. It must name BOTH
        spellings for a codex session, or a refresh would report a session as
        absent when its row is merely keyed the other way."""
        from vela_wash.session_key import index_probe_plan
        plan = index_probe_plan([
            {"agent": "claude-code", "raw": REL_RAW},
            {"agent": "codex", "raw": "raw/codex/rollout-2026-04-30T00-35-01-"
             "019dda18-07f6-79e0-8008-e4eb16a5448d.jsonl"},
        ])
        ids = {(p["harness"], p["id"]) for p in plan}
        self.assertIn(("claude-code", SESSION_ID), ids)
        self.assertIn(("codex", "019dda18-07f6-79e0-8008-e4eb16a5448d"), ids)
        self.assertIn(("codex", "rollout-2026-04-30T00-35-01-"
                       "019dda18-07f6-79e0-8008-e4eb16a5448d"), ids)
        self.assertEqual(len(plan), 3)

    def test_a2_rejects_paths_that_are_not_archive_pointers(self):
        for bad in ["", "raw/", "raw/notaharness/x.jsonl", "src/main.rs",
                    "raw/claude-code"]:
            self.assertIsNone(a2_parse(bad), bad)

    def test_corpus_relative_requires_the_corpus_root(self):
        self.assertEqual(corpus_relative(f"~/Documents/GitHub/AgentHistory/{REL_RAW}",
                                         AH), REL_RAW)
        self.assertEqual(corpus_relative(os.path.join(AH, REL_RAW), AH), REL_RAW)
        # a path outside raw/ is not an archived-session pointer at all
        self.assertIsNone(corpus_relative("~/Documents/GitHub/AgentHistory/summaries/x.md", AH))
        self.assertIsNone(corpus_relative("/etc/passwd", AH))
        self.assertIsNone(corpus_relative("", AH))


class TestSessionIndexLoading(unittest.TestCase):
    """The index is the guard's only window on the live table, so loading it
    fails closed on every way it could lie."""

    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.path = os.path.join(self.tmp, "index.json")

    def _write(self, payload):
        with open(self.path, "w", encoding="utf-8") as f:
            json.dump(payload, f)
        return self.path

    def test_missing_index_is_refused_with_the_refresh_instruction(self):
        with self.assertRaises(IndexError_) as ctx:
            SessionIndex.load(os.path.join(self.tmp, "nope.json"))
        self.assertIn("Refresh", str(ctx.exception))

    def test_wrong_schema_is_refused(self):
        p = self._write({"schema": "someone-elses/v9", "fetched_at": "2026-10-16T00:00:00+00:00",
                         "rows": _index_rows()})
        with self.assertRaises(IndexError_) as ctx:
            SessionIndex.load(p)
        self.assertIn("schema", str(ctx.exception))

    def test_row_missing_a_field_is_refused(self):
        bad = _index_rows()[0]
        del bad["raw_path"]
        p = self._write({"schema": INDEX_SCHEMA,
                         "fetched_at": "2026-10-16T00:00:00+00:00", "rows": [bad]})
        with self.assertRaises(IndexError_):
            SessionIndex.load(p)

    def test_empty_index_is_refused(self):
        """An empty index would resolve nothing and refuse every record for the
        wrong reason — say so instead of guessing."""
        p = self._write({"schema": INDEX_SCHEMA,
                         "fetched_at": "2026-10-16T00:00:00+00:00", "rows": []})
        with self.assertRaises(IndexError_) as ctx:
            SessionIndex.load(p)
        self.assertIn("empty", str(ctx.exception))

    def test_stale_index_is_refused(self):
        old = (datetime.now(timezone.utc) - timedelta(days=30)).isoformat(timespec="seconds")
        p = self._write({"schema": INDEX_SCHEMA, "fetched_at": old,
                         "rows": _index_rows()})
        idx = SessionIndex.load(p, max_age_days=14)
        with self.assertRaises(IndexError_) as ctx:
            idx.assert_fresh()
        self.assertIn("stale", str(ctx.exception).lower())

    def test_fresh_index_passes(self):
        now = datetime.now(timezone.utc).isoformat(timespec="seconds")
        p = self._write({"schema": INDEX_SCHEMA, "fetched_at": now,
                         "rows": _index_rows()})
        idx = SessionIndex.load(p, max_age_days=14)
        idx.assert_fresh()
        self.assertEqual(idx.resolve_a1("claude-code", SESSION_ID)["raw_path"], REL_RAW)


class TestValidateSessionKeys(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.path = write_index(os.path.join(self.tmp, "index.json"), _index_rows(),
                                source="test", scope="test")

    def _index(self):
        return SessionIndex.load(self.path)

    def test_both_spellings_resolving_to_one_row_passes(self):
        res = validate_session_keys([_record()], self._index(), AH)
        self.assertTrue(res["ok"], res["problems"])
        self.assertEqual(res["resolved_sessions"], [("claude-code", SESSION_ID)])

    def test_a_dangling_source_ref_is_refused(self):
        """The real defect: the L2 anchor is perfect, the session key is not."""
        rec = _record(source_ref="codex:019dda18-07f6-79e0-8008-e4eb16a5448d")
        res = validate_session_keys([rec], self._index(), AH)
        self.assertFalse(res["ok"])
        self.assertIn("does not resolve", res["problems"][0])
        self.assertIn("A1", res["problems"][0])

    def test_a_dangling_refusal_names_the_spelling_the_table_uses(self):
        """When A2 resolves and A1 does not, the refusal must say which key the
        row actually has — that is the difference between a wrong record and a
        row a migration missed."""
        rows = _index_rows() + [{
            "harness": "codex",
            "id": "rollout-2026-04-30T00-35-01-019dda18-07f6-79e0-8008-e4eb16a5448d",
            "raw_path": "raw/codex/rollout-2026-04-30T00-35-01-019dda18-07f6-79e0-8008-e4eb16a5448d.jsonl",
            "project": "seatloom", "corpus_loaded": False}]
        p = write_index(os.path.join(self.tmp, "i2.json"), rows, "test", "test")
        rec = _record(source_ref="codex:019dda18-07f6-79e0-8008-e4eb16a5448d",
                      l2_file_path="~/Documents/GitHub/AgentHistory/raw/codex/"
                                   "rollout-2026-04-30T00-35-01-"
                                   "019dda18-07f6-79e0-8008-e4eb16a5448d.jsonl")
        res = validate_session_keys([rec], SessionIndex.load(p), AH)
        self.assertFalse(res["ok"])
        msg = res["problems"][0]
        self.assertIn("pre-021 filename stem", msg)
        self.assertIn("corpus NOT loaded by 021", msg)

    def test_a_path_with_no_row_behind_it_is_refused(self):
        """A raw file on disk is NOT a row in the table. The archive index and
        the table disagree (271 files on disk, 261 rows reachable), so A2 has to
        be checked against the table too."""
        rows = _index_rows(id="some-other-session")
        p = write_index(os.path.join(self.tmp, "i3.json"), rows, "test", "test")
        res = validate_session_keys([_record()], SessionIndex.load(p), AH)
        self.assertFalse(res["ok"])
        self.assertIn("A2", res["problems"][0])

    def test_the_two_spellings_naming_different_sessions_is_refused(self):
        rows = _index_rows() + [{
            "harness": "claude-code", "id": "other-session",
            "raw_path": "raw/claude-code/other.jsonl",
            "project": "seatloom", "corpus_loaded": True}]
        p = write_index(os.path.join(self.tmp, "i4.json"), rows, "test", "test")
        rec = _record(source_ref="claude-code:other-session")
        res = validate_session_keys([rec], SessionIndex.load(p), AH)
        self.assertFalse(res["ok"])
        self.assertIn("different sessions", res["problems"][0])

    def test_a_malformed_source_ref_is_refused(self):
        res = validate_session_keys([_record(source_ref="claude-code")],
                                    self._index(), AH)
        self.assertFalse(res["ok"])
        self.assertIn("is not", res["problems"][0])

    def test_a_path_outside_the_corpus_is_refused(self):
        res = validate_session_keys(
            [_record(l2_file_path="/etc/passwd")], self._index(), AH)
        self.assertFalse(res["ok"])
        self.assertIn("outside", res["problems"][0].replace("corpus root", "outside"))

    def test_a_re_key_done_as_insert_leaves_two_rows_and_is_refused(self):
        """A re-key performed as INSERT instead of UPDATE leaves the old row and
        the new one pointing at the same raw_path. The guard must not quietly
        pick one: two rows for one file means the session's identity is
        ambiguous, and that ambiguity is exactly what a key is supposed to
        remove."""
        rows = _index_rows() + [{
            "harness": "claude-code", "id": "also-this-session",
            "raw_path": REL_RAW, "project": "seatloom", "corpus_loaded": True}]
        p = write_index(os.path.join(self.tmp, "i5.json"), rows, "test", "test")
        rec = _record(source_ref="claude-code:also-this-session")
        res = validate_session_keys([rec], SessionIndex.load(p), AH)
        self.assertFalse(res["ok"])
        self.assertIn("different sessions", res["problems"][0])

    def test_a_codex_row_keyed_with_the_harness_prefix_resolves(self):
        """Measured 2026-10-19: the re-keyed codex rows carry the harness prefix
        INSIDE the id column (`id = 'codex:019dce05-…'`), so A1's
        `<harness>:<native id>` resolves through a second lookup, not only the
        bare native id. Aegis verified this the hard way — his first JOIN used
        replace(source_ref,'codex:',) which stripped the colon and therefore
        never matched a key that itself contains one."""
        rows = [{"harness": "codex",
                 "id": "codex:019dda18-07f6-79e0-8008-e4eb16a5448d",
                 "raw_path": "raw/codex/rollout-2026-04-30T00-35-01-"
                             "019dda18-07f6-79e0-8008-e4eb16a5448d.jsonl",
                 "project": "seatloom", "corpus_loaded": True}]
        p = write_index(os.path.join(self.tmp, "i6.json"), rows, "test", "test")
        idx = SessionIndex.load(p)
        rec = _record(harness="codex",
                      session_id="019dda18-07f6-79e0-8008-e4eb16a5448d",
                      source_ref="codex:019dda18-07f6-79e0-8008-e4eb16a5448d",
                      l2_file_path="~/Documents/GitHub/AgentHistory/raw/codex/"
                                   "rollout-2026-04-30T00-35-01-"
                                   "019dda18-07f6-79e0-8008-e4eb16a5448d.jsonl")
        res = validate_session_keys([rec], idx, AH)
        self.assertTrue(res["ok"], res["problems"])
        self.assertEqual(res["resolved_sessions"],
                         [("codex", "codex:019dda18-07f6-79e0-8008-e4eb16a5448d")])

    def test_a_bare_uuid_row_still_resolves(self):
        """The claude-code rows are keyed by the bare native id, so both shapes
        must work — the lookup cannot prefer one spelling."""
        res = validate_session_keys([_record()], self._index(), AH)
        self.assertTrue(res["ok"], res["problems"])

    def test_an_unknown_source_kind_fails_closed(self):
        """No silent skip: a source kind this gate has no rule for stops the
        write until someone writes the rule."""
        res = validate_session_keys([_record(source_kind="document")],
                                    self._index(), AH)
        self.assertFalse(res["ok"])
        self.assertIn("no resolution rule", res["problems"][0])

    def test_the_check_is_exhaustive_not_sampled(self):
        """The L2 sampling lesson: a 5-record random sample reported 3 bad
        anchors as clean. A poisoned record anywhere in the set must be found,
        including the last one and including a set of one."""
        clean = [_record(record_id=f"{i:064x}") for i in range(40)]
        for position in (0, 1, 20, 39):
            poisoned = list(clean)
            poisoned[position] = _record(
                record_id=f"{position:064x}",
                source_ref="codex:019dda18-07f6-79e0-8008-e4eb16a5448d")
            res = validate_session_keys(poisoned, self._index(), AH)
            self.assertFalse(res["ok"], f"position {position} was not caught")
            self.assertEqual(len(res["problems"]), 1)
        # and a single poisoned record on its own
        res = validate_session_keys(
            [_record(source_ref="codex:019dda18-07f6-79e0-8008-e4eb16a5448d")],
            self._index(), AH)
        self.assertFalse(res["ok"])


@unittest.skipUnless(os.path.exists(RAW),
                     "the raw corpus is not on disk — the end-to-end cases need a "
                     "real L2 anchor so that only the session key is under test")
class TestDriverRefusesEndToEnd(unittest.TestCase):
    """The driver itself: exit code, no manifest, refusal record left behind."""

    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.records = os.path.join(self.tmp, "records.jsonl")
        self.topics = os.path.join(self.tmp, "topics.json")
        self.out = os.path.join(self.tmp, "s5")
        self.index = os.path.join(self.tmp, "index.json")
        with open(self.topics, "w", encoding="utf-8") as f:
            json.dump({"sessions": []}, f)
        write_index(self.index, _index_rows(), "test", "test")

        # a staged record exactly as S3 leaves it: corpus-relative l2 path, the
        # ~-rooted form applied by the driver, and a digest over those bytes.
        rec = _record()
        rec["l2_file_path"] = REL_RAW
        rec["l2_body_digest"] = self._digest()
        rec["record_id"] = content_digest({**rec, "l2_file_path":
                                           f"~/Documents/GitHub/AgentHistory/{REL_RAW}"})
        with open(self.records, "w", encoding="utf-8") as f:
            f.write(json.dumps(rec, ensure_ascii=False) + "\n")

    @staticmethod
    def _digest():
        h = hashlib.sha256()
        with open(RAW, encoding="utf-8") as f:
            for i, line in enumerate(f, 1):
                if i > 3:
                    break
                if i >= 1:
                    h.update(line.encode("utf-8"))
        return h.hexdigest()

    def _run(self):
        return subprocess.run(
            [sys.executable, DRIVER, "--records", self.records, "--topics", self.topics,
             "--out", self.out, "--index", self.index],
            capture_output=True, text=True)

    def test_a_resolving_session_key_emits_a_manifest(self):
        p = self._run()
        self.assertEqual(p.returncode, 0, p.stdout + p.stderr)
        manifest = os.path.join(self.out, "writein_manifest.json")
        self.assertTrue(os.path.exists(manifest))
        with open(manifest, encoding="utf-8") as f:
            m = json.load(f)
        self.assertTrue(m["session_key"]["ok"])
        self.assertEqual(m["session_key"]["records_checked"], 1)
        self.assertEqual(m["counts"]["inserted"], 1)
        self.assertFalse(os.path.exists(os.path.join(self.out, "writein_refusal.json")))

    def test_a_non_resolving_session_key_refuses_the_manifest(self):
        with open(self.records, encoding="utf-8") as f:
            rec = json.loads(f.read())
        rec["source_ref"] = "codex:019dda18-07f6-79e0-8008-e4eb16a5448d"
        with open(self.records, "w", encoding="utf-8") as f:
            f.write(json.dumps(rec, ensure_ascii=False) + "\n")

        p = self._run()
        self.assertEqual(p.returncode, 1, p.stdout + p.stderr)
        self.assertIn("REFUSING to emit a manifest", p.stdout)
        self.assertIn("session key does not resolve", p.stdout)
        # no manifest, and a refusal record that says why
        self.assertFalse(os.path.exists(os.path.join(self.out, "writein_manifest.json")))
        refusal = os.path.join(self.out, "writein_refusal.json")
        self.assertTrue(os.path.exists(refusal))
        with open(refusal, encoding="utf-8") as f:
            r = json.load(f)
        self.assertFalse(r["session_key"]["ok"])
        self.assertEqual(len(r["session_key"]["problems"]), 1)

    def test_a_stale_manifest_from_an_earlier_run_is_removed_on_refusal(self):
        """A stale plan sitting next to a refusal is the most dangerous artifact
        in the directory: it still looks executable."""
        os.makedirs(self.out, exist_ok=True)
        stale = os.path.join(self.out, "writein_manifest.json")
        with open(stale, "w", encoding="utf-8") as f:
            json.dump({"memories": ["someone could execute this"]}, f)
        with open(self.records, encoding="utf-8") as f:
            rec = json.loads(f.read())
        rec["source_ref"] = "codex:019dda18-07f6-79e0-8008-e4eb16a5448d"
        with open(self.records, "w", encoding="utf-8") as f:
            f.write(json.dumps(rec, ensure_ascii=False) + "\n")

        p = self._run()
        self.assertEqual(p.returncode, 1)
        self.assertIn("removed the stale", p.stdout)
        self.assertFalse(os.path.exists(stale))

    def test_a_missing_index_stops_the_run_before_any_work(self):
        p = subprocess.run(
            [sys.executable, DRIVER, "--records", self.records, "--topics", self.topics,
             "--out", self.out, "--index", os.path.join(self.tmp, "absent.json")],
            capture_output=True, text=True)
        self.assertEqual(p.returncode, 2, p.stdout + p.stderr)
        self.assertIn("REFUSING to run", p.stdout)
        self.assertFalse(os.path.exists(os.path.join(self.out, "writein_manifest.json")))


if __name__ == "__main__":
    unittest.main(verbosity=2)
