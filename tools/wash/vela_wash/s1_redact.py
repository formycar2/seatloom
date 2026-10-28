"""S1 — redaction. Runs AFTER normalization, BEFORE any LLM call or table write.

Red line (AGENTS.md): historical sessions almost certainly contain API keys /
cookies / intranet info. A miss is a DEFECT, not a blemish — the pipeline stops
and the pattern gets fixed.

Replacement form:  ⟪REDACTED:<type>:<fingerprint>⟫
fingerprint = sha256(secret)[:6] — same secret => same fingerprint (dedup and
correlation stay possible), content never recoverable.

Pattern set (packet §1): ak-* / sk-* / ghp_* / AKIA* / JWT / PEM / Bearer /
65-char stepplan key / cookie / intranet domains + high-entropy heuristics.

Calibration notes (measured against the seatloom corpus, 2026-10-10):
  * `sk-`/`ak-` prefixes are ALSO used by k8s object names in this corpus
    (`sk-db-baseline`, `sk-notification` — 900+ occurrences). A bare prefix
    rule would gut the corpus. The rule therefore requires a key-shaped tail:
    >=16 chars containing a digit or mixed case AND entropy >= 3.0 bits/char.
    Real keys hit (`ak-39pa6bj25stc8jdngfyd3se1eta0jkvq` — found in
    claude-code/b0100cff); k8s names survive (short, lowercase, dashed).
  * The 65-char rule also fires on 65-char base64 segments inside encrypted
    blobs — acceptable over-redaction (that content must not reach an LLM
    verbatim anyway) and it is the packet-mandated stepplan-key shape.
  * Intranet domains are replaced host-only (scheme + path survive) so the
    fact "an internal GitLab MR was referenced" stays recallable.
"""

from __future__ import annotations

import hashlib
import math
import re

MARKER = "⟪REDACTED:{type}:{fp}⟫"

# --- intranet domains (calibrated to this corpus; extend here, not inline) ---
INTRANET_DOMAINS = [
    "basemind.com",       # gitlab.basemind.com, hub.i.basemind.com
    "stepfun-inc.com",    # artifactory*, models-proxy*
    "stepfun.com",        # api.stepfun.com
    "shai-core.ws",       # ssh workspace hosts
    "bestony.com",        # model-proxy infra (crs/subworker)
]

# ---------------------------------------------------------------------------
# pattern table — (name, compiled regex, replacer)
# Order matters: structured secrets first, entropy sweep last.
# ---------------------------------------------------------------------------

