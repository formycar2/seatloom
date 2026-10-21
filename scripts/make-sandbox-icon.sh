#!/usr/bin/env bash
# make-sandbox-icon.sh — 把 SeatLoom 图标改成「沙箱版」:一眼能分辨的那张脸。
#
# 为什么需要它(2026-10-21 张先生的要求):
#   「能否给所有沙箱启动的页面有个显著的区别,加上每个实例版本区别。」
#
#   事实:沙箱装好后图标与主版**完全一样**(同一份 118KB icns)。
#   两个窗口在 macOS 的切换器/Cmd-Tab 里长得一模一样,**点错就是真事故** ——
#   他会以为自己点的是日常那份,实际点开了带未合并改动的沙箱。
#   一个「看起来一样」的沙箱,比没有沙箱更糟。
#
# 做什么:
#   1. 四边压一道橙色框 + 右上角一个橙色实心方块(颜色与主版的蓝/青区分开)
#   2. 整套重打成 icns,替进沙箱包
#   3. 换 Dock 缓存,否则要重启 Finder 才刷新
#
# 用什么做的:机器上没有 PIL / sharp / canvas(gen-app-icon.sh 里记着),
# 所以 PNG 的栅格化和编码自己写(下面那个 python 函数),再用 sips 缩放各尺寸,
# 最后由 iconutil 打成 .icns。
#
# 用法: bash scripts/make-sandbox-icon.sh <沙箱 .app 路径>
#    默认 dry-run:只报会改什么。
#    --apply  真的改。
set -uo pipefail

say(){ printf '%s\n' "$*"; }
line(){ say "────────────────────────────────────────────────────────────"; }
fail(){ say ""; say "❌ $*"; exit 1; }

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ICONS="$REPO/src-tauri/icons"
SAND_APP=""; APPLY=0
for a in "$@"; do
  case "$a" in
    --apply) APPLY=1;;
    *) [ -z "$SAND_APP" ] && SAND_APP="$a";;
  esac
done
[ -n "$SAND_APP" ] || fail "要给沙箱 .app 路径,例如 /Applications/SeatLoom-Sandbox.app"
[ -d "$SAND_APP" ] || fail "找不到 $SAND_APP"
SAND_ICNS="$SAND_APP/Contents/Resources/icon.icns"
[ -e "$SAND_ICNS" ] || fail "沙箱里没有 $SAND_ICNS —— 它可能不是用 prepare-sandbox-build.sh 装的"

# 基线图标:用仓库里最大的那张(设计源在 scripts/app-icon/)
SRC="$ICONS/128x128@2x.png"
[ -r "$SRC" ] || SRC="$ICONS/128x128.png"
[ -r "$SRC" ] || fail "找不到基线图标(试过 $ICONS/128x128@2x.png 和 128x128.png)"

line
if [ "$APPLY" = "1" ]; then say "沙箱图标 — 应用模式"; else say "沙箱图标 — 预演模式(只报不改)"; fi
line
say "  沙箱      : $SAND_APP"
say "  基线图标  : $SRC"
say "  要改成    : 橙色外框 + 右上角橙色方块,与主版蓝青区分开"
say "  目标      : $SAND_ICNS"
say "  主版图标  : $([ -e /Applications/SeatLoom.app/Contents/Resources/icon.icns ] && ls -la /Applications/SeatLoom.app/Contents/Resources/icon.icns | awk '{print $5" 字节"}' || echo '(主版未装在 /Applications 下,跳过对比)')"

[ "$APPLY" = "1" ] || { say ""; say "预演到此为止。要真改加 --apply。"; exit 0; }

WORK="$(mktemp -d)"
# 失败时把临时目录留着 —— 不然一出错就把现场删了,什么都查不到
# (2026-10-21 因为这个 trap,我反复用被 sed 改坏的副本去推断原脚本的行为)。
cleanup() { [ "${KEEP_WORK:-0}" = "1" ] || rm -rf "$WORK"; }
trap cleanup EXIT
cp "$SRC" "$WORK/base.png"

# ── 自己写 PNG 编码(与 gen-app-icon.sh 同一套理由:这台机器没有 PIL)──
python3 - "$WORK" <<'PY'
import zlib, struct, sys
work = sys.argv[1]
def png(path, w, h, rgb):
    raw = b''.join(b'\x00' + bytes(rgb) * w for _ in range(h))
    def chunk(t, d):
        c = t + d
        return struct.pack('>I', len(d)) + c + struct.pack('>I', zlib.crc32(c) & 0xffffffff)
    hdr = struct.pack('>IIBBBBB', w, h, 8, 2, 0, 0, 0)
    open(path, 'wb').write(
        b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', hdr)
        + chunk(b'IDAT', zlib.compress(raw)) + chunk(b'IEND', b'')
    )
# 橙:和 SeatLoom 的蓝/青分开,即使灰度打印也能看出「这不 normed」
png(f'{work}/band.png', 512, 96, (245, 158, 11))
print("band.png ok")
PY
# ── 从 BMP 读基线像素,自己合成(下面那个 python 段)──
BASE_H="$(sips -g pixelHeight "$WORK/base.png" 2>/dev/null | awk '/pixelHeight/{print $2}')"
[ -n "${BASE_H:-}" ] || BASE_H=256
say "  基线高度  : $BASE_H"

python3 - "$WORK" "${BASE_H}" <<'PY'
import zlib, struct, sys, subprocess
work, base_h = sys.argv[1], int(sys.argv[2])
# 读基线 PNG 的像素(sips 转成无压缩 BMP 中间格式最稳)
subprocess.run(['sips','-s','format','bmp',f'{work}/base.png','--out',f'{work}/base.bmp'],
               check=True, capture_output=True)
