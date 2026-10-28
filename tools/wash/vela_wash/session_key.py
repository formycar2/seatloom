"""S5 gate — the SESSION KEY: both spellings, exhaustively, or the manifest is refused.

A washed memory names the session it came from twice:

  A1  `source_ref`      = `<harness>:<native session id>`      (primary)
  A2  `l2_file_path`    = `<AgentHistory>/raw/<harness>/<file>` (derived)

Both are session keys. `archived_sessions` is keyed `PRIMARY KEY (harness, id)`
(011), so A1 is a direct key lookup and A2 is a `raw_path` lookup. Nothing in
the pipeline used to check either — S5 verified the L2 *line anchors*
exhaustively and left the session reference unverified, which is how a memory
ended up live with a `source_ref` that resolves against no row: the session it
came from then looks like it produced nothing.

WHY A1 IS PRIMARY (contract §3 A1/A2):
  * A1 names the session's OWN identity — the native id the transcript carries.
    Migration 021 re-keyed `archived_sessions.id` TO that native id for 182
    rows, so A1 tracks the table's key by construction. A2 does not: it is
    derived from a filename, and the codex filename is
    `rollout-<timestamp>-<uuid>`, so a path-derived id is only correct if you
    know to strip the timestamp prefix.
  * A2 already broke once, silently: before 021, codex references built one way
    resolved against no row at all, and nothing complained. Measured again on
    2026-10-16 (see README): 5 of the 6 in-scope codex rows are keyed by the
    native id, 1 is still keyed by the pre-021 filename stem.

The gate is EXHAUSTIVE — every record in the write set, never a sample. The
README records why: a 5-record random sample reported 3 bad L2 anchors as
clean. A gate that protects a per-record invariant cannot be statistical.

Resolution needs the live table, and this driver is offline and must not touch
the database (AGENTS.md: no psql; the seat's sanctioned reads are the MCP
tools). So the gate reads a small index of `archived_sessions` rows for the
in-scope sessions, produced by fetching through those read tools — see
`tools/wash/data/archived_sessions_index.json` and the refresh procedure in the
README. The index carries `fetched_at`; the gate REFUSES to run when it is
missing, malformed, or older than the freshness bound, because a guard that
silently trusts a stale snapshot is the same defect with extra steps.
"""

from __future__ import annotations

import json
import os
import re
from datetime import datetime, timezone

INDEX_SCHEMA = "vela-wash/archived-sessions-index/v1"
DEFAULT_INDEX = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                             "data", "archived_sessions_index.json")
DEFAULT_MAX_AGE_DAYS = 14.0

# 011's closed harness set. A harness outside it cannot be resolved, so a
# reference naming one is refused rather than waved through.
HARNESSES = ("codex", "claude-code", "stepcode", "cursor", "gemini", "opencode")

_UUID = r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}"
# codex names its rollout files rollout-<ISO ts>-<uuid>; the uuid is the tail.
_CODEX_UUID_TAIL = re.compile(_UUID + r"$")


class IndexError_(Exception):
    """The session index is missing, malformed, or stale. Fail closed."""


def corpus_relative(l2_file_path: str, raw_root: str) -> str | None:
    """`~/…/AgentHistory/raw/codex/x.jsonl` -> `raw/codex/x.jsonl`.

    Returns None when the path does not live under the corpus root: a pointer
    outside raw/ is not an archived-session pointer at all (red line 3 — raw/
    is the corpus of record).
    """
    if not l2_file_path:
        return None
    p = os.path.expanduser(l2_file_path)
    root = os.path.expanduser(raw_root).rstrip("/") + "/"
    if not p.startswith(root):
        return None
    rel = p[len(root):]
    return rel if rel.startswith("raw/") else None


def a1_parse(source_ref: str) -> tuple[str, str] | None:
    """A1: `<harness>:<native session id>` -> (harness, native_id).

    Exactly one colon is allowed: the id itself never contains one, and a
    second colon means the reference is not the contract's spelling.
    """
    if not source_ref or source_ref.count(":") != 1:
        return None
    harness, native_id = source_ref.split(":", 1)
    if harness not in HARNESSES or not native_id.strip():
        return None
    if native_id != native_id.strip():
        # Surrounding whitespace means the reference was assembled, not copied
        # from the session. Refuse it: silently trimming would launder a defect
        # into a passing key.
        return None
    return harness, native_id


