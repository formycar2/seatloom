#!/usr/bin/env bash
# rename-stepplan-providers.sh — 把 seats.default_model 里的旧 provider 名换成新名。
#
# 背景(2026-11-02,张先生指示):
#   stepplan / stepplan-anthropic / stepplan-wash / stepplan-wash-anthropic
#   合并成两个 anthropic-only provider:
#     stepplan + stepplan-anthropic        -> sp-personal
#     stepplan-wash + stepplan-wash-anthropic -> sp-team
#   理由:同 key 下两种 API 的模型列表完全等价,而 anthropic 的缓存更省
#   (实测同一 19K prompt:anthropic 第 2 次起 input 仅 467 tokens,
#    openai 仍按 15,315 计费)。
#
# 🔴 这是改 live 数据库的账本行(seats.default_model),属于被权限封顶的动作。
#    「一个能改席位登记模型的 agent 不该由自己执行」—— 所以给可审计、可重复跑的
#    脚本,由张先生执行。也不让席位代跑:那是把被拒的动作外包,同一个漏洞换个方向。
#
# 安全设计:
#   1. 预演(默认):只打印改前改后,不动库
#   2. --apply:在**一个事务**里改完,要么全改要么全不改
#   3. 改前校验新 provider/model 在 ~/.saih/agent/models.json 里真实存在
#   4. 改后打印对照,并列出仍有旧名的行(应为 0)
#
# 用法:
#   bash scripts/rename-stepplan-providers.sh                # 预演
#   bash scripts/rename-stepplan-providers.sh --apply        # 真改
#   bash scripts/rename-stepplan-providers.sh --apply --restart  # 改完重启空闲席位
set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PG="${SEATLOOM_PG_CONTAINER:-seatloom-postgres}"
# 🔴 2026-11-02 修:saih 读 .saih/,不是 .pi/。旧路径会让检查 A 永远失败。
MODELS_JSON="${HOME}/.saih/agent/models.json"

APPLY=0; RESTART=0
for a in "$@"; do
  case "$a" in
    --apply)   APPLY=1 ;;
    --restart) RESTART=1 ;;
    -h|--help) sed -n '2,26p' "$0"; exit 0 ;;
    *) echo "unknown option: $a" >&2; exit 2 ;;
  esac
done

say() { printf '%s\n' "$*"; }
hr()  { printf '%.0s─' {1..74}; printf '\n'; }

# ── 检查 A:新 provider/model 必须在 models.json 里存在 ─────────────────────
if [ ! -f "$MODELS_JSON" ]; then
  say "❌ 找不到 $MODELS_JSON —— 无法校验新名字。停。"
  exit 1
fi

for TARGET in "sp-personal/step-5-preview" "sp-team/step-5-preview"; do
  if ! python3 - "$MODELS_JSON" "$TARGET" <<'PY'
import json, sys
path, target = sys.argv[1], sys.argv[2]
prov, _, mid = target.partition("/")
d = json.load(open(path))
for p, v in d.get("providers", {}).items():
    if p == prov and any(m.get("id") == mid for m in v.get("models", [])):
        sys.exit(0)
sys.exit(1)
PY
  then
    say "❌ 目标模型 $TARGET 不在 $MODELS_JSON 里 —— 改过去席位起不来。停。"
    exit 1
  fi
done
say "✅ 检查 A 通过:sp-personal/step-5-preview 与 sp-team/step-5-preview 都在配置里"

# ── 检查 B:contextWindow 必须够(否则 reserveTokens 会把阈值压成负数) ────────
# 取两个 provider 里较小的那个:合并后席位可能落在任一个上。
CW="$(python3 - "$MODELS_JSON" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
windows = [
    m.get("contextWindow", 0)
    for p in ("sp-personal", "sp-team")
    for m in d["providers"][p]["models"]
    if m["id"] == "step-5-preview"
]
print(min(windows) if windows else 0)
PY
)"
CW="${CW//[[:space:]]/}"
if ! [[ "${CW:-0}" =~ ^[0-9]+$ ]]; then
  say "❌ 检查 B 失败:contextWindow 解析异常('${CW}')。停。"
  exit 1
fi
if [ "$CW" -lt 600000 ]; then
  say "❌ 检查 B 失败:sp-*/step-5-preview 的最小 contextWindow=${CW},小于 reserveTokens 安全余量。停。"
  exit 1
fi
say "✅ 检查 B 通过:最小 contextWindow=${CW}"

hr
say "将要执行的 UPDATE(单事务):"
say "  stepplan/step-5-preview            -> sp-personal/step-5-preview"
say "  stepplan-anthropic/step-5-preview  -> sp-personal/step-5-preview"
say "  stepplan-wash/step-5-preview       -> sp-team/step-5-preview"
say "  stepplan-wash-anthropic/step-5-preview -> sp-team/step-5-preview"
hr
say "改前状态:"
podman exec -e PGPASSWORD=seatloom "$PG" \
  psql -U seatloom -d seatloom -tAc \
  "select id, default_model from seats where default_model like 'stepplan%' order by id" | sed 's/^/  /'

if [ "$APPLY" -ne 1 ]; then
  hr
  say "预演结束,未改任何数据。加 --apply 真改。"
  exit 0
fi

hr
say "执行 UPDATE ..."
podman exec -e PGPASSWORD=seatloom "$PG" psql -U seatloom -d seatloom -tAc "
BEGIN;
UPDATE seats SET default_model = 'sp-personal/step-5-preview'
  WHERE default_model IN ('stepplan/step-5-preview','stepplan-anthropic/step-5-preview');
UPDATE seats SET default_model = 'sp-team/step-5-preview'
  WHERE default_model IN ('stepplan-wash/step-5-preview','stepplan-wash-anthropic/step-5-preview');
COMMIT;" >/dev/null

say "改后状态(应无 stepplan 残留):"
podman exec -e PGPASSWORD=seatloom "$PG" \
  psql -U seatloom -d seatloom -tAc \
  "select id, default_model from seats order by id" | sed 's/^/  /'

LEFT="$(podman exec -e PGPASSWORD=seatloom "$PG" \
  psql -U seatloom -d seatloom -tAc \
  "select count(*) from seats where default_model like 'stepplan%'" | tr -d ' ')"
if [ "${LEFT}" = "0" ]; then
  say "✅ 旧 provider 名已清零"
else
  say "⚠️ 仍有 ${LEFT} 行引用旧名,需人工检查"
fi

if [ "$RESTART" -eq 1 ]; then
  hr
  say "重启空闲席位(忙的会跳过)..."
  cd "$REPO"
  SEATLOOM_PI_BIN="$HOME/Documents/GitHub/seatloom/saih/bin/saih" \
    bash scripts/seat-launch.sh --restart-idle 2>&1 | sed 's/^/  /'
fi
