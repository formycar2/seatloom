#!/usr/bin/env bash
# fix-ql-role-domain.sh — 把 ql_delivery 那条绑定的角色从 worker 改成 custom(worker)。
#
# 为什么:project_role_bindings.role 在 Rust 侧是 enum SeatRole 反序列化,只认
#   supervisor / product_owner / architect / verifier / designer / data_engineer
#   / custom / custom(<payload>)
# 注意 SeatRole 用的是 snake_case,和 Harness/Agent 的 PascalCase 故意不同(代码注释提过这个不对称)。
#
# 我(Aegis)的 scripts/adopt-ql.sh 给 QL 填了 role='worker',而这个值不在域里。
# Mira 2026-10-17 量出来的,我自己没发现。
#
# 后果:不是读取失败,而是 ql_delivery 项目下会显示 **0 个席位**而不是 1 个 ——
# 一个看起来"这个项目没有人"的假象,比报错更难发现。
#
# 为什么填 custom(worker) 而不是 data_engineer:
#   custom(<payload>) 就是为"域里没有的那个真值"准备的。QL 干的是交付执行,
#   不是数据工程;填 data_engineer 是编一个值,而代码注释对编值的态度很明确 ——
#   「a fabricated value is indistinguishable from a real one」。
#
# 为什么是脚本:写 live 数据库,Aegis 被权限分类器封顶,必须由张先生执行。
#
# 只改那一行,可重复跑。
#
# 用法: bash scripts/fix-ql-role-domain.sh
set -uo pipefail

CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"

echo "=== 改之前 ==="
podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -c \
  "select project_id, seat_id, role from project_role_bindings order by project_id;"

# -i 是必须的:没有它 podman 不把 stdin 接进容器,heredoc 到不了 psql,脚本会静默什么都不做。
podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -v ON_ERROR_STOP=1 <<'SQL'
UPDATE project_role_bindings SET role = 'custom(worker)' WHERE role = 'worker';

SELECT project_id, seat_id, role FROM project_role_bindings ORDER BY project_id;
SQL

echo
echo "若 ql_delivery 那行变成 custom(worker),数据这一步完成。"
echo
echo "=== 但应用还读不到,前面还有两步 ==="
echo "1) enum Harness 加 Pi —— Nimbus 在做(分支 packet/nimbus/harness-pi)"
echo "2) 那一件验绿、你点卡、合并、重装"
echo "顺序:先跑这个脚本 → 等 Harness 那件合并 → 应用才读得到,ql_delivery 下才会显示 1 个席位。"
echo
echo "⚠️ 角色填的是 custom(worker) —— 保住「它是干活的席位」这个事实。"
echo "   如果你觉得该叫别的,告诉我,那是产品决定,我不替你拍。"
