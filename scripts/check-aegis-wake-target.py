#!/usr/bin/env python3
"""Validate the protocol path in a self-wake script without executing the script."""
import pathlib
import re
import shlex
import sys


def check(script):
    content = pathlib.Path(script).read_text(encoding="utf-8")
    lines = re.findall(r'^MSG=(.+)$', content, re.MULTILINE)
    if len(lines) != 1:
        raise ValueError("expected exactly one literal MSG assignment")
    words = shlex.split(lines[0])
    if len(words) != 1:
        raise ValueError("MSG must be one quoted literal")
    message = words[0]
    if not message.startswith("# "):
        raise ValueError("MSG must retain the shell-comment prefix")
    match = re.search(r' — Read (.+) in full and act on it\.$', message)
    if not match:
        raise ValueError("MSG has no unambiguous protocol path")
    path = pathlib.Path(match.group(1))
    if not path.is_absolute() or not path.is_file():
        raise ValueError(f"protocol file does not exist: {path}")
    with path.open(encoding="utf-8") as stream:
        if not stream.read().strip():
            raise ValueError(f"protocol file is empty: {path}")
    return path


if __name__ == "__main__":
    try:
        if len(sys.argv) != 2:
            raise ValueError("usage: check-aegis-wake-target.py <wake-script>")
        print(f"OK: wake protocol exists and is readable: {check(sys.argv[1])}")
    except (OSError, ValueError) as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        sys.exit(1)
