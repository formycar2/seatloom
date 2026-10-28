#!/usr/bin/env bash
# set-seat-default-model.sh — 把席位的 default_model 统一改成 stepplan-wash-anthropic/step-5-preview。
#
# 为什么需要它(2026-10-24,张先生指示):
#   席位模型不统一。库里八个席位分两类:
#     modelproxy/gpt-6.1-sol-sys-ptu        (aegis, lyra, mira, nimbus)
#     modelproxy-anthropic/step-5-preview-sys (flux, onyx, ql, vela)
#   张先生要求全部改为 stepplan-wash-anthropic/step-5-preview。
#
# 🔴 为什么这不是一条 UPDATE 了事:
#   这是改 live 数据库的账本行(seats.default_model),属于被权限封顶的动作。
#   「一个能给自己发权限的 agent 等于没有权限系统」—— 同理,一个能改席位登记
#   模型的 agent 也不该由自己执行。所以这里给可审计、可重复跑的脚本,由张先生执行。
#   **也不让席位代跑**:那是把被拒的动作外包,同一个漏洞换个方向。
#
# 这个脚本做什么:
#   1. 预演(默认):只打印将要改什么,不动库
#   2. --apply:真改,并打印改前改后的对照
#   3. --apply --restart:改完再重启对应席位(仅重启 is_busy 判定为空闲的)
#
# 安全检查(每一条不通过就停):
#   A. 目标模型必须在 models.json 里存在 —— 否则席位起不来
#   B. 目标模型的 contextWindow 必须 >= 当前压缩阈值 + reserveTokens 的安全余量。
#      2026-10-24 实测踩过:deepseek-v4.1-flash-sys 的窗口只有 128,000,而
#      reserveTokens=560,000 会让阈值变成负数 -> 恒触发压缩。脚本会算并拒跑。
#   C. 要重启的席位必须 is_busy 判定为空闲 —— 忙就跳过并说明,不打断
#
# 用法:
#   bash scripts/set-seat-default-model.sh                  # 预演
#   bash scripts/set-seat-default-model.sh --apply          # 只改库
#   bash scripts/set-seat-default-model.sh --apply --restart  # 改库 + 重启空闲席位
set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PG="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
TARGET="${SEATLOOM_TARGET_MODEL:-stepplan-wash-anthropic/step-5-preview}"
MODELS_JSON="${HOME}/.pi/agent/models.json"

APPLY=0; RESTART=0
for a in "$@"; do
  case "$a" in
    --apply)   APPLY=1 ;;
    --restart) RESTART=1 ;;
    -h|--help) sed -n '2,40p' "$0"; exit 0 ;;
    *) echo "unknown option: $a" >&2; exit 2 ;;
  esac
done

say() { printf '%s\n' "$*"; }
hr()  { printf '%.0s─' {1..74}; printf '\n'; }

# ── 检查 A:目标模型必须在 models.json 里 ──────────────────────────────────────
if [ ! -f "$MODELS_JSON" ]; then
  say "❌ 找不到 $MODELS_JSON —— 无法校验目标模型是否存在。停。"
  exit 1
fi
MODEL_INFO="$(python3 - "$MODELS_JSON" "$TARGET" <<'PY'
import json, sys
path, target = sys.argv[1], sys.argv[2]
prov, _, mid = target.partition("/")
d = json.load(open(path))
for p, v in d.get("providers", {}).items():
    if not isinstance(v, dict):
        continue
    for m in v.get("models", []):
        if p == prov and m.get("id") == mid:
            print(f'{m.get("contextWindow", 0)}')
            sys.exit(0)
sys.exit(1)
PY
)"
if [ $? -ne 0 ] || [ -z "$MODEL_INFO" ]; then
  say "❌ 目标模型 $TARGET 不在 $MODELS_JSON 里 —— 改过去席位会起不来。停。"
  exit 1
fi

# ── 检查 B:contextWindow 必须容得下当前压缩配置 ───────────────────────────────
SETTINGS="${HOME}/.saih/agent/settings.json"
RESERVE=16384; KEEP=20000
if [ -f "$SETTINGS" ]; then
  RESERVE="$(python3 -c "import json;print(json.load(open('$SETTINGS')).get('compaction',{}).get('reserveTokens',16384))" 2>/dev/null || echo 16384)"
  KEEP="$(python3 -c "import json;print(json.load(open('$SETTINGS')).get('compaction',{}).get('keepRecentTokens',20000))" 2>/dev/null || echo 20000)"