data = open(f'{work}/base.bmp','rb').read()
# BMP: 文件头 14 字节,然后是 DIB 头;从 DIB 头拿宽高
w = struct.unpack('<i', data[18:22])[0]
h_raw = struct.unpack('<i', data[22:26])[0]
bpp = struct.unpack('<H', data[28:30])[0]
off = struct.unpack('<I', data[10:14])[0]
px = data[off:]
topdown = h_raw < 0
h = abs(h_raw)
step = bpp // 8
row = (w * step + 3) // 4 * 4
if len(px) < row * h:
    raise SystemExit(f'BMP 像素数据不足:{len(px)} < {row*h}')
def get(x, y):
    yy = y if topdown else (h - 1 - y)
    i = yy * row + x * step
    b, g, r = px[i], px[i+1], px[i+2]
    return (r, g, b, 255)
# icns 必须正方形。基线是 256x256(源图标本身正方形),这里按 w 输出,
# 并把 h/<差补透明>,最后建一道硬核对。
SQ = max(w, h)

out = bytearray()
# 2026-10-21 实测:底部加色带会产出 256x352 这种**非正方形** PNG,
# 而 .icns 要求正方形 —— iconutil 对它只报一句「Failed to generate ICNS」
# 不说原因,而 sips -Z 按最大边缩放,于是整个 iconset 全变成非正方形。
# 修法:**保持正方形**。区别改用在正方形内能表达的形式:
# 四边各压一道橙框 + 右上角一个橙色实心方块(「沙箱记号」),
# 缩到 16x16 时是一个带橙角的小方块,仍然一眼可分辨。
FRAME = max(4, w // 16)
BOX = max(4, w // 5)
for y in range(SQ):
    out += b'\x00'
    for x in range(SQ):
        if y < h and x < w:
            r_, g_, b_, a_ = get(x, y)
            in_box = x >= w - FRAME - BOX and x < w - FRAME and y < FRAME + BOX
            in_frame = (y < FRAME or y >= h - FRAME or x < FRAME or x >= w - FRAME)
            if in_box:
                out += bytes((245, 158, 11, 255))      # 右上角橙块:沙箱记号
            elif in_frame:
                out += bytes((245, 158, 11, 255))      # 橙框
            else:
                out += bytes((r_, g_, b_, a_))
        else:
            # 正方形外的补白,填透明,不抢戏
            out += bytes((0, 0, 0, 0))
def chunk(t, d):
    c = t + d
    return struct.pack('>I', len(d)) + c + struct.pack('>I', zlib.crc32(c) & 0xffffffff)
hdr = struct.pack('>IIBBBBB', SQ, SQ, 8, 6, 0, 0, 0)
open(f'{work}/sandbox_1024.png','wb').write(
    b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', hdr)
    + chunk(b'IDAT', zlib.compress(bytes(out))) + chunk(b'IEND', b'')
)
print(f'sandbox_1024.png {SQ}x{SQ} ok')
PY
[ -s "$WORK/sandbox_1024.png" ] || fail "合成图没造出来"

# ── 出全套尺寸并打成 icns ──
# iconutil 要的是**完整**命名集,少一个就静默失败(2026-10-21 实测:只给
# 一张 16x16 时它一声不响地返回非零)。所以逐个文件名列清楚,并且产完核对齐全。
IS="$WORK/icon.iconset"; mkdir -p "$IS"
declare -a SPEC=(
  "icon_16x16.png:16"
  "icon_16x16@2x.png:32"
  "icon_32x32.png:32"
  "icon_32x32@2x.png:64"
  "icon_128x128.png:128"
  "icon_128x128@2x.png:256"
  "icon_256x256.png:256"
  "icon_256x256@2x.png:512"
  "icon_512x512.png:512"
  "icon_512x512@2x.png:1024"
)
for entry in "${SPEC[@]}"; do
  name="${entry%%:*}"; size="${entry##*:}"
  sips -Z "$size" "$WORK/sandbox_1024.png" --out "$IS/$name" >/dev/null 2>&1 \
    || fail "产 $name 失败"
done
# 核对齐全:一个不少才许打
for entry in "${SPEC[@]}"; do
  name="${entry%%:*}"
  [ -s "$IS/$name" ] || fail "尺寸集缺 $name —— iconutil 会静默失败,所以在打之前就拦住"
done
say "  尺寸集      : ${#SPEC[@]} 个文件,齐全"
iconutil -c icns "$IS" -o "$WORK/icon.icns" || fail "iconutil 打 icns 失败"
[ -s "$WORK/icon.icns" ] || fail "icns 是空的"

cp "$WORK/icon.icns" "$SAND_ICNS" || fail "写不进 $SAND_ICNS"
# 换 Dock/访达缓存,否则图标不刷新
touch "$SAND_APP" "$SAND_APP/Contents/Info.plist" 2>/dev/null || true
line; say "结果"; line
say "  ✅ 沙箱图标已换:$SAND_ICNS"
say "  ✅ 与主版同一设计,但四边是橙框、右上角有橙块 —— 灰度打印也分得清"
say ""
say "  接着做第二层区别(窗口里那条版本标识条):那要改前端代码,已单独派单。"
say "  图标只解决 Dock 和 Cmd-Tab 认得出;窗口内还得看得见,否则打开之后又分不清。"
exit 0