def a2_parse(rel_raw_path: str) -> dict | None:
    """A2: what a raw path says about the session, without trusting the id.

    Returns {harness, native_id, stem, shape}. `native_id` is the session's own
    id as far as the filename can tell; `stem` is the bare filename, which is
    what the pre-021 `archived_sessions.id` was for codex. Both are reported so
    a refusal can say WHICH spelling the table actually uses.

    The derived id is guidance for a human refreshing the index — the gate
    itself resolves A2 by matching `raw_path` exactly, never by an id derived
    from the path. The cursor row shows why: its raw_path is
    `raw/cursor/<proj>__<hash>/chat_dump.json`, so the last component's stem is
    `chat_dump`, which is a file, not a session.
    """
    if not rel_raw_path or not rel_raw_path.startswith("raw/"):
        return None
    parts = rel_raw_path.split("/")
    if len(parts) < 3:
        return None
    harness = parts[1]
    if harness not in HARNESSES:
        return None
    rest = parts[2:]
    # The last path component is the file (or, for cursor/gemini, the archived
    # DIRECTORY — sha256 is NULL for dir-copies, 011).
    stem = rest[-1]
    for ext in (".jsonl", ".json"):
        if stem.endswith(ext):
            stem = stem[: -len(ext)]
            break
    native_id = stem
    shape = "stem"
    if harness == "codex":
        m = _CODEX_UUID_TAIL.search(stem)
        if m:
            native_id = m.group(0)
            shape = "rollout-<ts>-<uuid>"
    return {"harness": harness, "native_id": native_id, "stem": stem,
            "shape": shape, "raw_path": rel_raw_path}


