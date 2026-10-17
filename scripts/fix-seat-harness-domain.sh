#!/usr/bin/env bash
# fix-seat-harness-domain.sh — 把那两行的 default_harness 改成真值 Stepcode。
#
# 为什么不再是选择题(Nimbus 2026-10-17 量过之后定的,不是我拍的):
#
#   seats.default_harness 用的是 enum Harness { Direct, Stepcode, Custom(String) }。
#   `pi` 是 **agent**(驱动哪个命令行),不是 **harness**(跑在什么包装层/凭证/代理下面)。
#   八个席位全都跑在 stepcode 代理下面,而六个本来就对的行写的正是 Stepcode。
#   所以 harness 这个域是对的,错的是那两行 —— 真值就是 Stepcode,没有第二种可能。
#
# 而 default_agent 那边是另一件事,分开处理:
#   · enum Agent 加 Pi 变体由 Nimbus 做(disp-4WRVFBHEc0ib_TP0WXFI_,提交 1547219),
#     序列化就是 "Pi",正好是库里已经存的值,所以那一列不用改数据。
#   · 但另外六个席位的 default_agent 现在是 ClaudeCode,而它们跑的是 pi —— 那是假的。
#     要不要一起改成 Pi,张先生还没最终点头,这个脚本**不动**那一列。
#
# 🔴 只改 harness 一列,而且只改值为 'Pi' 的行。可重复跑,不碰别的。
#
# 为什么是脚本:写 live 数据库,Aegis 被权限分类器封顶,必须由张先生执行。
#
# 用法: bash scripts/fix-seat-harness-domain.sh
set -uo pipefail

CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"

echo "=== 改之前 ==="
podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -c \
  "select id, default_harness, default_agent from seats order by id;"

# -i 是必须的:没有它 podman 不把 stdin 接进容器,heredoc 到不了 psql,脚本会静默什么都不做。
podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -v ON_ERROR_STOP=1 <<'SQL'
UPDATE seats SET default_harness = 'Stepcode' WHERE default_harness = 'Pi';

SELECT id, default_harness, default_agent, default_model FROM seats ORDER BY id;
SQL

echo
echo "若上面 8 行的 default_harness 全是 Stepcode,这一列修好了。"
echo
echo "=== 但应用还读不到,因为还差一步 ==="
echo "default_agent='Pi' 目前也读不出来 —— enum Agent 里还没有 Pi 这个变体。"
echo "那一步是代码,归 Nimbus(提交 1547219,在他分支上),要验绿、要你点验收卡、合进去之后才成立。"
echo "顺序:先跑这个脚本(harness 真值)→ 等 Nimbus 那一件合并 → 应用才读得到数据库。"
echo
echo "=== 另外六个席位的 default_agent=ClaudeCode 是假的,这个脚本没动 ==="
echo "要不要一起改成 Pi,张先生定。别让六假两真留着。"
