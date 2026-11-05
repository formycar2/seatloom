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

# 🔴 这一条是 2026-10-31 补的,因为它在一小时内骗倒了两个人。
#
# 上面那行 cd 会把工作目录改成「脚本所在目录的上一级」。
# 如果脚本被拷贝到别处(临时仓库的根目录、/tmp、别人的机器),
# 它就 cd 到一个不是 git 仓库的地方,于是:
#   git rev-parse 失败 → 基线退化成空树(4b825dc...)
#   → git diff 什么也没有 → 打印「没有改过的文档」→ **退出码 0**
#
# 一个看不见自己仓库的闸门,报告的是成功。
# 2026-10-31 15:00 我和 Lyra 先后踩到:她为了变异测试这个闸门,
# 把 verify-docs.sh 拷到临时仓库根目录而不是 scripts/ 下,
# 闸门 cd 到 /tmp,输出「nothing to check vs 4b825dc」——
# 空树的哈希就摆在输出里,而她(和我)都读过去了。
#
# 她记下了那条分辨:「nothing to check」不是「all conform」。
# 一个说「我看了,什么都没找到」,一个说「我看了,一切正常」。
# 而这里更该做的是让前者根本不可能发生:先证明自己在仓库里。
if ! git rev-parse --git-dir >/dev/null 2>&1; then
  echo "verify-docs: ❌ 当前目录不是 git 仓库: $(pwd)" >&2
  echo "  这个脚本靠 cd \"\$(dirname \$0)/..\" 找仓库根。它被拷贝到别处了。" >&2
  echo "  在看不见仓库的情况下,它会把基线退化成空树、报告『没有改过的文档』并退出 0 ——" >&2
  echo "  那是一个看不见仓库的闸门在报告成功。" >&2
  exit 2
