#!/usr/bin/env bash
# bind-vela-to-seatloom.sh — 把洗数据席位 vela 绑到 SeatLoom 项目下。
#
# 为什么需要:project_role_bindings 是「哪个席位属于哪个项目」的唯一依据,
# 而 vela 一条绑定都没有。这被一个 v0.1 权宜遮了很久:界面水合时
# `for (p of projectsDto) { seatsDto.map(...) }` 把**全部席位挂到每个项目下**,
# 所以 vela 一直「看起来」在 SeatLoom 里。
#
# 张先生 2026-10-16 裁定:Vela 就是 SeatLoom 的席位。所以要真的绑上。
# 这一绑也意味着:等 Mira 把「按绑定过滤」修好之后,vela 不会从 SeatLoom 项目下消失。
#
# 为什么是脚本:插 project_role_bindings 是改 live 数据库,Aegis 被权限分类器
# 封顶(席位注册、迁移落库、账本行都被拦),所以这一步必须由张先生执行。
#
# 只插一行,ON CONFLICT DO NOTHING 可重复跑,不改任何已有行。
#
# 用法: bash scripts/bind-vela-to-seatloom.sh
set -uo pipefail

CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
SEAT="${SEATLOOM_VELA_SEAT:-seat-vela-001}"
PROJECT="${SEATLOOM_VELA_PROJECT:-seatloom}"
ROLE="${SEATLOOM_VELA_ROLE:-data_engineer}"

# 席位必须真的存在,否则外键会拒。先查,查不到就明说,不要静默跳过。
exists="$(podman exec -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -tAc \
  "select count(*) from seats where id='$SEAT'" 2>/dev/null | tr -d ' ')"
if [ "$exists" != "1" ]; then
  echo "❌ 席位 $SEAT 在 seats 表里不存在(查到 '$exists')—— 先注册席位再绑定"
  exit 1
fi

# -i 是必须的:没有它 podman 不把 stdin 接进容器,heredoc 到不了 psql,
# 脚本会静默什么都不做。
podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -v ON_ERROR_STOP=1 <<SQL
INSERT INTO project_role_bindings (seat_id, project_id, role)
VALUES ('$SEAT', '$PROJECT', '$ROLE')
ON CONFLICT (seat_id, project_id) DO NOTHING;

SELECT seat_id, project_id, role FROM project_role_bindings WHERE seat_id = '$SEAT';
SQL

echo
echo "若上面看到 $SEAT / $PROJECT / $ROLE 一行,绑定成功。把这段输出贴给 Aegis。"
echo "角色填的是 $ROLE —— 如果张先生觉得该叫别的,告诉他,那是产品决定。"
