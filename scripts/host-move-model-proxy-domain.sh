#!/usr/bin/env bash
# host-move-model-proxy-domain.sh — 把 stepgo 的模型代理域名从
#   models-proxy.stepfun-inc.com  迁到  mp.z01.normandyfunc.com
#
# 设计文档:docs/coordination/tasks/lyra/LYRA-2026-10-15-model-proxy-domain-migration-v1.md
#
# 用法:
#   ./host-move-model-proxy-domain.sh                    # 只改活动配置(默认,安全)
#   ./host-move-model-proxy-domain.sh --include-bundle   # 同时改装机程序里的 host 白名单/默认值
#   ./host-move-model-proxy-domain.sh --dry-run          # 只报告会改什么,什么都不动
#   ./host-move-model-proxy-domain.sh --rollback         # 把 .bak-hostmove-* 拷回原名
#
# 幂等:重复跑第二次是 no-op(已经替换过就没有旧域名可换)。
# 备份:每个被改的文件第一次被改前存一份 .bak-hostmove-<TS>。
#
# 🔴 bundle(装机程序)默认不碰 —— 改第三方安装代码、且升级会被覆盖,必须是显式决定。
# 🔴 绝不碰:logs/ sessions/ shadow-eval/ *.bak-* *.backup-* *.legacy-disabled-*
#    cache/(派生数据)~/.stepcode/(已确认死的旧目录)
#
# 兼容 macOS 自带的 bash 3.2(不用 nameref / mapfile)。
set -euo pipefail

OLD="models-proxy.stepfun-inc.com"
NEW="mp.z01.normandyfunc.com"
TS="$(date +%Y%m%d-%H%M%S)"
STEPGO="${STEPGO_HOME:-${STEPCODE_HOME:-$HOME/.stepgo}}"
PI_MODELS="$HOME/.pi/agent/models.json"

DRY=0; WITH_BUNDLE=0; ROLLBACK=0
for a in "$@"; do
  case "$a" in
    --dry-run)       DRY=1 ;;
    --include-bundle) WITH_BUNDLE=1 ;;
    --rollback)      ROLLBACK=1 ;;
    *) echo "未知参数:$a" >&2; exit 2 ;;
  esac
done

# ── 含旧域名的文件清单(macOS bash 3.2 兼容,不用 nameref)────────────────────
list_live_config() {
  for f in "$STEPGO/config/config.json" \
           "$STEPGO/config/agent/managed/config.json" \
           "$STEPGO/agents/cursor/config.json" \
           "$STEPGO"/agents/claude/launch/*/settings.json \
           "$PI_MODELS"; do
    [ -f "$f" ] && grep -q "$OLD" "$f" 2>/dev/null && echo "$f"
  done
  return 0
}
list_bundle() {
  for f in "$STEPGO/runtime/host/dist/QZGQPGY3.js" \
           "$STEPGO/runtime/host/dist/VJN2XLCC.js" \
           "$STEPGO/runtime/host/dist/SUL6KV6A.js"; do
    [ -f "$f" ] && grep -q "$OLD" "$f" 2>/dev/null && echo "$f"
  done
  return 0
}
list_all() { { list_live_config; [ "$WITH_BUNDLE" = 1 ] && list_bundle; } | sort -u; return 0; }

if [ "$ROLLBACK" = 1 ]; then
  echo "== 回滚:把 .bak-hostmove-* 拷回原名 =="
  n=0
  for bak in $(find "$STEPGO" "$HOME/.pi/agent" -name '*.bak-hostmove-*' -type f 2>/dev/null | sort); do
    orig="${bak%.bak-hostmove-*}"
    cp -p "$bak" "$orig" && echo "  restored  ${orig#$HOME/}" && n=$((n+1))
  done
  echo "回滚 $n 个文件。"
  [ "$n" -gt 0 ] && echo "提醒:claude code 的会话要重启才会重新读到旧域名。"
  exit 0
fi

echo "== 迁移 $OLD  ->  $NEW =="
[ "$WITH_BUNDLE" = 1 ] || echo "   (默认模式:只改活动配置;加 --include-bundle 才改装机程序)"
[ "$DRY" = 1 ] && echo "   *** DRY RUN —— 什么都不改 ***"

changed=0
for f in $(list_all); do
  n="$(grep -c "$OLD" "$f")"
  if [ "$DRY" = 1 ]; then
    printf '  would change  %-70s (%s 处)\n' "${f#$HOME/}" "$n"
  else
    cp -p "$f" "$f.bak-hostmove-$TS"
    sed -i '' "s/$OLD/$NEW/g" "$f"
    printf '  changed       %-70s (%s 处)\n' "${f#$HOME/}" "$n"
    changed=$((changed+1))
  fi
done
[ "$(list_all | wc -l | tr -d ' ')" = 0 ] && echo "  (无需改动 —— 已是新域名)"

echo
echo "== 校验 =="
leftover=$( { list_live_config; [ "$WITH_BUNDLE" = 1 ] && list_bundle; } | sort -u || true )
if [ -n "$leftover" ]; then
  echo "  ⚠ 以下文件仍含旧域名:"; echo "$leftover" | sed 's|^|      |'
else
  echo "  ✓ 已检查范围里旧域名清零"
fi

if command -v stepgo >/dev/null 2>&1; then
  eff="$(stepgo config get baseUrl 2>/dev/null | grep -o 'https\?://.*' | head -1 | tr -d '\r' || true)"
  eff_host="${eff#*://}"; eff_host="${eff_host%%/*}"
  echo "  stepgo config get baseUrl -> ${eff:-<取不到>}  (host: ${eff_host:-<无>})"
  if [ -z "$eff" ]; then
    echo "  ⚠ 取不到生效值,请手动跑 stepgo config get baseUrl"
  elif [ "$eff_host" = "$NEW" ]; then
    echo "  ✓ 生效值是新域名"
  else
    echo "  ✗ 生效值不是新域名 —— 检查是否激活了 profile(见设计文档 §3.2)"
  fi
fi

if command -v curl >/dev/null 2>&1; then
  key="$(python3 - "$STEPGO/config/config.json" <<'PY' 2>/dev/null || true
import re,json,sys
try:
    d=json.loads(re.sub(r'^\s*//.*$','',open(sys.argv[1],encoding='utf-8').read(),flags=re.M))
    print(d.get('apiKey',''))
except Exception:
    pass
PY
)"
  if [ -n "$key" ]; then
    a=$(curl -sS --max-time 15 -H "Authorization: Bearer $key" "https://$OLD/v1/models?purpose=code" 2>/dev/null | head -c 40 || true)
    b=$(curl -sS --max-time 15 -H "Authorization: Bearer $key" "https://$NEW/v1/models?purpose=code" 2>/dev/null | head -c 40 || true)
    if [ -n "$a" ] && [ "$a" = "$b" ]; then
      echo "  ✓ 新旧 host 返回一致(同一后端)"
    else
      echo "  ⚠ 新旧 host 返回不一致或取不到,需人工看"
    fi
  fi
fi

echo
[ "$DRY" = 1 ] && echo "DRY RUN 结束,未改任何文件。" || echo "改了 $changed 个文件;备份后缀 .bak-hostmove-$TS"
echo "回滚:$0 --rollback"