def _fp(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()[:6]


def _marker(value: str, kind: str) -> str:
    return MARKER.format(type=kind, fp=_fp(value))


def _shannon_entropy(s: str) -> float:
    if not s:
        return 0.0
    counts: dict[str, int] = {}
    for ch in s:
        counts[ch] = counts.get(ch, 0) + 1
    n = len(s)
    return -sum((c / n) * math.log2(c / n) for c in counts.values())


_PATTERNS: list[tuple[str, re.Pattern, object]] = []


def _p(name: str, pattern: str, flags: int = 0):
    def deco(fn):
        _PATTERNS.append((name, re.compile(pattern, flags), fn))
        return fn
    return deco


@_p("pem_private_key", r"-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----.*?-----END [A-Z0-9 ]*PRIVATE KEY-----", re.DOTALL)
def _r_pem(m: re.Match) -> str:
    return _marker(m.group(0), "pem_private_key")


@_p("jwt", r"\beyJ[A-Za-z0-9_-]{4,}\.[A-Za-z0-9_-]{4,}\.[A-Za-z0-9_-]{4,}\b")
def _r_jwt(m: re.Match) -> str:
    return _marker(m.group(0), "jwt")


@_p("jwt_fragment", r"\beyJ[A-Za-z0-9_-]{8,}")
def _r_jwt_fragment(m: re.Match) -> str:
    # truncated tokens (e.g. "first 20 chars: eyJhbGciOiJSUzI1NiIs...") still
    # leak token material — redact the fragment too
    return _marker(m.group(0), "jwt_fragment")


@_p("aws_access_key_id", r"\b(?:AKIA|ASIA)[0-9A-Z]{16}\b")
def _r_aws(m: re.Match) -> str:
    return _marker(m.group(0), "aws_access_key_id")


@_p("github_token", r"\b(?:ghp_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,})\b")
def _r_github(m: re.Match) -> str:
    return _marker(m.group(0), "github_token")


@_p("bearer_token", r"\bBearer\s+([A-Za-z0-9._~+/=-]{8,})")
def _r_bearer(m: re.Match) -> str:
    return "Bearer " + _marker(m.group(1), "bearer_token")


@_p("basic_auth", r"\bBasic\s+([A-Za-z0-9+/=]{8,})")
def _r_basic(m: re.Match) -> str:
    return "Basic " + _marker(m.group(1), "basic_auth")


def _key_tail_ok(tail: str) -> bool:
    """Key-shaped tail for sk-*/ak-* prefixed values.

    Calibrated on this corpus: k8s object names share the prefixes
    (`sk-db-baseline`, `sk-book-oc-gw-7d9f8c5b` — lowercase, dashed, short
    segments) while real keys are dash-free random or mixed-case random.
    Rule: >=16 chars AND (mixed case with entropy >=3.0, OR no dash with a
    digit and entropy >=3.5).
    """
    if len(tail) < 16:
        return False
    if any(c.isupper() for c in tail):
        return _shannon_entropy(tail) >= 3.0
    if "-" in tail:
        return False  # dashed lowercase = identifier (k8s naming)
    return any(c.isdigit() for c in tail) and _shannon_entropy(tail) >= 3.5


@_p("sk_api_key", r"\bsk-([A-Za-z0-9_-]{16,})")
def _r_sk(m: re.Match) -> str:
    return _marker(m.group(0), "sk_api_key") if _key_tail_ok(m.group(1)) else m.group(0)


@_p("ak_api_key", r"\bak-([A-Za-z0-9_-]{16,})")
def _r_ak(m: re.Match) -> str:
    return _marker(m.group(0), "ak_api_key") if _key_tail_ok(m.group(1)) else m.group(0)


@_p("stepplan_key65", r"\b[A-Za-z0-9]{65}\b")
def _r_stepplan(m: re.Match) -> str:
    return _marker(m.group(0), "stepplan_key")


@_p("cookie_value", r"(?i)\b(?:set-cookie|cookie)\s*[:=]\s*[\"']?([^\s;,\"'}{⟪⟫]{8,})")
def _r_cookie(m: re.Match) -> str:
    head = m.group(0)[: m.start(1) - m.start(0)]
    return head + _marker(m.group(1), "cookie_value")


@_p("session_token", r"(?i)\bsession(?:id)?=([A-Za-z0-9._-]{8,})")
def _r_session(m: re.Match) -> str:
    return m.group(0)[: m.start(1) - m.start(0)] + _marker(m.group(1), "session_token")


@_p("intranet_domain", r"\b(?:[a-z0-9-]+\.)+(?:" + "|".join(d.replace(".", r"\.") for d in INTRANET_DOMAINS) + r")\b")
def _r_intranet(m: re.Match) -> str:
    return _marker(m.group(0), "intranet_domain")


# --- high-entropy sweep (last) --------------------------------------------
# Token: >=24 chars of base64url-ish material, no path separators.
#
# Calibrated against the seatloom corpus (2026-10-10). Raw entropy alone does
# NOT separate secrets from this project's identifiers: coordination packet
# names (`MIRA-2026-04-28-Serial-Restart-...-v1`, entropy 4.62) overlap the
# random-token range. The sweep therefore uses structure, not just entropy:
#   * exempt pure-hex runs of ANY length — measured in-scope: the only
#     long hex strings are 64-char sha256/Docker content digests (public
#     hashes, same class as git SHAs), entropy 3.6-3.9, far below the
#     4.6+ zone where real tokens live;
#   * exempt the packet-id shape `ROLE-YYYY-MM-DD-...` — those names ARE the
#     recall content and must survive washing;
#   * redact >=32-char tokens with >=3 character classes and entropy >=4.6
#     (random base64: p5=4.33/median=4.56 -> catches opaque payloads like the
#     4,484 `gAAAAA...` encrypted blobs in scope while sparing doc names), or
#     >=24-char tokens with entropy >=4.9 (short but violently random).
_ENTROPY_TOKEN = re.compile(r"(?<![A-Za-z0-9_+/=.-])[A-Za-z0-9_-]{24,}={0,2}(?![A-Za-z0-9_+/=.-])")
_HEX_RUN = re.compile(r"^[0-9a-f]{7,}$")   # any-length hex = content hash / git sha / md5 — public
_UUID = re.compile(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
_DASHED_LOWER = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)+$")
_PACKET_ID = re.compile(r"^[A-Z]+-[0-9]{4}-[0-9]{2}-[0-9]{2}-[A-Za-z0-9_-]+$")


def _char_classes(tok: str) -> int:
    return sum([
        any(c.islower() for c in tok),
        any(c.isupper() for c in tok),
        any(c.isdigit() for c in tok),
        any(c in "-_" for c in tok),
    ])


def _entropy_hit(tok: str) -> bool:
    if _HEX_RUN.match(tok) or _UUID.match(tok) or _PACKET_ID.match(tok):
        return False
    ent = _shannon_entropy(tok)
    if _DASHED_LOWER.match(tok) and ent < 4.2:
        return False
    return (len(tok) >= 32 and _char_classes(tok) >= 3 and ent >= 4.6) or \
           (len(tok) >= 24 and ent >= 4.9)


def _entropy_redact(text: str) -> tuple[str, int]:
    hits = 0

    def repl(m: re.Match) -> str:
        nonlocal hits
        tok = m.group(0)
        if _entropy_hit(tok):
            hits += 1
            return _marker(tok, "high_entropy")
        return tok

    return _ENTROPY_TOKEN.sub(repl, text), hits


def redact_text(text: str, counters: dict[str, int] | None = None) -> str:
    """Redact one string. Idempotent: markers never re-match."""
    if not text:
        return text
    for name, rx, fn in _PATTERNS:
        def _counting(m: re.Match, _fn=fn, _name=name) -> str:
            out = _fn(m)
            if out != m.group(0) and counters is not None:
                counters[_name] = counters.get(_name, 0) + 1
            return out
        text = rx.sub(_counting, text)
    text, n = _entropy_redact(text)
    if counters is not None and n:
        counters["high_entropy"] = counters.get("high_entropy", 0) + n
    return text


def redact_turn(turn: dict, counters: dict[str, int] | None = None) -> dict:
    """Redact one canonical turn in place-safe fashion: text + refs.

    tool names, roles, kinds and provenance (src_file/src_line/cwd) are
    structural and never carry secrets; everything the LLM would see passes
    through redact_text.
    """
    out = dict(turn)
    if out.get("text"):
        out["text"] = redact_text(out["text"], counters)
    if out.get("refs"):
        out["refs"] = [redact_text(r, counters) for r in out["refs"]]
    return out


def scan_text(text: str) -> list[str]:
    """Return the pattern names that WOULD fire on this text (gate-2 audit)."""
    if not text:
        return []
    hits = []
    for name, rx, fn in _PATTERNS:
        for m in rx.finditer(text):
            if fn(m) != m.group(0):
                hits.append(name)
                break
    for m in _ENTROPY_TOKEN.finditer(text):
        if _entropy_hit(m.group(0)):
            hits.append("high_entropy")
            break
    return hits
