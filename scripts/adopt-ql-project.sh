#!/usr/bin/env bash
# adopt-ql-project.sh — 把 project_delivery/QL 登记成 SeatLoom 里的一个项目。
#
# 为什么要脚本:canonical_events.project_id 有外键指向 projects(id),
# 所以项目不登记,任何记在这个项目下的事件都插不进去 —— 那个目录里的活
# 就永远「游离在 SeatLoom 之外」。而 Aegis 在「改 live 数据库」上被权限分类器
# 封顶(迁移落库、席位注册、账本行都被拦),所以这一步必须由张先生执行。
#
# 这个脚本只做一件事:插一行 projects。ON CONFLICT DO NOTHING,可重复跑,
# 不改任何已有行,不碰其它表。
#
# 注意它**不**注册席位。QL 是张先生自己干活的地方(他用 stepcode claude code),
# 不是被派单的舰队席位,所以不需要 seats 行。如果以后他想往 QL 派单,
# 那是另一件事,另写一个脚本。
#
# 用法: bash scripts/adopt-ql-project.sh
set -uo pipefail

CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
QL_PATH="${SEATLOOM_QL_PATH:-/Users/jyxc-dz-0100609/Documents/GitHub/project_delivery/QL}"

[ -d "$QL_PATH" ] || { echo "❌ 找不到 $QL_PATH —— 用 SEATLOOM_QL_PATH 指定正确路径"; exit 1; }

# -i 是必须的:没有它,podman 不把 stdin 接进容器,heredoc 到不了 psql,
# 脚本会静默什么都不做。2026-10-10 第一版就是这个 bug。
podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -v ON_ERROR_STOP=1 <<SQL
INSERT INTO projects (id, name, created_at, home_path)
VALUES ('QL', 'project_delivery/QL', now(), '$QL_PATH')
ON CONFLICT (id) DO NOTHING;

SELECT id, name, home_path, worker_budget_tokens, supervisor_budget_tokens
  FROM projects WHERE id = 'QL';
SQL

echo
echo "若上面看到 QL 一行,登记成功。把这段输出贴给 Aegis —— 他据此建那个目录的"
echo "MCP 配置和 tmux 会话,你的 claude code 会话才能连上并记进账本。"
