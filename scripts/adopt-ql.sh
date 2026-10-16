#!/usr/bin/env bash
# adopt-ql.sh — 把 project_delivery/QL 收养成 SeatLoom 里的一个项目 + 一个席位。
#
# 为什么要脚本:三张表有外键,少一行后面全插不进去 ——
#   canonical_events.project_id  -> projects(id)
#   dispatches.target_seat_id    -> seats(id)
#   project_role_bindings.seat_id-> seats(id)
# 而 Aegis 在「改 live 数据库」上被权限分类器封顶(迁移落库、席位注册、
# 账本行都被拦),所以这一步必须由张先生执行。
#
# 插三行,都可重复跑(ON CONFLICT DO NOTHING),不改任何已有行:
#   1. projects            —— QL 这个项目
#   2. seats               —— seat-ql-001,QL 的席位
#   3. project_role_bindings —— 把它绑到 QL 下,给一个角色
#
# 🔴 一个你必须知道的既有行为(v0.1 妥协,不是这个脚本造成的):
#    界面的水合是 `for (const p of projectsDto) { const seats = seatsDto.map(...) }`
#    —— **每个项目下挂的都是全部席位**。useDataStore.ts 的注释原话:
#    「v0.1: all seeded data belongs to the single 'seatloom' project. We
#      attribute every seat/... under each project's ProjectData to make the
#      V1 shell render regardless. Multi-project partitioning lands when
#      schema adds project_id FKs.」
#    所以 seat-ql-001 会出现在 QL 下面(你要的),**也会出现在 seatloom、
#    ColossalAI、OfficeBench 等每一个项目下面**。要按项目真正分开,
#    得等 schema 加 project_id 外键 —— 那是另一件活,已报给 Aegis。
#
# 用法: bash scripts/adopt-ql.sh
set -uo pipefail

CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
QL_PATH="${SEATLOOM_QL_PATH:-/Users/jyxc-dz-0100609/Documents/GitHub/project_delivery/QL}"
QL_SEAT="${SEATLOOM_QL_SEAT:-seat-ql-001}"
QL_SEAT_NAME="${SEATLOOM_QL_SEAT_NAME:-ql}"
QL_MODEL="${SEATLOOM_QL_MODEL:-stepplan-anthropic/step-5-preview}"

[ -d "$QL_PATH" ] || { echo "❌ 找不到 $QL_PATH —— 用 SEATLOOM_QL_PATH 指定"; exit 1; }

# -i 是必须的:没有它,podman 不把 stdin 接进容器,heredoc 到不了 psql,
# 脚本会静默什么都不做。2026-10-10 第一版就是这个 bug。
podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -v ON_ERROR_STOP=1 <<SQL
INSERT INTO projects (id, name, created_at, home_path)
VALUES ('QL', 'project_delivery/QL', now(), '$QL_PATH')
ON CONFLICT (id) DO NOTHING;

INSERT INTO seats (id, name, default_runtime, capability_tags, status,
                   created_at, default_harness, default_agent, default_model)
VALUES ('$QL_SEAT', '$QL_SEAT_NAME', 'Pi', '{ql,delivery,huawei-storage}', 'active', now(),
        'Pi', 'Pi', '$QL_MODEL')
ON CONFLICT (id) DO NOTHING;

INSERT INTO project_role_bindings (seat_id, project_id, role)
VALUES ('$QL_SEAT', 'QL', 'worker')
ON CONFLICT (seat_id, project_id) DO NOTHING;

SELECT 'projects' AS what, id, name FROM projects WHERE id = 'QL'
UNION ALL
SELECT 'seats', id, name FROM seats WHERE id = '$QL_SEAT'
UNION ALL
SELECT 'binding', seat_id, role FROM project_role_bindings WHERE seat_id = '$QL_SEAT';
SQL

echo
echo "若上面看到 QL 三行(项目/席位/绑定),登记成功。把这段输出贴给 Aegis ——"
echo "他据此在 QL 目录写 MCP 配置、建 tmux 会话,席位才真的能跑起来并被你看见。"
