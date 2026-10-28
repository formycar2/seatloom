"""Gate 2 (packet §3): S1 MUST hit on known-key samples. A miss is a defect.

Also asserts:
  * marker form ⟪REDACTED:<type>:<fp>⟫ with fp = sha256(secret)[:6]
  * idempotency (redacting twice changes nothing the second time)
  * the coordination-packet naming convention SURVIVES washing (those names
    are the recall content; calibrated exemption, not an oversight)
"""

import hashlib
import json
import os
import sys
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from vela_wash.s1_redact import MARKER, redact_text  # noqa: E402

HERE = os.path.dirname(__file__)


def load_samples():
    with open(os.path.join(HERE, "known_keys.jsonl"), encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


class TestGate2KnownKeys(unittest.TestCase):
    def test_every_known_key_is_redacted(self):
        for s in load_samples():
            with self.subTest(sample=s["id"]):
                out = redact_text(s["text"])
                self.assertNotIn(s["must_redact"], out,
                                 f"{s['id']}: secret survived redaction")
                self.assertIn("⟪REDACTED:", out, f"{s['id']}: no marker emitted")

    def test_fingerprint_is_sha256_first_6(self):
        for s in load_samples():
            with self.subTest(sample=s["id"]):
                out = redact_text(s["text"])
                fp = hashlib.sha256(s["must_redact"].encode()).hexdigest()[:6]
                self.assertIn(fp, out, f"{s['id']}: fingerprint mismatch")

    def test_marker_type_matches_sample(self):
        for s in load_samples():
            with self.subTest(sample=s["id"]):
                out = redact_text(s["text"])
                self.assertIn(f"⟪REDACTED:{s['type']}:", out,
                              f"{s['id']}: expected type {s['type']} in {out!r}")

    def test_same_secret_same_fingerprint(self):
        a = redact_text("key one ak-39pa6bj25stc8jdngfyd3se1eta0jkvq here")
        b = redact_text("key two ak-39pa6bj25stc8jdngfyd3se1eta0jkvq there")
        fa = [w for w in a.split() if w.startswith("⟪REDACTED")][0]
        fb = [w for w in b.split() if w.startswith("⟪REDACTED")][0]
        self.assertEqual(fa, fb)

    def test_idempotent(self):
        for s in load_samples():
            with self.subTest(sample=s["id"]):
                once = redact_text(s["text"])
                twice = redact_text(once)
                self.assertEqual(once, twice, f"{s['id']}: not idempotent")

    def test_marker_itself_never_rematches(self):
        out = redact_text("ak-39pa6bj25stc8jdngfyd3se1eta0jkvq")
        self.assertEqual(redact_text(out), out)

    def test_packet_names_survive(self):
        """Coordination identifiers must NOT be redacted (calibrated exemption)."""
        keep = [
            "packet/vela/washing-pipeline",
            "VELA-2026-10-10-washing-pipeline-p1-delivery-v1",
            "NIMBUS-2026-09-28-runtime-triple-refactor-v1",
            "FLUX-2026-10-08-p0-step4-mcp-emit-verify-v1",
            "docs/coordination/tasks/vela/VELA-2026-10-10-washing-pipeline-v1.md",
            "sk-db-baseline",           # k8s secret name, in-scope shape
            "sk-notification",
            "019dda18-07f6-79e0-8008-e4eb16a5448d",   # uuid
            "b6751a1",                  # short git sha
        ]
        for text in keep:
            with self.subTest(text=text):
                self.assertEqual(redact_text(text), text)

    def test_k8s_names_with_hashes_survive(self):
        self.assertEqual(redact_text("sk-book-oc-gw-7d9f8c5b"), "sk-book-oc-gw-7d9f8c5b")

    def test_content_hashes_survive(self):
        """Pure-hex runs are public content hashes (git SHAs, sha256/Docker
        digests) — measured in-scope entropy 3.6-3.9, redacting them would
        destroy pointer material without protecting anything."""
        keep = [
            "eb84fdc6f2a3a064445bb2a2fbc89c515666c428d6c96b6ab68a4cd218819688",
            "1b3c642526f8d274b12bdcd93b90aeb7e68a1f59eb20613adb96ed561c01d98c",
            "sha256:26d41993b1f2f78dbd010160b83b06b0a38e17b57a269e9bf0be7f2078073f98",
        ]
        for text in keep:
            with self.subTest(text=text):
                self.assertEqual(redact_text(text), text)


if __name__ == "__main__":
    unittest.main(verbosity=2)