class SessionIndex:
    """The live `archived_sessions` key space for the in-scope sessions.

    Two lookup directions, because the two spellings resolve differently:
      by_key  (harness, id)        -> row     [A1]
      by_raw  raw_path             -> row     [A2]
    A row is a dict {harness, id, raw_path, project, corpus_loaded}.
    """

    def __init__(self, rows: list[dict], fetched_at: str, source: str,
                 scope: str, max_age_days: float):
        self.rows = rows
        self.fetched_at = fetched_at
        self.source = source
        self.scope = scope
        self.max_age_days = max_age_days
        self.by_key = {(r["harness"], r["id"]): r for r in rows}
        self.by_raw = {r["raw_path"]: r for r in rows}

    # ------------------------------------------------------------ loading
    @classmethod
    def load(cls, path: str = DEFAULT_INDEX,
             max_age_days: float = DEFAULT_MAX_AGE_DAYS) -> "SessionIndex":
        """Load and validate the index. Raises IndexError_ (fail closed) when it
        is absent, not the pinned schema, missing a field, or stale."""
        if not os.path.exists(path):
            raise IndexError_(
                f"no archived-session index at {path} — the session-key gate "
                f"cannot resolve A1 or A2 without it. Refresh it from the live "
                f"table through the seat's read tools (see tools/wash/README.md, "
                f"\"Refreshing the archived-session index\").")
        try:
            with open(path, encoding="utf-8") as f:
                data = json.load(f)
        except json.JSONDecodeError as e:
            raise IndexError_(f"{path} is not valid JSON: {e}") from e
        if data.get("schema") != INDEX_SCHEMA:
            raise IndexError_(
                f"{path}: schema {data.get('schema')!r} != {INDEX_SCHEMA!r} — "
                f"refusing an index this driver does not know how to read.")
        rows = data.get("rows")
        if not isinstance(rows, list) or not rows:
            raise IndexError_(f"{path}: no rows — an empty index would resolve "
                              f"nothing and refuse every record for the wrong reason.")
        for r in rows:
            for k in ("harness", "id", "raw_path"):
                if not isinstance(r.get(k), str) or not r[k].strip():
                    raise IndexError_(f"{path}: row missing {k}: {r!r}")
            if r["harness"] not in HARNESSES:
                raise IndexError_(f"{path}: row harness {r['harness']!r} is "
                                  f"outside 011's closed set {HARNESSES}")
        fetched_at = data.get("fetched_at")
        if not fetched_at:
            raise IndexError_(f"{path}: no fetched_at — an index of unknown age "
                              f"cannot be trusted to describe the live table.")
        return cls(rows, fetched_at, data.get("source", "?"),
                   data.get("scope", "?"), max_age_days)

    # ---------------------------------------------------------- freshness
    def age_days(self) -> float:
        ts = self.fetched_at.replace("Z", "+00:00")
        try:
            when = datetime.fromisoformat(ts)
        except ValueError:
            raise IndexError_(f"fetched_at {self.fetched_at!r} is not ISO-8601")
        if when.tzinfo is None:
            when = when.replace(tzinfo=timezone.utc)
        return (datetime.now(timezone.utc) - when).total_seconds() / 86400.0

    def assert_fresh(self) -> None:
        age = self.age_days()
        if age > self.max_age_days:
            raise IndexError_(
                f"the archived-session index is {age:.1f} days old (bound "
                f"{self.max_age_days:g}). A stale index is a lying guard: it "
                f"would resolve keys against a table that has since changed. "
                f"Refresh it (tools/wash/README.md) and re-run.")

    # --------------------------------------------------------- resolution
    def resolve_a1(self, harness: str, native_id: str) -> dict | None:
        """Look a session up by primary key.

        Two spellings are tried, because the table does not use one: measured
        2026-10-19, the re-keyed codex rows carry the harness prefix INSIDE the
        id column (`id = 'codex:019dce05-…'`), while the claude-code and
        cursor/gemini rows carry the bare native id. A1's `source_ref` is always
        `<harness>:<native id>`, so the prefixed row is what a codex reference
        actually resolves to — the colon in source_ref is the harness separator,
        and the same colon is also part of the row's key.
        """
        row = self.by_key.get((harness, native_id))
        if row is not None:
            return row
        return self.by_key.get((harness, f"{harness}:{native_id}"))

    def resolve_a2(self, rel_raw_path: str) -> dict | None:
        return self.by_raw.get(rel_raw_path)


