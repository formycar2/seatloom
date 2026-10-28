#!/usr/bin/env bash
# bind-seat-to-project.sh — 给一个席位绑定到某个项目(project_role_bindings)。
#
# 为什么要脚本:改 live 数据库的动作我被权限分类器封顶(席位注册、改账本行都被拦,
# 每次都拦对了),所以这一步由张先生执行。这个脚本只插一行绑定,ON CONFLICT DO NOTHING,
# 可重复跑,不改任何已有行,不碰其它表。
#
# 🔴 角色必须由你显式给出,脚本不提供默认值。
#    角色绑定是一条治理事实,不是显示偏好:它决定这个席位在这个项目下被当成什么、
#    该干什么。2026-10-16 修「每个项目挂全部席位」时,seat-vela-001 暴露出一条绑定都没有;
#    我没有替她选角色写进脚本 —— 那是数据和治理决定,不该由界面席位拍。所以这里要求
#    调用方明说,脚本不接受省略。
#
# 用法: bash scripts/bind-seat-to-project.sh <seat_id> <project_id> <role>
#   例: bash scripts/bind-seat-to-project.sh seat-vela-001 seatloom data_engineer
#
# 现有角色(从 live 量到的,不是编的):
#   seatloom    supervisor(aegis) / verifier(flux) / product_owner(lyra) /
#               designer(mira) / architect(nimbus) / data_engineer(onyx)
#   ql_delivery worker(ql)
#
# 跑完请把输出贴给 Aegis。

set -euo pipefail

CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"

SEAT_ID="${1:-}"
PROJECT_ID="${2:-}"
ROLE="${3:-}"

if [ -z "$SEAT_ID" ] || [ -z "$PROJECT_ID" ] || [ -z "$ROLE" ]; then
  cat >&2 <<'USAGE'
用法: bash scripts/bind-seat-to-project.sh <seat_id> <project_id> <role>

三个参数都必填,角色没有默认值 —— 它是治理决定,得由你明说。

例: bash scripts/bind-seat-to-project.sh seat-vela-001 seatloom data_engineer
USAGE
  exit 2
fi

# 角色不在已知集合里也允许(词表会扩),但先说出来,免得打错一个字静默写进去。
KNOWN_ROLES="supervisor verifier product_owner designer architect data_engineer worker"
case " $KNOWN_ROLES " in
  *" $ROLE "*) : ;;
  *) echo "⚠️  角色 '$ROLE' 不在现有集合($KNOWN_ROLES)里。仍会写入,但请确认没打错。" >&2 ;;
esac

# 先只读核一遍:席位和项目都必须真的存在,否则 FK 会拒,而那条错误信息看不出是哪个参数错。
echo "→ 核对席位与项目存在"
podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -v ON_ERROR_STOP=1 -t <<SQL
SELECT 'seat:' || id FROM seats WHERE id = '${SEAT_ID}';
SELECT 'project:' || id FROM projects WHERE id = '${PROJECT_ID}';
SQL

echo "→ 写入绑定 ${SEAT_ID} -> ${PROJECT_ID} (${ROLE})"
# -i 是必须的:没有它 podman 不把 stdin 接进容器,heredoc 到不了 psql,脚本会静默什么都不做。
podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -v ON_ERROR_STOP=1 <<SQL
-- 列名以 information_schema 为准,不凭记忆:这张表没有 active 列(它有的是
-- active_delegation_id)。第一稿写了 active,psql 在 ON_ERROR_STOP 下直接报错退出,
-- 一行都没插 —— 那是对的失败方式,但也说明脚本必须先核对列再写。
INSERT INTO project_role_bindings (project_id, seat_id, role)
VALUES ('${PROJECT_ID}', '${SEAT_ID}', '${ROLE}')
ON CONFLICT DO NOTHING;

SELECT project_id, seat_id, role
  FROM project_role_bindings
 WHERE project_id = '${PROJECT_ID}'
 ORDER BY seat_id;
SQL

echo "✓ 完成。这个项目现在的绑定如上。重开 SeatLoom 的席位列表即可看到变化。"