fi

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

  # §4 的第三种形状:常驻规程文档(席位心跳协议)没有日期,因为它不该有 ——
  # 它是**同一份文档的持续修订**,不是 10-31 这一版、10-30 那一版那样的交付物。
  # 实测 AEGIS-HEARTBEAT.md 和 LYRA-HEARTBEAT.md 都不在豁免名单里,
  # 也就是两个席位的心跳协议**从来就不可能通过这个闸门**。
  #
  # 豁免必须按「没有日期段」判,不能按文件名里有 HEARTBEAT 判。
  # 我第一版写的是 *HEARTBEAT*.md,变异测试救了它:把 .zhs 容忍度删掉,
  # 闸门照样 0 违规 —— 因为我的心跳交付文件名里也带 heartbeat,
  # 16 份全被这条豁免吃掉了,§4 一次都没在它们身上跑过。
  # 一条把该查的 16 个文件一起吃掉的豁免,比没有豁免更糟:
  # 它让「我修了闸门」看起来成功了。
  # 真正的区分是有没有 YYYY-MM-DD:规程文档没有,交付物有。
  standing=0
  case "$base" in
    *HEARTBEAT*.md|*HEARTBEAT*.zhs.md)
      printf '%s' "$base" | grep -qE '[0-9]{4}-[0-9]{2}-[0-9]{2}' || standing=1 ;;
  esac
  if [ "$standing" = "1" ]; then
    :
  else
  # 这里必须按 $f(完整路径)分发,不能按 $base。
  # 我改成 $base 之后,§4 一条都没跑过而闸门照样全绿 ——
  # 因为 basename 永远匹配不上 docs/coordination/tasks/* 这种路径模式。
  # 变异测试(删掉 .zhs 容忍度)第二次存活,才把它揪出来。
  case "$f" in
    docs/coordination/tasks/*)      [[ "$base" =~ ^[A-Z]+-[0-9]{4}-[0-9]{2}-[0-9]{2}-.+-v[0-9]+(\.zhs)?\.md$ ]] || echo "$f: task filename must be ROLE-YYYY-MM-DD-<topic>-vN.md (§4)" ;;
    docs/coordination/reviews/*)    [[ "$base" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}-.+\.md$ ]] || echo "$f: review filename must be YYYY-MM-DD-<topic>.md (§4)" ;;
    docs/coordination/acceptance/*) [[ "$base" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}-.+\.md$ ]] || echo "$f: acceptance filename must be YYYY-MM-DD-<topic>.md (§4)" ;;
    docs/coordination/memory/*)     [[ "$base" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}\.md$ ]] || echo "$f: memory filename must be YYYY-MM-DD.md (§4)" ;;
  esac
  fi
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
# Resolve the base to diff against (see lint-changed.sh for the rationale):
# explicit arg > MR base > merge-base with track/infra-foundation > merge-base
# with the default branch > empty tree. Bare origin/main is unusable (GitLab's
# main is an unrelated empty Initial commit).
# 🔴🔴🔴 2026-11-05 17:0x — THE HOOK WAS A FAKE GATE FOR TWO ROUNDS, AND THIS IS WHY.
#
# The pre-commit hook calls `verify-docs.sh HEAD "$DOC_PROBE"`, intending "compare HEAD
# against the commit that is about to exist". But this script takes exactly ONE
# positional argument: BASE. The second argument was silently discarded, and the
# diff below is hardcoded to `$BASE..HEAD` — where HEAD is the LAST COMMITTED
# commit, which by definition does not contain the new file. So the answer was
# always "all conform" for anything not yet committed.
#
# Measured: I staged a markdown file with NO universal header at all, ran the hook,
# and got `pre-commit: OK - docs touched by this commit conform` and exit 0. The
# gate I reported as "wired up" at 15:00 had never refused anything.
#
# THE FIX is a real second argument: RIGHT (default HEAD). When the hook passes
# the probe commit, the diff is BASE..RIGHT and the new file is visible.
#
# 🔴 Why this class of bug is the one I keep making: the script's OUTPUT was
# correct for the question it actually answered ("do the docs changed between
# BASE and HEAD conform?") and wrong for the question I thought I was asking.
# A tool that answers a different question than you asked, in the same format,
# is indistinguishable from a working tool until you feed it a case that must
# fail. So: after wiring any gate, feed it a case that MUST fail.
BASE="${1:-}"
RIGHT="${2:-HEAD}"
if [ -z "$BASE" ]; then
  if [ -n "${CI_MERGE_REQUEST_DIFF_BASE_SHA:-}" ]; then
    BASE="$CI_MERGE_REQUEST_DIFF_BASE_SHA"
  else
    for cand in origin/track/infra-foundation "origin/${CI_DEFAULT_BRANCH:-main}"; do
      git rev-parse --verify -q "$cand" >/dev/null 2>&1 || continue
      mb=$(git merge-base HEAD "$cand" 2>/dev/null) && [ -n "$mb" ] && { BASE="$mb"; break; }
    done
  fi
fi
git rev-parse --verify -q "${BASE:-}^{object}" >/dev/null 2>&1 || BASE="$(git hash-object -t tree /dev/null)"

CHANGED="$( {
  git diff --name-only --diff-filter=ACMR "$BASE" "$RIGHT" -- docs 2>/dev/null
  # The index/worktree lines only make sense when RIGHT is HEAD; comparing them
  # against an arbitrary commit would double-count.
  [ "$RIGHT" = "HEAD" ] && git diff --name-only --diff-filter=ACMR -- docs 2>/dev/null
  [ "$RIGHT" = "HEAD" ] && git ls-files --others --exclude-standard -- docs 2>/dev/null
} | grep -E '\.md$' | sort -u )"

[ -z "$CHANGED" ] && { echo "verify-docs: no changed docs vs $BASE..$RIGHT — nothing to check."; exit 0; }

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
  echo "verify-docs: changed docs vs $BASE..$RIGHT — all conform."
else
  echo "verify-docs: violations found. New/changed docs must follow DOCUMENT_TEMPLATES + COORDINATION_RULES; fix or (for pre-existing debt) add to $GRANDFATHER."
fi
exit "$FAIL"