def validate_session_keys(records: list[dict], index: SessionIndex,
                          raw_root: str) -> dict:
    """Exhaustive A1/A2 resolution over the write set.

    A record passes only when BOTH spellings resolve against
    `archived_sessions` AND they name the SAME row. Anything else is a problem
    and the manifest is refused: a half-resolving session key is exactly the
    silent break this gate exists to close.
    """
    problems: list[str] = []
    verdicts: list[dict] = []
    resolved_keys: set[tuple[str, str]] = set()

    for r in records:
        rid = r.get("record_id", "?")[:8]
        source_kind = r.get("source_kind")
        if source_kind != "archived_session":
            # Fail closed: this driver only knows how to resolve the
            # archived_session key. A new source kind needs a rule written
            # for it, not a silent skip.
            problems.append(
                f"{rid}: source_kind={source_kind!r} — the session-key gate has "
                f"no resolution rule for it; write one before storing these")
            continue

        # ---- A1: source_ref -> (harness, id) key lookup
        parsed = a1_parse(r.get("source_ref", ""))
        if parsed is None:
            problems.append(
                f"{rid}: source_ref {r.get('source_ref')!r} is not "
                f"'<harness>:<native session id>' (harness must be one of "
                f"{HARNESSES})")
            verdicts.append({"record_id": rid, "a1": "malformed"})
            continue
        harness, native_id = parsed
        row_a1 = index.resolve_a1(harness, native_id)
        if row_a1 is None:
            # Say which spelling the table DOES use for this session, when the
            # other spelling resolves — that is the difference between a wrong
            # record and a row migration 021 missed.
            hint = ""
            rel = corpus_relative(r.get("l2_file_path", ""), raw_root)
            row_a2 = index.resolve_a2(rel) if rel else None
            if row_a2 is not None:
                loaded = "" if row_a2.get("corpus_loaded", True) else \
                    " (corpus NOT loaded by 021)"
                hint = (f"; A2 does resolve — the row for {rel} is keyed "
                        f"{row_a2['id']!r}{loaded}, i.e. the pre-021 filename "
                        f"stem, not the native id"
                        if row_a2["id"] != native_id else
                        f"; A2 resolves to the same row, so the id spelling is "
                        f"the only thing wrong")
            problems.append(
                f"{rid}: session key does not resolve — A1 "
                f"{r.get('source_ref')!r} has no archived_sessions row{hint}")
            verdicts.append({"record_id": rid, "a1": "unresolved",
                             "a2": "resolved" if row_a2 else "unresolved"})
            continue

        # ---- A2: l2_file_path -> raw_path lookup
        rel = corpus_relative(r.get("l2_file_path", ""), raw_root)
        if rel is None:
            problems.append(
                f"{rid}: l2_file_path {r.get('l2_file_path')!r} is not under "
                f"the corpus root {raw_root}/raw/ — A2 cannot resolve")
            verdicts.append({"record_id": rid, "a1": "resolved", "a2": "outside-corpus"})
            continue
        row_a2 = index.resolve_a2(rel)
        if row_a2 is None:
            problems.append(
                f"{rid}: session key does not resolve — A2 {rel} has no "
                f"archived_sessions row (the archive index on disk is NOT the "
                f"table: a raw file can exist with no row behind it)")
            verdicts.append({"record_id": rid, "a1": "resolved", "a2": "unresolved"})
            continue

        # ---- both resolved: they must be the SAME row
        if (row_a1["harness"], row_a1["id"]) != (row_a2["harness"], row_a2["id"]):
            problems.append(
                f"{rid}: the two spellings name different sessions — A1 "
                f"{r.get('source_ref')!r} -> {row_a1['id']!r} but A2 {rel} -> "
                f"{row_a2['id']!r}. A memory cannot have two origins.")
            verdicts.append({"record_id": rid, "a1": "resolved", "a2": "resolved",
                             "agree": False})
            continue

        resolved_keys.add((row_a1["harness"], row_a1["id"]))
        verdicts.append({"record_id": rid, "a1": "resolved", "a2": "resolved",
                         "agree": True, "row_id": row_a1["id"]})

    return {"records": len(records), "problems": problems,
            "verdicts": verdicts, "resolved_sessions": sorted(resolved_keys),
            "ok": not problems}


def index_probe_plan(manifest_rows: list[dict]) -> list[dict]:
    """The (harness, id) pairs to probe when refreshing the index.

    Both spellings per session, because the table does not use one
    consistently: the native id A1 needs, and the raw-path stem the pre-021
    rows kept. A refresh that probes only one spelling would report a session
    as absent when its row is merely keyed the other way.
    """
    seen: dict[tuple[str, str], dict] = {}
    for row in manifest_rows:
        harness = row.get("agent") or row.get("harness")
        raw = row.get("raw") or row.get("raw_path") or ""
        parsed = a2_parse(raw)
        if not harness or not parsed:
            continue
        cands = [parsed["native_id"]]
        if parsed["stem"] != parsed["native_id"]:
            cands.append(parsed["stem"])
        for cand in cands:
            seen.setdefault((harness, cand), {"harness": harness, "id": cand,
                                              "raw_path": raw})
    return [seen[k] for k in sorted(seen)]


def write_index(path: str, rows: list[dict], source: str, scope: str,
                fetched_at: str | None = None) -> str:
    """Persist a refreshed index. Called by the seat after it has read the live
    table through its sanctioned read tools. Returns the path written."""
    payload = {
        "schema": INDEX_SCHEMA,
        "fetched_at": fetched_at or datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "source": source,
        "scope": scope,
        "rows": sorted(rows, key=lambda r: (r["harness"], r["id"])),
    }
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        json.dump(payload, f, ensure_ascii=False, indent=1)
        f.write("\n")
    return path
