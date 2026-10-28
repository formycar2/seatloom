#!/usr/bin/env bash
# fix-seat-harness-to-pi.sh — 按张先生 2026-10-17 的裁定,把全部席位的 default_harness 改成 Pi。
#
# 裁定原话:「现在要求所有的 Harness 都改成 PI」。
#
# ⚠️ 这一改和实测事实相反,张先生在知情后仍然这么定,所以照办,但把代价记在这里:
#
#   Nimbus 2026-10-17 在 live 上量过:seats.default_harness 用的是
#   enum Harness { Direct, Stepcode, Custom(String) },定义是
#   「Process wrapper / credential / proxy layer the agent runs under」。
#   八个席位实际跑在 stepcode 代理下面,六个行写的正是 Stepcode。
#   按那个定义,`pi` 属于 agent(驱动哪个命令行),不属于 harness。
#
#   张先生的读法成立的一面:`pi` 本身也是一个包装层(它包住模型调用),
#   而 stepcode 的代理信息其实已经记在 default_model 里
#   (modelproxy-anthropic/...、stepplan-anthropic/...),harness 这一列并不承载它。
#   两种读法都说得通,这是词汇表的边界问题,不是真假问题 —— 归张先生定。
#
# 🔴 代价,照实说:改成 Pi 之后,「跑在什么包装层下面」这个信息在这张表里就没有了,
#   只剩 default_model 里的代理线索。以后想按 harness 统计「谁走哪条代理」会统计不到。
#
# 🔴 单独跑这个脚本**不能**让应用读到数据库。还必须让 enum Harness 加上 Pi 变体
#   (归 Nimbus,见派单)。否则 default_harness='Pi' 照样反序列化失败,红框照旧。
#   顺序:先跑这个脚本 → 等 Harness 加 Pi 那一件验绿合并 → 应用才读得到。
#
# 为什么是脚本:写 live 数据库,Aegis 被权限分类器封顶,必须由张先生执行。
#
# 改全部 8 行(不是只改那两行坏的),可重复跑。
#
# 用法: bash scripts/fix-seat-harness-to-pi.sh
set -uo pipefail

CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"

echo "=== 改之前 ==="
podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -c \
  "select id, default_harness, default_agent from seats order by id;"

# -i 是必须的:没有它 podman 不把 stdin 接进容器,heredoc 到不了 psql,脚本会静默什么都不做。
podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -v ON_ERROR_STOP=1 <<'SQL'
UPDATE seats SET default_harness = 'Pi';

SELECT id, default_harness, default_agent, default_model FROM seats ORDER BY id;
SQL

echo
echo "若上面 8 行的 default_harness 全是 Pi,数据这一步完成。"
echo
echo "=== 但应用还读不到,因为还差代码那一步 ==="
echo "enum Harness 里还没有 Pi 这个变体,default_harness='Pi' 照样反序列化失败。"
echo "那一步归 Nimbus,要验绿、要你点验收卡、合进去之后才成立。"
echo
echo "=== 另外,default_agent 那一列这个脚本没动 ==="
echo "另外六个现在是 ClaudeCode,而它们跑的是 pi —— 那是假的。"
echo "要不要一起改成 Pi(enum Agent 加 Pi 的代码 Nimbus 已经在做),张先生定。"
