#!/usr/bin/env bash
# test-memory-dedup-query.sh — the gate for the 023 memory-dedup BEHAVIOURAL tests.
#
# Why a script rather than plain `cargo test`:
#
#   1. These tests need a real PostgreSQL, and `cargo test --workspace` must stay
#      green for seats that do not have one. A test that PANICS when DATABASE_URL
#      is unset was measured and rejected: it took the workspace from 188 passed
#      to 198 passed / 7 failed, and all 7 failures were "DATABASE_URL is not
#      set". A gate that is red for unrelated reasons is a gate people learn to
#      ignore.
#
#   2. `#[ignore]` alone would be the silent-skip defect this whole line of work
#      exists to close — a test that reports green while measuring nothing. So
#      the tests ARE `#[ignore]`d (counted in `cargo test` output as `N
#      ignored`, not hidden), and THIS SCRIPT is the gate: it runs them with
#      `--include-ignored` and FAILS IF ZERO TESTS RAN. A configuration where
#      they would all skip is itself a failure, reported as such.
#
# It also refuses the live `seatloom` database by name, because these tests
# insert and delete rows.
#
# Exit codes:
#   0  every behavioural test ran and passed
#   1  a test failed, or no test ran (the silent-skip case), or the URL is live
#   2  the environment cannot run the tests at all (no cargo, no DATABASE_URL)

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO" || exit 2

export PATH="$HOME/.cargo/bin:$PATH"
command -v cargo >/dev/null 2>&1 || { echo "❌ cargo 不在 PATH 上"; exit 2; }

# The scratch database. Default to the one this was measured against, and allow
# an override so a seat with its own scratch database can use it.
DEFAULT_URL="postgresql://seatloom:seatloom@localhost:5432/seatloom_nimbus_dedup_probe"
URL="${DATABASE_URL:-$DEFAULT_URL}"

# Refuse the live database by NAME. `seatloom?` catches a query string glued to
# the name, which a bare substring check misses.
DBNAME="${URL##*/}"
if [ "$DBNAME" = "seatloom" ] || [ "$DBNAME" = "seatloom?${URL#*\?}" ] \
   || [ "${URL##*/}" = "seatloom" ]; then
  echo "❌ 拒绝指向 live seatloom 数据库:这些测试会插入和删除行。"
  echo "   指向一个 scratch 数据库,例如 $DEFAULT_URL"
  exit 1
fi

echo "── 023 memory-dedup behavioural tests ──"
echo "  DATABASE_URL = $URL"

# Apply the schema if `memories` is missing, so a fresh scratch database works.
#
# 🔴 The connectivity check must come FIRST, and separately from the schema
#    check. An earlier version treated "no memories table" as "schema not
#    applied" and tried to apply it, so a URL pointing at a dead port produced
#    "schema apply failed" — correct by accident, but the message named the
#    wrong cause and the silent-skip guard below was unreachable.
if ! podman exec -e PGPASSWORD=seatloom seatloom-postgres \
      psql -U seatloom -d postgres -tAc "SELECT 1" >/dev/null 2>&1; then
  echo "❌ 连不上 seatloom-postgres 容器 —— 先把它跑起来。"
  exit 2
fi
if ! podman exec -e PGPASSWORD=seatloom seatloom-postgres \
      psql -U seatloom -d "$DBNAME" -tAc "SELECT 1" >/dev/null 2>&1; then
  echo "❌ 连不上数据库 $DBNAME —— 它存在吗?"
  exit 2
fi
if ! podman exec -e PGPASSWORD=seatloom seatloom-postgres \
      psql -U seatloom -d "$DBNAME" -tAc "SELECT to_regclass('public.memories')" 2>/dev/null \
      | grep -q 'memories'; then
  echo "  ⚠️  $DBNAME 没有 memories 表 —— 应用 schema"
  podman exec seatloom-postgres mkdir -p /tmp/schema >/dev/null 2>&1
  for f in infra/postgres/schema/*.sql; do
    podman cp "$f" "seatloom-postgres:/tmp/schema/$(basename "$f")" >/dev/null 2>&1
  done
  for f in $(ls infra/postgres/schema/*.sql | sort); do
    b="$(basename "$f")"
    if ! podman exec -e PGPASSWORD=seatloom seatloom-postgres \
          psql -U seatloom -d "$DBNAME" -v ON_ERROR_STOP=1 -q -f "/tmp/schema/$b" >/dev/null 2>&1; then
      echo "  ❌ $b 应用失败"
      exit 1
    fi
  done
  echo "  ✓ schema 已应用"
fi

echo ""
echo "── 跑 behavioural 测试(--include-ignored)──"
OUT="$(DATABASE_URL="$URL" cargo test -p seatloom-core --test memory_dedup_query \
        -- --include-ignored --test-threads=1 2>&1)"
RC=$?
printf '%s\n' "$OUT" | grep -E '^test |test result' | sed 's/^/  /'

# 🔴 失败时必须把**原因**一起印出来。
#    第一版只 grep '^test |test result',于是失败时只印测试名和 FAILED ——
#    panic 的文件、行号、原因全被过滤掉。Lyra 实测踩到:她复现 panic 那次,
#    闸门报 3 个 FAILED 而一个字都没说它们为什么失败,她只能绕过闸门自己跑
#    cargo test 才看到原因。
#    **一个职责是让失败可读的闸门,报出「有东西失败了」却把「为什么」扔掉,
#    比没有闸门更糟** —— 它让读的人以为已经看到了全部。
#    退出码一直是对的,这是可读性缺陷,不是正确性缺陷;但可读性缺陷的成本
#    真实存在:它花了 Lyra 一步。
if [ "$RC" != "0" ]; then
  echo ""
  echo "  ── 失败的细节(为什么)──"
  # 从第一个 '---- ' 分隔线印到结尾:那是 cargo test 写每个失败测试的
  # stdout / panic 消息的地方。'\(^---- .*\)$' 之前的行也要,因为它们带着
  # 'thread ... panicked at file:line' 这个消息本身。
  printf '%s\n' "$OUT" | awk '/^---- /{p=1} p' | sed 's/^/  /'
  # 兜底:万一上面什么都没匹配到(输出形状变了),就把原始输出全给出来。
  # 一个永远不会报错的检查,比没有检查更有害。
  if ! printf '%s\n' "$OUT" | awk '/^---- /{p=1} p' | grep -q .; then
    echo "  (没匹配到失败块 —— 下面是完整输出)"
    printf '%s\n' "$OUT" | sed 's/^/  /'
  fi
fi

# The silent-skip guard. If zero tests ran, the suite measured nothing, and that
# is a failure — not a pass.
RAN="$(printf '%s\n' "$OUT" | grep -cE '^test .* \.\.\. (ok|FAILED)')"
if [ "$RAN" = "0" ]; then
  echo ""
  echo "❌ 0 个测试跑起来了 —— 这一套什么都没测到。"
  echo "   这是静默跳过的情形,它本身是失败,不是通过。"
  exit 1
fi
echo ""
echo "  跑起来的测试数: $RAN"

if [ "$RC" != "0" ]; then
  echo ""
  echo "❌ 有失败项"
  exit 1
fi

echo ""
echo "✅ 023 behavioural 测试全部通过"