fi
# 阈值 = contextWindow - reserveTokens。必须显著大于 keepRecentTokens,
# 否则压缩后剩余(keepRecent + 摘要)几乎贴住阈值 -> 恒触发压缩。
# 2026-10-24 实测:阈值 220,000 / keepRecent 120,000 -> 4 分钟压一次。
THRESHOLD=$(( MODEL_INFO - RESERVE ))
MIN_SANE=$(( KEEP * 2 ))
say "目标模型      : $TARGET"
say "  contextWindow = $MODEL_INFO"
say "  reserveTokens = $RESERVE  =>  压缩阈值 = $THRESHOLD"
say "  keepRecentTokens = $KEEP"
if [ "$THRESHOLD" -le "$MIN_SANE" ]; then
  say ""
  say "❌ 阈值 $THRESHOLD 不够健康(<= keepRecentTokens 的 2 倍 = $MIN_SANE)。"
  say "   这会让压缩后剩余(keepRecent + 摘要)几乎贴住阈值,导致频繁压缩。"
  say "   先调 ~/.saih/agent/settings.json 的 compaction,再跑本脚本。停。"
  exit 1
fi
say "  => 阈值健康,压缩后可增长空间约 $(( THRESHOLD - KEEP )) token"

# ── 读当前库里各席位的模型 ───────────────────────────────────────────────────
hr
say "将要改成: $TARGET"
hr
CURRENT="$(podman exec -e PGPASSWORD=seatloom "$PG" \
  psql -U seatloom -d seatloom -tAc \
  "select id || '|' || coalesce(default_model,'(空)') from seats order by id" 2>/dev/null)"
if [ -z "$CURRENT" ]; then
  say "❌ 读不到 seats 表(容器 $PG 起了吗?)。停。"
  exit 1
fi
printf '%-20s %-42s %s\n' "seat" "current" "→"
while IFS='|' read -r sid cur; do
  [ -z "$sid" ] && continue
  if [ "$cur" = "$TARGET" ]; then
    printf '%-20s %-42s (已是目标值)\n' "$sid" "$cur"
  else
    printf '%-20s %-42s → %s\n' "$sid" "$cur" "$TARGET"
  fi
done <<< "$CURRENT"

if [ "$APPLY" -eq 0 ]; then
  hr
  say "预演结束,未动库。真改加 --apply。"
  exit 0
fi

# ── 真改 ─────────────────────────────────────────────────────────────────────
hr
say "写入数据库 ..."
podman exec -e PGPASSWORD=seatloom "$PG" \
  psql -U seatloom -d seatloom -c \
  "update seats set default_model = '$TARGET' where default_model is distinct from '$TARGET'" 2>&1 | sed 's/^/  /'
say ""
say "改后:"
podman exec -e PGPASSWORD=seatloom "$PG" \
  psql -U seatloom -d seatloom -tAc \
  "select id || '|' || coalesce(default_model,'(空)') from seats order by id" 2>/dev/null \
  | while IFS='|' read -r sid m; do printf '  %-20s %s\n' "$sid" "$m"; done

# ── 可选:重启空闲席位 ────────────────────────────────────────────────────────
if [ "$RESTART" -eq 1 ]; then
  hr
  say "重启席位(仅 is_busy 判定为空闲的;忙的跳过并说明)"
  say ""
  # seat-sessions.tsv 只有 会话名/session_id/时间/模型 四列,没有 seat id ——
  # 2026-10-24 实测:第 3 列是 launched_at,不是 seat id,拿 $3==seatid 匹配全落空。
  # 所以改用席位名在会话名里做大小写不敏感的子串匹配:
  #   seat-flux-001 -> "flux" -> "Flux-Quality&Ops-seatloom"
  # 2026-10-24 另一次踩坑:推成 "-flux-seatloom$" 精确匹配不到,因为真实名字带 "-Quality&Ops"。
  while IFS='|' read -r sid m; do
    [ -z "$sid" ] && continue
    seatname="${sid#seat-}"; seatname="${seatname%-001}"
    sess="$(tmux list-sessions -F '#{session_name}' 2>/dev/null \
            | grep -i -- "$seatname" | head -1)"
    if [ -z "$sess" ]; then
      say "  $sid: 找不到含 '$seatname' 的 tmux 会话,跳过"
      continue
    fi
    if ! tmux has-session -t "$sess" 2>/dev/null; then
      say "  $sid ($sess): tmux 会话不存在,跳过"
      continue
    fi
    if tmux capture-pane -p -t "$sess" 2>/dev/null \
         | grep -qE '^[^ ]? ?[A-Z][a-z]+(ing|ed)… \([0-9]+[ms]'; then
      say "  $sid ($sess): 🔴 忙,跳过 —— 不打断在飞的活"
      continue
    fi
    # Aegis 不走 seat-launch.sh(它的 cwd 是主检出,且 session 由自己管)
    if [ "$seatname" = "aegis" ]; then
      say "  $sid ($sess): 跳过 —— Aegis 请用 scripts/restart-aegis.sh"
      continue
    fi
    say "  $sid ($sess): 重启 ..."
    SEATLOOM_PI_BIN="$HOME/Documents/GitHub/seatloom/saih/bin/saih" \
      bash "$REPO/scripts/seat-launch.sh" "$sess" 2>&1 | sed 's/^/      /'
  done <<< "$CURRENT"
fi

hr
say "完成。"
