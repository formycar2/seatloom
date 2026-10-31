#!/usr/bin/env bash
# register-fleet-seats.sh — 把舰队六个核心席位注册进 seats 表。
#
# 为什么要这个脚本(2026-10-31 10:57 的事故):
#   Lyra 在证明 apply-db.sh 排序缺陷时,把 `-d seatloom` 写成了她本意中的
#   `-d lyra_probe`(差一个 token),对**生产库**跑了
#   `drop schema public cascade; create schema public;`。整个账本没了。
#   她恢复了 schema,但 seed 只留下 4 个席位 —— 而且**是错的 4 个**:
#   留下的是 networkorchestrator / ql / stepaicloud / vela,
#   而舰队真正的六个核心席位是 mira / nimbus / flux / onyx / lyra / vela。
#
#   后果:`dispatches.target_seat_id` 有 FK 到 `seats(id)`,所以任何派给
#   mira / nimbus / flux / onyx / lyra 的单都插不进去。**不补这五条,
#   整个舰队无法再被派活。**
#
# 这个脚本只做一件事:插六行 seats。ON CONFLICT DO NOTHING,可重复跑。
# 不改任何已有行,不碰其它表,不灌任何数据。
#
# ⚠️ 这是写 live 数据库。Aegis 被权限分类器封顶,不能自己跑,
#    也不绕(不让席位代跑、不拆成小块、不换工具)。请张先生执行。
#
# 用法: bash scripts/register-fleet-seats.sh
#       先看一眼要插什么,确认无误再跑。

set -uo pipefail

# 默认干跑。写 live 库的动作必须由张先生明确要求才执行,
# 而「默认执行」意味着任何一次误粘贴、误双击、误 cron 都会写生产库。
# 我自己定的规矩:被权限封顶的动作,写成脚本 + 默认干跑 + 一条命令给他。
# 这个脚本 10-31 12:00 交付时没有这一条,是漏的,不是故意的。
APPLY=0
[ "${1:-}" = "--apply" ] && APPLY=1

CONTAINER="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"

if [ "$APPLY" = "0" ]; then
  echo "── 干跑(不会写任何东西)──"
  echo
  echo "将执行一条 INSERT,六个核心席位,ON CONFLICT (id) DO NOTHING —— 可重复跑。"
  echo "库里现有席位:"
  podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -t -A \
    -c "select '  ' || id || '  ' || name from seats order by id;" 2>&1 || {
    echo "❌ 读不到 seats 表。先确认容器在跑:podman ps | grep postgres" >&2; exit 1; }
  echo
  echo "缺的是这五个(有 FK,不补就没法给它们派单):"
  for s in seat-mira-001 seat-nimbus-001 seat-flux-001 seat-onyx-001 seat-lyra-001; do
    n=$(podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -t -A \
      -c "select count(*) from seats where id='$s';" 2>/dev/null)
    [ "$n" = "0" ] && echo "  $s"
  done
  echo
  echo "确认无误就跑:  bash scripts/register-fleet-seats.sh --apply"
  exit 0
fi

# -i 是必须的:没有它,podman 不把 stdin 接进容器,heredoc 永远到不了 psql,
# 脚本会静默地什么都不做 —— 跑完零输出、零写入,而看起来像成功了。
# (2026-10-10 第一版就是这个 bug。)
podman exec -i -e PGPASSWORD=seatloom "$CONTAINER" psql -U seatloom -d seatloom -v ON_ERROR_STOP=1 <<'SQL'
-- 六个核心席位。capability_tags / default_model 从各席位自己的
-- .seatloom/dispatch-seats.tsv 和 tmux 会话实测抄来,不是我猜的。
INSERT INTO seats (id, name, default_runtime, capability_tags, status,
                   created_at, default_harness, default_agent, default_model) VALUES
  ('seat-mira-001',  'mira',  'Pi', '{ui,ux,frontend,design}',         'active', now(), 'Pi', 'Pi', 'stepplan-wash/step-5-preview'),
  ('seat-nimbus-001','nimbus','Pi', '{backend,architecture,rust,tauri}','active', now(), 'Pi', 'Pi', 'stepplan/step-5-preview'),
  ('seat-flux-001',  'flux',  'Pi', '{quality,verification,testing}',   'active', now(), 'Pi', 'Pi', 'stepplan/step-5-preview'),
  ('seat-onyx-001',  'onyx',  'Pi', '{database,schema,migrations}',     'active', now(), 'Pi', 'Pi', 'stepplan/step-5-preview'),
  ('seat-lyra-001',  'lyra',  'Pi', '{product,prioritisation,acceptance-criteria}', 'active', now(), 'Pi', 'Pi', 'stepplan-wash/step-5-preview'),
  ('seat-vela-001',  'vela',  'Pi', '{corpus-ingest,extraction,redaction,data-washing}', 'active', now(), 'Pi', 'Pi', 'stepplan-wash-anthropic/step-5-preview')
ON CONFLICT (id) DO NOTHING;

SELECT id, name, default_model, status FROM seats ORDER BY id;
SQL

echo
echo "── 核对 ──"
echo "上面应该列出 10 行(6 个核心 + 事故后留下的 4 个外部项目席位)。"
echo "如果少于 10 行,说明有 INSERT 被 ON CONFLICT 跳过了 —— 那说明该席位已存在,"
echo "不是错误。请把这段输出贴给 Aegis,他需要据此发门事件解封等着的派单。"
