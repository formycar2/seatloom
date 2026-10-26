#!/usr/bin/env bash
# adopt-network-orchestrator.sh — 把 networkOrchestrator 接进 SeatLoom。
#
# 为什么要有这个:张先生 2026-10-26 要在 iTerm 里开一个 saih(窗口 tab 名
# no_window,cwd 在 project_delivery/networkOrchestrator),并且要能在 SeatLoom UI
# 里选中它、指挥它。UI 只枚举 tmux 会话,所以必须把它搬进 tmux;而账本里既没有
# 这个项目也没有这个席位,所以还要建项目 + 建席位。
#
# 🔴 为什么由张先生跑而不是 Aegis:Aegis 被自动权限分类器封顶,
#    不能写 live 数据库(建项目、建席位、改账本行都拦)。
#    Aegis 的职责是把动作写成可审计、可重复跑的脚本,然后给张先生一条命令。
#    不拆成小块绕、不换工具绕、不让席位代跑。
#
# 🔴 默认 dry-run:什么都不写,只打印将要做什么。要真的执行必须显式 --apply。
#    一条会改账本的命令,默认必须是"不做事"的那一档。
set -uo pipefail

DB_CONTAINER="${SEATLOOM_DB_CONTAINER:-seatloom-postgres}"
DB_NAME="${SEATLOOM_DB_NAME:-seatloom}"
DB_USER="${SEATLOOM_DB_USER:-seatloom}"
DB_PASS="${SEATLOOM_DB_PASS:-seatloom}"

# 项目和席位的命名:项目 id 用目录名,和 project_delivery/QL 同一规矩 ——
# 这样 UI 里按项目筛选时,它和 QL 并列出现,一眼看得懂。
PROJECT_ID="project_delivery/networkOrchestrator"
PROJECT_HOME="$HOME/Documents/GitHub/project_delivery/networkOrchestrator"
# 席位字段的值照 QL 的真实行填(2026-10-26 查过 seats 表,不是猜):
#   seat-ql-001 | ql | Pi | Pi | Pi | stepplan-wash-anthropic/step-5-preview | active
# 所以 default_runtime/harness/agent 都是 "Pi" —— Aegis 第一版照字面猜成
# local/saih/saih,是错的。harness 现在实际是 saih,但**账本里记的是 Pi**;
# 这里保持和账本一致,不引入第二种写法(改名是另一件事,不在这个脚本里做)。
SEAT_ID="seat-networkorchestrator-001"
SEAT_NAME="networkorchestrator"
ACTOR_REF="seat:$SEAT_ID"
MODEL="stepplan-wash-anthropic/step-5-preview"
SEAT_RUNTIME="Pi"
SEAT_HARNESS="Pi"
SEAT_AGENT="Pi"
# 角色绑定:照 QL 的真实行填。QL 是外部项目交付席位,role = custom(worker);
# networkOrchestrator 是同一类(项目交付,不是产品席位),所以同 role。
# 🔴 这一步不能省:UI 的实时席位页不是显示所有 tmux 会话,而是
#    list_seats_for_project() JOIN project_role_bindings,再拿这个名单
#    过滤 tmux 会话(boundMirrorNames)。没有绑定行 → 席位名单是空的 →
#    一个席位都不显示。Aegis 第一版就是漏了这里,张先生在 UI 里搜不到。
SEAT_ROLE="custom(worker)"
# parent_project_id 全表没人用(QL 那行也是空),所以这里也不挂父项目。
TMUX_SESSION="NetworkOrchestrator-seatloom"
SESSION_UUID="01a177a8-94cb-7571-ba17-8982de5299f6"

APPLY=0
[ "${1:-}" = "--apply" ] && APPLY=1

say(){ printf '%s\n' "$*"; }
line(){ printf '%s\n' "────────────────────────────────────────────────────────────"; }

# psql 在容器里;宿主机没有 psql 客户端(2026-10-26 实测)。
q(){ podman exec -e "PGPASSWORD=$DB_PASS" "$DB_CONTAINER" \
      psql -X -U "$DB_USER" -d "$DB_NAME" -tAc "$1" 2>&1; }

line; say "  把 networkOrchestrator 接进 SeatLoom"; line
say "  模式            : $([ "$APPLY" = 1 ] && echo 'APPLY — 真的写账本' || echo 'DRY-RUN — 什么都不写')"
say "  项目 id         : $PROJECT_ID"
say "  项目目录        : $PROJECT_HOME"
say "  席位 id         : $SEAT_ID (名字 $SEAT_NAME)"
say "  UI 里显示为     : $SEAT_NAME"
say "  tmux 会话       : $TMUX_SESSION"
say "  resume 的会话   : $SESSION_UUID"
say ""

