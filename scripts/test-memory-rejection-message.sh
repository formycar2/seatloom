#!/usr/bin/env bash
# test-memory-rejection-message.sh — 跑那两条「拒绝必须说得出原因」的行为测试。
#
# 为什么要有这个脚本,而不是只靠 `cargo test`:
#
#   那两条测试是 `#[ignore]`d 的,理由写在 ignore 串里(需要一个指向
#   **临时库**的 DATABASE_URL)。`#[ignore]` 在 `cargo test` 里会计成
#   "N ignored",不是被藏起来 —— 这一点是对的。
#
#   但「被ignore」和「跑过了」是两件事。一个没人跑的测试,和没有测试是同一种绿。
#   所以这个脚本是那道门:它用 `--include-ignored` 真的把那两条跑起来,
#   并且**如果一条都没跑成,它自己失败**。
#
#   🔴 一个会静默跳过的测试,比没有测试更糟 —— 它让人以为有人在看。
#
# 它刻意不碰线上库:库名必须是 scratch 库,测试自己也会按名字拒绝 `seatloom`。

set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROBE_DB="seatloom_nimbus_memref_probe"
URL="postgresql://seatloom:seatloom@localhost:5432/${PROBE_DB}"

say(){ printf '%s\n' "$*"; }
die(){ say "❌ $*"; exit 1; }

[ -d "$ROOT/crates/seatloom-mcp-emit" ] || die "找不到 $ROOT/crates/seatloom-mcp-emit"

# 1. 库必须存在。不存在就说明没人建过,直接失败并说怎么建 ——
#    不要悄悄建一个:建库是对环境的一次改动,该由人决定。
if ! podman exec -e PGPASSWORD=seatloom seatloom-postgres \
     psql -U seatloom -d postgres -tAc \
     "SELECT 1 FROM pg_database WHERE datname='${PROBE_DB}';" 2>/dev/null | grep -q 1; then
  die "临时库 ${PROBE_DB} 不存在。建它(这是对环境的一次改动,所以不自动做):
  podman exec -e PGPASSWORD=seatloom seatloom-postgres psql -U seatloom -d postgres \\
    -c \"CREATE DATABASE ${PROBE_DB};\"
  cat infra/postgres/schema/*.sql | podman exec -i -e PGPASSWORD=seatloom \\
    seatloom-postgres psql -U seatloom -d ${PROBE_DB} -q"
fi

# 2. memories 表必须在。schema 没应用的话,测试会以一个很难懂的理由失败。
if ! podman exec -e PGPASSWORD=seatloom seatloom-postgres \
     psql -U seatloom -d "$PROBE_DB" -tAc \
     "SELECT to_regclass('public.memories') IS NOT NULL;" 2>/dev/null | grep -q t; then
  die "临时库 ${PROBE_DB} 里没有 public.memories —— 先把 schema 应用上:
  cat infra/postgres/schema/*.sql | podman exec -i -e PGPASSWORD=seatloom \\
    seatloom-postgres psql -U seatloom -d ${PROBE_DB} -q"
fi

say "临时库:${PROBE_DB}"
say "跑那两条行为测试(--include-ignored)……"

OUT="$(cd "$ROOT" && DATABASE_URL="$URL" \
  cargo test -p seatloom-mcp-emit --bin seatloom-mcp-emit \
  'memory::tests::' -- --include-ignored --test-threads=1 2>&1)"
RC=$?
printf '%s\n' "$OUT" | sed 's/^/  /'

# 3. 门本身:一条都没跑成,就是失败。
#    这是整个脚本存在的理由 —— 否则它只是一条打印得很漂亮的命令。
#
#    🔴 第一版这里写成「RAN 为空 **且** PASSED=0 **且** FAILED=0」才失败。
#       三个条件用 && 连起来,而 grep -c 在没匹配时返回 0 并且**退出码非零**,
#       但 set -uo pipefail 没有 -e,所以它照样往下走。真正的漏洞是:
#       `grep -oE '[0-9]+ passed'` 在一条都没跑时仍然匹配到 "0 passed",
#       于是 RAN 不是空的,那个 && 永远不成立 —— 门打印「✅ 通过(0 条 ok)」。
#       **一个一条都没跑却报通过的门,比没有门更糟:它让人以为有人在看。**
#       改成直接要求跑过的条数 > 0,不再依赖那个组合条件。
PASSED="$(printf '%s\n' "$OUT" | grep -cE '^test .* \.\.\. ok$')"
FAILED="$(printf '%s\n' "$OUT" | grep -cE '^test .* \.\.\. FAILED$')"

say ""
if [ "$PASSED" = "0" ] && [ "$FAILED" = "0" ]; then
  die "一条测试都没跑 —— 这个门是空的。上面应该有 memory::tests:: 下的若干条。"
fi

if [ "$FAILED" != "0" ]; then
  say "❌ 有失败项(${PASSED} 条 ok,${FAILED} 条失败)"
  exit 1
fi
if [ "$RC" != "0" ]; then
  say "❌ cargo 自己退出非零($RC),但没有一条测试报 FAILED —— 这不对,查上面"
  exit 1
fi
say "✅ 通过(${PASSED} 条 ok)"
exit 0
