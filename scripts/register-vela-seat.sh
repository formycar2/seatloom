#!/usr/bin/env bash
# register-vela-seat.sh — 把清洗流水线席位 vela 注册进 seats 表。
#
# 为什么要脚本:dispatches.target_seat_id 有 FK 到 seats(id),所以席位不注册,
# 任何派给它的单都插不进去。而 Aegis 在"改 live 数据库"上被权限分类器封顶
# (席位注册、迁移落库都被拦),所以这一步必须由张先生执行。
#
# 这个脚本只做一件事:插一行 seats。ON CONFLICT DO NOTHING,可重复跑。
# 不改任何已有行,不碰其它表。
#
# 跑完请把输出贴给 Aegis —— 他需要据此发门事件 ev-seat-vela-001-registered,
# 才能解封 Lyra 那条等着的清洗派单。
#
# 用法: bash scripts/register-vela-seat.sh
set -uo pipefail

CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"

# -i 是必须的:没有它,podman 不把 stdin 接进容器,下面的 heredoc 永远到不了 psql,
# 脚本会静默地什么都不做(2026-10-10 第一版就是这个 bug —— 跑完零输出、零写入)。
podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -v ON_ERROR_STOP=1 <<'SQL'
INSERT INTO seats (id, name, default_runtime, capability_tags, status,
                   created_at, default_harness, default_agent, default_model)
VALUES ('seat-vela-001', 'vela', 'Pi',
        '{corpus-ingest,extraction,redaction,data-washing}', 'active', now(),
        'Pi', 'Pi', 'stepplan-wash-anthropic/step-5-preview')
ON CONFLICT (id) DO NOTHING;

SELECT id, name, default_runtime, default_model, status
  FROM seats WHERE id = 'seat-vela-001';
SQL

echo
echo "若上面看到 seat-vela-001 一行,注册成功。把这段输出贴给 Aegis。"
