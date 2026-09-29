#!/usr/bin/env bash
# verify-docs.sh — ratchet gate for coordination/authority documents.
#
# Enforces DOCUMENT_TEMPLATES §2 (universal header) / §11.1 (template family),
# COORDINATION_RULES §4 (filename convention) and §10 (English-only, ASCII
# filenames) on docs/**/*.md — but ONLY on files that are new or changed
# relative to a base ref AND not listed in .gitlab/docs-grandfather.txt.
#
# The grandfather list is a snapshot of every doc that FAILS these checks at
# bootstrap time (headerless docs, partial headers, legacy Chinese docs). New
# docs must be born correct; existing debt is deferred. Fix a grandfathered doc
# and remove its line — the ratchet only ever tightens. Regenerate the snapshot
# with:  scripts/verify-docs.sh --audit > .gitlab/docs-grandfather.txt
#
# Modes:
#   verify-docs.sh [BASE_REF]   gate changed docs (default; CI uses this)
#   verify-docs.sh --audit      print every currently-failing doc path (for the
#                               grandfather snapshot); always exits 0
#
# Portable to bash 3.2 (no mapfile / associative arrays). Needs: git, python3.
# Exit 0 = all checked docs conform (or nothing to check); 1 = violations.

set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

GRANDFATHER=".gitlab/docs-grandfather.txt"

tpl_dir() {
  case "$1" in
    T1) echo "docs" ;;               T2) echo "docs/coordination/roles" ;;
    T3) echo "docs/coordination/tasks" ;;  T4) echo "docs/coordination/reviews" ;;
    T5) echo "docs/coordination/acceptance" ;; T6) echo "docs/coordination/memory" ;;
    T7) echo "docs/coordination" ;;
  esac
}

# check_file <path> — echo each violation as "<path>: <reason>"; silent if clean.
check_file() {
  f="$1"; base="$(basename "$f")"

  if printf '%s' "$base" | LC_ALL=C grep -q '[^ -~]'; then
    echo "$f: filename contains non-ASCII characters (§10)"
  fi

  missing=""
  for k in template subtype id status author date version tags; do
    grep -qE "^\| *$k *\|" "$f" || missing="$missing $k"
  done
  if [ -n "$missing" ]; then
    echo "$f: missing universal-header field(s):$missing (§2)"
    return                                     # header broken → skip deeper checks
  fi

  tpl="$(grep -oE '^\| *template *\| *[^|]+' "$f" | head -1 | grep -oE 'T[1-7]')"
  if [ -z "$tpl" ]; then
    echo "$f: template not in T1–T7 (§11)"
  else
    want="$(tpl_dir "$tpl")"
    case "$f" in
      "$want"/*) : ;;
      docs/*.md) [ "$tpl" = "T1" ] || echo "$f: $tpl doc must live under $want/ (§1)" ;;
      *) echo "$f: $tpl doc must live under $want/ (§1)" ;;
    esac
  fi

  case "$base" in
    *.zhs.md) : ;;
    *) if python3 -c "import sys,re; sys.exit(0 if re.search('[一-鿿]', open(sys.argv[1],encoding='utf-8').read()) else 1)" "$f" 2>/dev/null; then
         echo "$f: contains CJK text; new docs must be English (§10)"
       fi ;;
  esac

  case "$f" in
    docs/coordination/tasks/*)      [[ "$base" =~ ^[A-Z]+-[0-9]{4}-[0-9]{2}-[0-9]{2}-.+-v[0-9]+\.md$ ]] || echo "$f: task filename must be ROLE-YYYY-MM-DD-<topic>-vN.md (§4)" ;;
    docs/coordination/reviews/*)    [[ "$base" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}-.+\.md$ ]] || echo "$f: review filename must be YYYY-MM-DD-<topic>.md (§4)" ;;
    docs/coordination/acceptance/*) [[ "$base" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}-.+\.md$ ]] || echo "$f: acceptance filename must be YYYY-MM-DD-<topic>.md (§4)" ;;
    docs/coordination/memory/*)     [[ "$base" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}\.md$ ]] || echo "$f: memory filename must be YYYY-MM-DD.md (§4)" ;;
  esac
}

all_docs() { { git ls-files -- docs; git ls-files --others --exclude-standard -- docs; } | grep -E '\.md$'; }

# --- Audit mode: list every currently-failing doc path (for the snapshot) ---
if [ "${1:-}" = "--audit" ]; then
  all_docs | sort -u | while IFS= read -r f; do
    [ -f "$f" ] || continue
    [ -n "$(check_file "$f")" ] && echo "$f"
  done
  exit 0
fi

# --- Gate mode ---
BASE="${1:-${CI_MERGE_REQUEST_DIFF_BASE_SHA:-${CI_DEFAULT_BRANCH:+origin/$CI_DEFAULT_BRANCH}}}"
BASE="${BASE:-origin/main}"
git rev-parse --verify --quiet "$BASE" >/dev/null || BASE="$(git hash-object -t tree /dev/null)"

CHANGED="$( {
  git diff --name-only --diff-filter=ACMR "$BASE" HEAD -- docs 2>/dev/null
  git diff --name-only --diff-filter=ACMR -- docs 2>/dev/null
  git ls-files --others --exclude-standard -- docs 2>/dev/null
} | grep -E '\.md$' | sort -u )"

[ -z "$CHANGED" ] && { echo "verify-docs: no changed docs vs $BASE — nothing to check."; exit 0; }

is_grandfathered() { [ -f "$GRANDFATHER" ] && grep -qxF "$1" "$GRANDFATHER"; }

FAIL=0
while IFS= read -r f; do
  [ -n "$f" ] && [ -f "$f" ] || continue
  is_grandfathered "$f" && continue
  out="$(check_file "$f")"
  if [ -n "$out" ]; then
    printf '%s\n' "$out" | sed 's/^/  ✗ /'
    FAIL=1
  fi
done <<EOF
$CHANGED
EOF

# id uniqueness across the whole repo (§11)
dupes="$(all_docs | sort -u | while IFS= read -r g; do
  [ -f "$g" ] && grep -oE '^\| *id *\| *[^|]+' "$g" | head -1 | sed 's/.*| *//;s/ *$//'
done | sort | uniq -d)"
[ -n "$dupes" ] && { echo "  ✗ duplicate document id(s): $(printf '%s' "$dupes" | tr '\n' ' ') (§11)"; FAIL=1; }

if [ "$FAIL" -eq 0 ]; then
  echo "verify-docs: changed docs vs $BASE — all conform."
else
  echo "verify-docs: violations found. New/changed docs must follow DOCUMENT_TEMPLATES + COORDINATION_RULES; fix or (for pre-existing debt) add to $GRANDFATHER."
fi
exit "$FAIL"
