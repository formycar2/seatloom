#!/usr/bin/env python3
"""Offline tests: invalid target must fail, valid target must pass."""
import importlib.util
import pathlib
import subprocess
import sys
import tempfile
import unittest

CHECKER = pathlib.Path(__file__).with_name("check-aegis-wake-target.py")
spec = importlib.util.spec_from_file_location("wake_target", CHECKER)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class TargetTests(unittest.TestCase):
    def test_target_cases(self):
        with tempfile.TemporaryDirectory() as root:
            root = pathlib.Path(root)
            full = root / "protocol.md"
            full.write_text("# Protocol\nRead and act.\n", encoding="utf-8")
            empty = root / "empty.md"
            empty.touch()
            for name, path, expected in [
                ("valid", full, 0),
                ("deleted-path", root / "missing.md", 1),
                ("empty", empty, 1),
                ("directory", root, 1),
            ]:
                with self.subTest(name=name):
                    script = root / "wake.sh"
                    script.write_text(f'MSG="# Aegis 自醒心跳 — Read {path} in full and act on it."\n', encoding="utf-8")
                    result = subprocess.run([sys.executable, str(CHECKER), str(script)], capture_output=True, text=True)
                    self.assertEqual(result.returncode, expected, result.stdout + result.stderr)
                    self.assertIn("OK:" if expected == 0 else "FAIL:", result.stdout + result.stderr)

    def test_repository_message_points_to_current_protocol(self):
        script = CHECKER.with_name("aegis-self-wake.sh")
        self.assertEqual(module.check(script).name, "AEGIS-HEARTBEAT-v1.zhs.md")


if __name__ == "__main__":
    unittest.main()
