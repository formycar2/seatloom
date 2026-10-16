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
# 我上次说「查不出来」是因为我在命令行里只能查表、调不到界面命令,看不到反序列化抛的错。
#
# 为什么是脚本:改 live 数据库,Aegis 被权限分类器封顶,必须由张先生执行。
#
# 只 UPDATE 那两行的三个字段,ON CONFLICT 无关(不是插入),可重复跑。
# 取值照另外六个好席位(它们全是 ClaudeCode / Stepcode / ClaudeCode),
# 也和 QL 会话里实际要跑的 `stepcode claude code` 一致。
#
# 用法: bash scripts/fix-seat-harness-domain.sh
set -uo pipefail

CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
RUNTIME="ClaudeCode"
HARNESS="Stepcode"
AGENT="ClaudeCode"

echo "=== 修之前的坏行 ==="
podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -c \
  "select id, default_runtime, default_harness, default_agent from seats
   where default_harness not in ('Direct','Stepcode')
      or default_harness like 'Custom(%'
      or default_agent not in ('ClaudeCode','Codex','CursorCli','GeminiCli','Opencode')
   order by id;"

# -i 是必须的:没有它 podman 不把 stdin 接进容器,heredoc 到不了 psql,脚本会静默什么都不做。
podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -v ON_ERROR_STOP=1 <<SQL
UPDATE seats SET
  default_runtime = '$RUNTIME',
  default_harness = '$HARNESS',
  default_agent   = '$AGENT'
WHERE id IN ('seat-ql-001', 'seat-vela-001');

SELECT id, default_runtime, default_harness, default_agent, default_model
FROM seats ORDER BY id;
SQL

echo
echo "若上面 8 行全是 ClaudeCode / Stepcode / ClaudeCode,修好了。把这段输出贴给 Aegis。"
echo
echo "=== 还要做一件事:重启应用 ==="
echo "界面是启动时读的,库改完要重开应用才生效。"
echo "  退出 SeatLoom,再: bash scripts/launch-seatloom.sh"
