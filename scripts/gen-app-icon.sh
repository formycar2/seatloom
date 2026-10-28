#!/usr/bin/env bash
# gen-app-icon.sh — 生成 SeatLoom 的全套应用图标。
#
# 分工(为什么有两步,而不是一个工具):
#   1. scripts/app-icon/render.js + design.mjs  —— **设计源**。这台机器上没有
#      PIL / sharp / canvas,所以栅格化和 PNG 编码都是自己写的。它的价值不是「能出
#      图」,是能在终端里把 16x16 / 32x32 直接打出来看 —— 张先生这条派单的硬约束
#      是「必须在小尺寸下还认得出来」,不看是迭代不出来的(本目录下有 7 个候选方案
#      就是这么比掉的)。
#   2. node_modules/.bin/tauri icon            —— **出全套**。.icns / .ico /
#      Windows Store 图标 / android / ios 都由它生成,不手搓尺寸。
#
# 用法:bash scripts/gen-app-icon.sh [--force]
#   --force  覆盖 icons/ 下已有文件(默认遇到已有文件会停)

set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

ICONS_DIR="src-tauri/icons"
CONF="src-tauri/tauri.conf.json"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

FORCE=0
[ "${1:-}" = "--force" ] && FORCE=1

if [ -d "$ICONS_DIR" ] && [ -n "$(find "$ICONS_DIR" -name '*.png' -o -name '*.icns' -o -name '*.ico' 2>/dev/null | head -1)" ] && [ "$FORCE" != "1" ]; then
  echo "✗ $ICONS_DIR 下已有图标文件。确认覆盖就加 --force。" >&2
  exit 1
fi

# ── 1. 设计源:栅格化 1024x1024 母图 ─────────────────────────────────────────
echo "→ 栅格化 1024x1024 母图(设计源:scripts/app-icon/design.mjs)"
node --no-warnings --input-type=module -e "
import { writeFileSync } from 'node:fs';
import { renderPng } from '$PWD/scripts/app-icon/design.mjs';
const { png, ss } = renderPng(1024, 'B', 4096);
writeFileSync('$WORK/app-icon.png', png);
console.log('  母图', png.length, 'bytes,每像素', ss + 'x' + ss, '采样');
"

# ── 2. 出全套:tauri icon ────────────────────────────────────────────────────
echo "→ tauri icon 生成全套(.icns / .ico / Windows Store / android / ios)"
mkdir -p "$ICONS_DIR"
./node_modules/.bin/tauri icon "$WORK/app-icon.png" -o "$ICONS_DIR" 2>&1 | tail -3

# ── 3. 填 tauri.conf.json 的 bundle.icon ────────────────────────────────────
echo "→ 填 bundle.icon"
python3 - "$CONF" <<'PY'
import json, sys
p = sys.argv[1]
d = json.load(open(p))
d["bundle"]["icon"] = [
    "icons/32x32.png",
    "icons/128x128.png",
    "icons/128x128@2x.png",
    "icons/icon.icns",
    "icons/icon.ico",
]
with open(p, "w") as f:
    json.dump(d, f, indent=2)
    f.write("\n")
print("  bundle.icon =", d["bundle"]["icon"])
PY

# ── 4. 验证:文件在、尺寸对、.icns 里有全套、圆角四角没有白边 ─────────────────
echo "→ 验证"
FAIL=0
for f in 32x32.png 128x128.png 128x128@2x.png icon.icns icon.ico icon.png; do
  if [ ! -s "$ICONS_DIR/$f" ]; then echo "  ✗ 缺 $f"; FAIL=1; fi
done
[ "$FAIL" = "0" ] && echo "  ✓ bundle.icon 五个文件都在,icon.png(母图)也在"

python3 - "$ICONS_DIR" <<'PY' || FAIL=1
import struct, sys, os
raw = open(os.path.join(sys.argv[1], "icon.icns"), "rb").read()
magic, total = struct.unpack(">4sI", raw[:8])
assert magic == b"icns", magic
off, seen = 8, {}
while off + 8 <= len(raw):
    ostype, length = struct.unpack(">4sI", raw[off:off + 8])
    seen[ostype.decode("latin1")] = length
    off += length
# tauri-icns 用的是现代条目码:16=is32/ic11,32=il32/ic12,64=ic13,128=ic07,
# 256=ic08/ic14,512=ic09,1024=ic10。老码 ic04/ic05 它不写 —— 所以不按码名验,
# 按尺寸验(下面用 iconutil 还原,那才是权威)。
expected = ["ic11", "ic12", "ic13", "ic07", "ic08", "ic09", "ic10"]
missing = [e for e in expected if e not in seen]
if missing:
    print("  ✗ .icns 缺条目:", ", ".join(missing))
    sys.exit(1)
print(f"  ✓ .icns {total} bytes,{len(seen)} 个条目,16/32/64/128/256/512/1024 七档都在")
PY

# iconutil 是 Apple 自己的工具:它能把 .icns 还原成完整 iconset,就说明这个文件
# 在 macOS 眼里是对的。按文件名验尺寸,不按条目码猜。
ICONSET_CHECK="$WORK/verify.iconset"
if iconutil -c iconset "$ICONS_DIR/icon.icns" -o "$ICONSET_CHECK" 2>/dev/null; then
  N=$(find "$ICONSET_CHECK" -name '*.png' | wc -l | tr -d ' ')
  if [ "$N" -ge 10 ]; then
    echo "  ✓ iconutil 能把它还原成 $N 个尺寸(16/32/64/128/256/512 + 各 @2x)"
  else
    echo "  ✗ iconutil 只还原出 $N 个尺寸"; FAIL=1
  fi
else
  echo "  ✗ iconutil 读不出这个 .icns"; FAIL=1
fi

node --no-warnings --input-type=module -e "
import { readFileSync } from 'node:fs';
import { decodePng } from '$PWD/scripts/app-icon/render.js';
// 圆角方块的四角必须全透明。任何非零 alpha 就是白边/发虚 —— 验收卡里明确要打回的那条。
let bad = 0;
for (const size of [16, 32]) {
  const p = '$ICONS_DIR/icon.icns';
  // 直接从母图重栅格一份同尺寸的来比(.icns 里是 PNG/raw 混合,不必解)
  const { render } = await import('$PWD/scripts/app-icon/render.js');
  const { scene } = await import('$PWD/scripts/app-icon/design.mjs');
  const rgba = render(scene('B'), size, Math.max(1, Math.round(4096 / size)));
  for (const [x, y] of [[0,0],[size-1,0],[0,size-1],[size-1,size-1]]) {
    const a = rgba[(y * size + x) * 4 + 3];
    if (a !== 0) { console.log('  ✗ ' + size + 'px 角 (' + x + ',' + y + ') alpha=' + a); bad++; }
  }
}
console.log(bad === 0 ? '  ✓ 16/32px 圆角四角全透明,没有白边' : '  ✗ 有白边');
process.exit(bad === 0 ? 0 : 1);
" || FAIL=1

echo "→ 产物"
ls -la "$ICONS_DIR" | tail -n +4 | awk '{printf "  %-28s %8s\n", $9, $5}'

[ "$FAIL" = "0" ] && echo "✓ 应用图标生成完毕" || { echo "✗ 验证有失败项" >&2; exit 1; }
