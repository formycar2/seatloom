#!/usr/bin/env bash
# fix-seat-harness-domain.sh — 修两行让整个界面读不到数据库的坏数据。
#
# 现象(张先生 2026-10-16 20:27 截图):应用顶部红框
#   「读不到数据库 —— 上面这些不是你的项目」
#   seats row "seat-ql-001": column default_harnass is out of domain:
#   "Pi" is not a valid Harness value (expected one of: Direct, Stepcode, Custom, Custom(<payload>))
#
# 根因:seats.default_harness / default_agent / default_runtime 三列在库里是 text,
# 但 Rust 侧是枚举反序列化,只认固定几个值:
#   Harness: Direct | Stepcode | Custom(...)
#   Agent:   ClaudeCode | Codex | CursorCli | GeminiCli | Opencode | Custom(...)
# 有两行存了 'Pi',读取整表时反序列化失败 → 水合失败 → 界面退回写死的假数据。
#
# 🔴 两行坏数据,一行是我造成的,一行不是:
#   seat-ql-001   'Pi'  ← scripts/adopt-ql.sh 我登记 QL 时填的,填错了
#   seat-vela-001 'Pi'  ← 早就存在,不是我填的
# 也就是说:**水合从 Vela 登记那天起就是坏的**,那三条假项目卡能显示这么久,根子在这。
#
# ⚠️ 但"填什么"不是技术问题,是数据决定,张先生 2026-10-16 已指出我上次擅自照抄另外六个是又一次
#    替他做决定。代码注释明确反对编值:
#      "None means not recorded at the time, never a default to be guessed at.
#       Do not default it to Direct — a fabricated value is indistinguishable from a real one."
#    另外六个填的 ClaudeCode **很可能也是假的** —— 它们跑的是 pi,不是 Claude Code。
#
# 所以这个脚本让你选,不默认:
#   1) Custom("pi")  —— 记「跑 pi」这个事实(枚举里没有这个词,Custom 就是为它准备的)
#   2) 留空 NULL     —— 最诚实:「我们没记过它们跑什么」
#   3) ClaudeCode    —— 和另外六个一致,但如果是假的,就是六假两真
#
# 为什么是脚本:改 live 数据库,Aegis 被权限分类器封顶,必须由张先生执行。
#
# 只 UPDATE 那两行的三个字段,可重复跑。另外六个不动 —— 要不要一起改另说。
#
# 用法: bash scripts/fix-seat-harness-domain.sh
set -uo pipefail

CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"

echo "=== 修之前的坏行 ==="
podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -c \
  "select id, default_runtime, default_harness, default_agent from seats order by id;"

echo
echo "这三个字段记的是什么(代码里的原话):"
echo "  default_harness = 跑在什么包装层/凭证/代理下面   → Direct | Stepcode | Custom(...)"
echo "  default_agent   = 驱动哪个命令行或协议           → ClaudeCode | Codex | CursorCli | GeminiCli | Opencode | Custom(...)"
echo
echo "选一个填法(这三个字段一起改):"
echo "  1) Custom(\"pi\")  记「跑 pi」这个事实"
echo "  2) 留空 NULL     最诚实:我们没记过"
echo "  3) ClaudeCode    和另外六个一致(但它们很可能也是假的)"
printf "你的选择 [1/2/3]: "
read -r CHOICE

case "$CHOICE" in
  1) HARNESS='Custom(pi)'; AGENT='Custom(pi)'; RUNTIME='Custom(pi)' ;;
  2) HARNESS='NULL';      AGENT='NULL';      RUNTIME='NULL' ;;
  3) HARNESS='Stepcode';  AGENT='ClaudeCode'; RUNTIME='ClaudeCode' ;;
  *) echo "❌ 只认 1/2/3,你给的是 '$CHOICE' —— 什么都没改"; exit 1 ;;
esac

# -i 是必须的:没有它 podman 不把 stdin 接进容器,heredoc 到不了 psql,脚本会静默什么都不做。
podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -v ON_ERROR_STOP=1 <<SQL
UPDATE seats SET
  default_runtime = $RUNTIME,
  default_harness = $HARNESS,
  default_agent   = $AGENT
WHERE id IN ('seat-ql-001', 'seat-vela-001');

SELECT id, default_runtime, default_harness, default_agent, default_model
FROM seats ORDER BY id;
SQL

echo
echo "=== 改完了。还要做一件事:重启应用 ==="
echo "界面是启动时读的,库改完要重开应用才生效:"
echo "  退出 SeatLoom,再: bash scripts/launch-seatloom.sh"
echo
echo "⚠️ 另外六个席位(lyra/mira/nimbus/flux/onyx/aegis)这次没动。它们填的 ClaudeCode"
echo "   很可能同样是假的。要不要一起改成你选的那个值,张先生定 —— 别让六假两真留着。"
