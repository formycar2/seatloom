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
-- 🔴🔴 2026-11-01 01:00 二次修订。00:00 那一版把因果写错了,Lyra 量出来了。
--
-- 原版的错:注释写「capability_tags / default_model 从各席位自己的
-- .seatloom/dispatch-seats.tsv 和 tmux 会话实测抄来,不是我猜的」。
--
-- 🔴 00:00 我写下的「那六个模型值在整个仓库里一处都不存在(grep 全库零命中)」
--    **也是假的**。Lyra 00:45 量出来:在改动前的提交上,
--    `stepplan/step-5-preview` 出现在 **2 个**文件、`stepplan-wash/step-5-preview`
--    **1 个**、`stepplan-wash-anthropic/step-5-preview` **7 个**,
--    其中 docs/coordination/reviews/2026-10-08-aegis-harness-migration-plan-v1.md:67
--    明确写着 `stepplan/step-5-preview` 是「Default workhorse(unlimited)」。
--
--    我的 grep 为什么会零命中 —— 02:00 第三次修订,而这次连我上一轮给的
--    原因也是错的,Lyra 01:xx 逐条测过:
--      「--include=*.md 匹配不到 .zhs.md」 → **假**。GNU grep 的 glob 匹配整个
--        文件名,`*.md` 匹配任何以 .md 结尾的名字。她在 knowledge/ 下实测,
--        `--include=*.md` 返回 4 个文件,其中就有 INDEX.zhs.md。
--      「我的 include 列表会返回零」 → **假**。她把我那条命令原样从仓库根跑,
--        返回 11 个文件;git grep 同样的 glob 返回 7 个,包括我引用的那份评审。
--      「任何路径都到不了零」 → 她试了 include 列表、git grep、限定 scripts/
--        (返回 1)、限定 crates/ 和 ui/(返回 0,但那是子集不是全库)。都到不了零。
--
-- 🔴🔴 真因,02:00 我自己查出来的:**我搜的串和我说结论的串不是同一个。**
--     我的命令搜的是 `stepplan-wash-**anthropic**`,
--     而我的结论写的是 `stepplan-wash/**step-5-preview**`、`stepplan/**step-5-preview**`。
--     三个不同的字符串。搜 A 然后对 B 下结论,是这次一切混乱的源头 ——
--     而我上一轮还给自己编了一个「.zhs.md 匹配不到」的更整齐的故事。
--
--     **一个能解释失败的漂亮机制,比「我不知道为什么」更危险。**
--     我上一轮写下的那个原因读起来完全合理(.zhs.md 确实是这个仓库的特例),
--     所以它会被下一个人记住并用在错误的地方。Lyra 说得准:
--     「诚实的状态是未知。他可能在不同目录、用不同模式、对不同树跑的,
--     我看不出来,读者也看不出来 —— 所以原因该记成未知,而不是猜一个。」
--
-- 所以真正的错只有一条,00:00 我说对了但理由写错了:
--   ✅ 对的那条:.seatloom/dispatch-seats.tsv 里只有「席位 id ↔ tmux 会话名」两列,
--      没有任何模型和能力标签。「从它实测抄来」是假的。
--   ❌ 错的那条:那些模型值不存在于仓库。它们存在,只是没有一处说它们属于这些席位。
--
-- **「值在仓库里存在」和「值对这个席位是对的」是两件事。** 我为了把原版钉死,
--   换了一个更严重但不真实的指控。而一个假的强指控会让真正的那条错显得可疑。
--
-- 一个只在我脑子里存在的模型名会怎样:scripts/seat-launch.sh 的 model_for()
-- 从 seats.default_model 取值,只要含 `/` 就原样传给 pi(它只校验形状,不校验存在性),
-- 于是 pi 会拿到一个不存在的模型。**这是会静默坏掉的那种错。**
--
-- 现在每一列的值都能指出它来自哪个文件:
--   default_runtime / harness / agent = 'Pi' —— 与库里现存的 4 个席位一致
--     (Lyra 10-31 重建时写入),也与 scripts/fix-seat-harness-to-pi.sh 的 UPDATE 一致。
--   capability_tags = seed/001_real_collaboration_baseline.sql 里的历史原值
--     (Claude Code 时代登记的那一份)。它只被存储、不参与路由
--     (grep 全库:只有 seat_registry.rs 的测试碰它),所以用历史值不会改行为。
--   default_model = 'stepplan/step-5-preview',五个核心席位**逐个相同**。
--     出处:docs/coordination/reviews/2026-10-08-aegis-harness-migration-plan-v1.md §4
--     「Default workhorse: stepplan/step-5-preview (unlimited — no credit burn).
--      Routine seat work.」,而且它是线上实际回退链的第一环
--     (knowledge/a_seat_that_cannot_reach_the_model_looks_online_forever.md:24
--      记着实测链路 stepplan/step-5-preview → stepplan-wash/step-5-preview)。
--
--     🔴 五个完全相同是有意的,而且我知道它的风险。seat-launch.sh:33-37 记着
--     2026-10-11 一次实测事故:洗数据席位登记的是无限额的 stepplan-wash-anthropic,
--     重启后被换成普通额度的,面板没有任何告警。**写死一个值给所有席位,
--     会在重启时静默把席位降级。**
--     我仍然这么写的理由:(a) 这一列此刻全是空的,补五个相同的值不比空更坏;
--     (b) seed/005 给那 4 个已采纳外部项目登记的是 stepplan-wash-anthropic,
--           而本脚本 ON CONFLICT DO NOTHING 不碰它们,所以不会把无限额的那个改掉;
--     (c) 精细分配有专门的脚本 scripts/set-seat-default-models.sh,
--           它跑之前会检查四个目标席位是否都存在,不存在就整体回滚,不会改一半。
--     也就是说:这一列现在是「统一的工作马」,不是「正确的最终值」。
--
-- ⚠️ 有一件我核不了,必须说清:set-seat-default-models.sh(10-19)把 aegis/mira/nimbus/lyra
--    分成了 modelproxy/gpt-6.1-sol-sys-ptu。10-30 23:00 之前账本里它们到底是什么,
--    **已经不可考** —— 账本 10-31 被清空,而那之前没有任何一处留下快照。
--    所以我统一用回退值,并把精细分配留给 set-seat-default-models.sh(它自己会在跑之前
--    检查四个 GPT 目标席位是否存在,不存在就整体回滚,不会半改)。
INSERT INTO seats (id, name, default_runtime, capability_tags, status,
                   created_at, default_harness, default_agent, default_model) VALUES
  ('seat-mira-001',  'mira',  'Pi', '{ux-design,react,typescript,component-prototyping}',       'active', now(), 'Pi', 'Pi', 'stepplan/step-5-preview'),
  ('seat-nimbus-001','nimbus','Pi', '{architecture,rust,tauri,infrastructure}',                  'active', now(), 'Pi', 'Pi', 'stepplan/step-5-preview'),
  ('seat-flux-001',  'flux',  'Pi', '{qa,verification,ui-audit,ops}',                            'active', now(), 'Pi', 'Pi', 'stepplan/step-5-preview'),
  ('seat-onyx-001',  'onyx',  'Pi', '{data-modeling,postgres,schema-migration,data-quality}',    'active', now(), 'Pi', 'Pi', 'stepplan/step-5-preview'),
  ('seat-lyra-001',  'lyra',  'Pi', '{product-ownership,coordination,acceptance,planning}',      'active', now(), 'Pi', 'Pi', 'stepplan/step-5-preview'),
  ('seat-vela-001',  'vela',  'Pi', '{corpus-ingest,extraction,redaction,data-washing}',         'active', now(), 'Pi', 'Pi', 'stepplan-wash-anthropic/step-5-preview')
ON CONFLICT (id) DO NOTHING;

SELECT id, name, default_model, status FROM seats ORDER BY id;
SQL

echo
echo "── 核对 ──"
echo "上面应该列出 10 行(6 个核心 + 事故后留下的 4 个外部项目席位)。"
echo "如果少于 10 行,说明有 INSERT 被 ON CONFLICT 跳过了 —— 那说明该席位已存在,"
echo "不是错误。请把这段输出贴给 Aegis,他需要据此发门事件解封等着的派单。"
