"""Session manifest — the seatloom project scope (16 sessions).

Scope is fixed by operator decision (packet VELA-2026-10-10-washing-pipeline-v1
§0): seatloom ONLY — 16 sessions / 168 MB / 58,015 lines. Expanding to the
other 17 projects is NOT an automatic next step and requires a separate
operator decision.

Line counts below are actual `wc -l` of the raw jsonl files (58,011 jsonl
lines + cursor's 4-turn chat_dump = the 58,015 in the packet; gemini is
register-only, 0 extracted lines).
"""

from __future__ import annotations

import os

# AgentHistory root. Override with VELA_WASH_RAWHOME for tests.
RAWHOME = os.environ.get(
    "VELA_WASH_RAWHOME",
    os.path.expanduser("~/Documents/GitHub/AgentHistory/raw"),
)

PROJECT = "seatloom"

# (harness, session_id, relpath-under-raw, note)
SESSIONS = [
    # --- claude-code (8 sessions, 34 MB) ---
    ("claude-code", "b0100cff-2426-4786-b165-45827c9e2713",
     "claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl", ""),
    ("claude-code", "adb08b3c-2591-4cea-8be5-be5241967415",
     "claude-code/adb08b3c-2591-4cea-8be5-be5241967415.jsonl", ""),
    ("claude-code", "01565715-3e7f-48c8-9ff0-fdd8edbfdf9f",
     "claude-code/01565715-3e7f-48c8-9ff0-fdd8edbfdf9f.jsonl", ""),
    ("claude-code", "5c09cdda-ec7d-4881-89af-2a7c00b53c68",
     "claude-code/5c09cdda-ec7d-4881-89af-2a7c00b53c68.jsonl", ""),
    ("claude-code", "ca5f59be-32fc-493e-ac44-3731062d4c75",
     "claude-code/ca5f59be-32fc-493e-ac44-3731062d4c75.jsonl", ""),
    ("claude-code", "621ae382-d6c1-4f0c-b3fa-7c1155a5ecb8",
     "claude-code/621ae382-d6c1-4f0c-b3fa-7c1155a5ecb8.jsonl", ""),
    ("claude-code", "ccec8b39-b1b9-4445-bb7a-336ec823f8f2",
     "claude-code/ccec8b39-b1b9-4445-bb7a-336ec823f8f2.jsonl", ""),
    ("claude-code", "75984b9d-9917-4057-bc90-8d4fcb553786",
     "claude-code/75984b9d-9917-4057-bc90-8d4fcb553786.jsonl", ""),
    # --- codex (6 sessions, 134 MB) ---
    ("codex", "019dda18-07f6-79e0-8008-e4eb16a5448d",
     "codex/rollout-2026-04-30T00-35-01-019dda18-07f6-79e0-8008-e4eb16a5448d.jsonl", ""),
    ("codex", "019dce05-fb23-7081-aaa6-40d60b4b33d2",
     "codex/rollout-2026-04-27T16-19-51-019dce05-fb23-7081-aaa6-40d60b4b33d2.jsonl", ""),
    ("codex", "019ddc3e-06d0-73c3-a80e-0d49f38ffe80",
     "codex/rollout-2026-04-30T10-35-45-019ddc3e-06d0-73c3-a80e-0d49f38ffe80.jsonl", ""),
    ("codex", "019dfc2f-3035-7d83-88b7-ae941c74e95c",
     "codex/rollout-2026-05-06T15-27-24-019dfc2f-3035-7d83-88b7-ae941c74e95c.jsonl", ""),
    ("codex", "019dfc2c-e64b-7952-b1dd-c416d499939f",
     "codex/rollout-2026-05-06T15-24-54-019dfc2c-e64b-7952-b1dd-c416d499939f.jsonl", ""),
    ("codex", "019dbf27-7abf-7391-9661-d90524e4a554",
     "codex/rollout-2026-04-24T19-02-08-019dbf27-7abf-7391-9661-d90524e4a554.jsonl", ""),
    # --- cursor (1 session): read the EXISTING chat_dump.json, no sqlite ---
    ("cursor", "d73725d80aa3442b0d73b0e4a7911be0",
     "cursor/seatloom__d73725d80aa3442b0d73b0e4a7911be0/chat_dump.json", ""),
    # --- gemini (1 session): register-only, must not block the mainline ---
    ("gemini", "seatloom",
     "gemini/seatloom", "register-only: history/ holds .project_root only"),
]


def session_path(relpath: str) -> str:
    return os.path.join(RAWHOME, relpath)


def session_rel(relpath: str) -> str:
    """Path relative to AgentHistory root, used as the L2 file pointer."""
    return os.path.join("raw", relpath)