# ── 0. 前提条件:不满足就停,不做半截 ────────────────────────────────
line; say "0 — 前提条件"; line
fail=0
if [ ! -d "$PROJECT_HOME" ]; then
  say "  ❌ 项目目录不存在:$PROJECT_HOME"; fail=1
else say "  ✅ 项目目录在"; fi

if podman exec "$DB_CONTAINER" true 2>/dev/null; then :; else
  say "  ❌ 连不上数据库容器 $DB_CONTAINER"; fail=1
fi

# 会话文件必须真的在 —— resume 靠它。不在的话 quit 就白退了。
SESSION_DIR="$HOME/.saih/agent/sessions/--Users-jyxc-dz-0100609-Documents-GitHub-project_delivery-networkOrchestrator--"
if ls "$SESSION_DIR"/*"$SESSION_UUID".jsonl >/dev/null 2>&1; then
  say "  ✅ 会话文件在(uuid $SESSION_UUID 可 resume)"
else
  say "  ❌ 找不到会话文件:$SESSION_DIR/*$SESSION_UUID.jsonl —— resume 会失败"; fail=1
fi

if tmux has-session -t "$TMUX_SESSION" 2>/dev/null; then
  say "  ⚠️  tmux 会话 $TMUX_SESSION 已存在(会复用它,不新建)"
else say "  ✅ tmux 会话名没被占用"; fi

[ "$fail" != 0 ] && { say ""; say "❌ 前提条件不满足,停。修好再跑。"; exit 1; }
say ""

# ── 1. 幂等性:已经在账本里的就别重复建 ────────────────────────────
line; say "1 — 查账本现状(决定要不要建)"; line
p_exists="$(q "select count(*) from projects where id='$PROJECT_ID';" | tr -d '[:space:]')"
s_exists="$(q "select count(*) from seats where id='$SEAT_ID';" | tr -d '[:space:]')"
say "  项目已存在? $p_exists (1=已建)"
say "  席位已存在? $s_exists (1=已建)"
say ""

# ── 2. 项目绑定:让 UI 知道这个项目归这个席位 ───────────────────────
# seats 表没有 owner 列(2026-10-26 查过),项目归谁是由 projects 侧记录的。
# 照 project_delivery/QL 的既有形状:QL 是 custom(worker) 席位 + 项目绑定。
line; say "2 — 将要执行的 SQL(DRY-RUN 下只打印)"; line
say "  -- 项目(幂等)"
say "  INSERT INTO projects (id, home_path, name, created_at)"
say "    VALUES ('$PROJECT_ID', '$PROJECT_HOME', 'networkOrchestrator', now())"
say "    ON CONFLICT (id) DO UPDATE SET home_path = EXCLUDED.home_path;"
say ""
say "  -- 席位(幂等)。字段值照 QL 的真实行填,不是猜的。"
say "  INSERT INTO seats (id, name, default_runtime, default_harness,"
say "                    default_agent, default_model, status, created_at)"
say "    VALUES ('$SEAT_ID', '$SEAT_NAME', '$SEAT_RUNTIME', '$SEAT_HARNESS',"
say "            '$SEAT_AGENT', '$MODEL', 'active', now())"
say "    ON CONFLICT (id) DO UPDATE SET default_model = EXCLUDED.default_model,"
say "                                     status = 'active';"
say ""

say "  -- 角色绑定(幂等)。照 QL 的真实行:role = custom(worker)。"
say "  -- 🔴 少了这一行,UI 的席位列表就是空的(它 JOIN 这张表再过滤 tmux)。"
say "  INSERT INTO project_role_bindings (seat_id, project_id, role)"
say "    VALUES ('$SEAT_ID', '$PROJECT_ID', '$SEAT_ROLE')"
say "    ON CONFLICT (seat_id, project_id) DO UPDATE SET role = EXCLUDED.role;"
say ""

if [ "$APPLY" != 1 ]; then
  line; say "这是 DRY-RUN,什么都没改。确认上面这些就是你要的,然后跑:"; line
  say ""
  say "    bash scripts/adopt-network-orchestrator.sh --apply"
  say ""
  exit 0
fi

# ── 3. 真写 ────────────────────────────────────────────────────────
line; say "3 — 写账本"; line
SQL="
INSERT INTO projects (id, home_path, name, created_at)
  VALUES ('$PROJECT_ID', '$PROJECT_HOME', 'networkOrchestrator', now())
  ON CONFLICT (id) DO UPDATE SET home_path = EXCLUDED.home_path;

INSERT INTO seats (id, name, default_runtime, default_harness,
                   default_agent, default_model, status, created_at)
  VALUES ('$SEAT_ID', '$SEAT_NAME', '$SEAT_RUNTIME', '$SEAT_HARNESS',
          '$SEAT_AGENT', '$MODEL', 'active', now())
  ON CONFLICT (id) DO UPDATE SET default_model = EXCLUDED.default_model,
                                   status = 'active';

INSERT INTO project_role_bindings (seat_id, project_id, role)
  VALUES ('$SEAT_ID', '$PROJECT_ID', '$SEAT_ROLE')
  ON CONFLICT (seat_id, project_id) DO UPDATE SET role = EXCLUDED.role;
"
printf '%s\n' "$SQL" | podman exec -i -e "PGPASSWORD=$DB_PASS" "$DB_CONTAINER" \
  psql -X -U "$DB_USER" -d "$DB_NAME" >/tmp/no-adopt-psql.log 2>&1 || {
    say "  ❌ SQL 执行失败,看 /tmp/no-adopt-psql.log"; cat /tmp/no-adopt-psql.log; exit 1; }
say "  ✅ SQL 已执行(项目 + 席位)"

# ── 4. 读回核对:不看"执行成功",看账本真的变了 ─────────────────────
line; say "4 — 读回核对"; line
p_now="$(q "select id||' | '||home_path from projects where id='$PROJECT_ID';" | tr -d '\n')"
s_now="$(q "select id||' | '||name from seats where id='$SEAT_ID';" | tr -d '\n')"
b_now="$(q "select seat_id||' → '||project_id||' as '||role from project_role_bindings where seat_id='$SEAT_ID' and project_id='$PROJECT_ID';" | tr -d '\n')"
say "  项目: ${p_now:-❌没有}"
say "  席位: ${s_now:-❌没有}"
say "  绑定: ${b_now:-❌没有}"
[ -n "$p_now" ] && [ -n "$s_now" ] && [ -n "$b_now" ] || { say "❌ 读回对不上,账本没真的变。停。"; exit 1; }
say ""

# ── 5. tmux 里起 saih(带 resume),并配 MCP ─────────────────────────
line; say "5 — 在 tmux 里起 saih(resume 原会话)"; line
if tmux has-session -t "$TMUX_SESSION" 2>/dev/null; then
  say "  tmux 会话已存在,不重复起"
else
  tmux new-session -d -s "$TMUX_SESSION" -c "$PROJECT_HOME" && say "  ✅ tmux 会话已建"
fi
# cwd 必须是项目目录: SeatLoom 用会话的 cwd 认这是哪个项目的活。
cur_cwd="$(tmux list-panes -t "$TMUX_SESSION" -F '#{pane_current_path}')"
say "  会话 cwd: $cur_cwd"
[ "$cur_cwd" = "$PROJECT_HOME" ] || say "  ⚠️ cwd 不是项目目录,UI 可能认不出项目"

# MCP 配置:saih 读 .saih/mcp.json(不是 .mcp.json)。项目 id 用目录名,
# 这样 UI 按项目筛的时候它和 QL 并列。演员身份用**新席位**,
# 不能复用 seat-ql-001 —— 那会把这里的活记成 QL 干的。
MCP="$PROJECT_HOME/.saih/mcp.json"
if [ -f "$MCP" ]; then
  say "  .saih/mcp.json 已存在,内容:"
  sed 's/^/    /' "$MCP"
else
  say "  ⚠️ 没有 $MCP —— saih 不会加载 MCP,建了也指挥不了"
fi

# 只在 pane 停在 shell 上时才发命令。往 agent 里塞文本不是启动,是乱按键。
pane_cmd="$(tmux list-panes -t "$TMUX_SESSION" -F '#{pane_current_command}' | head -1)"
say "  pane 当前跑的是: $pane_cmd"
if [ "$pane_cmd" = "zsh" ] || [ "$pane_cmd" = "bash" ] || [ "$pane_cmd" = "sh" ]; then
  # resume 用 saih 自己的提示:saih --session <uuid>
  tmux send-keys -t "$TMUX_SESSION" -l "saih --session $SESSION_UUID --approve"
  sleep 0.3
  tmux send-keys -t "$TMUX_SESSION" Enter
  say "  ✅ 已发出 resume 命令,等它起来…"
  sleep 25
else
  say "  ⚠️ pane 不是 shell(是 $pane_cmd),不自动发命令"
  say "     你在这个会话里手动跑: saih --session $SESSION_UUID --approve"
fi

line; say "完成。接下来:"; line
say "  1. 在 SeatLoom UI 的实时席位页,应该能看到 $SEAT_NAME"
say "  2. 派单时选项目 $PROJECT_ID"
say "  3. 如果看不到,先看 tmux attach -t $TMUX_SESSION 里 saih 起没起来"
