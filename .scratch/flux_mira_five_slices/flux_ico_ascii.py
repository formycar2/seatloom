"""Extract a PNG or BMP entry from an .ico and render it as ASCII (dominant colour)."""
import struct
import sys
import zlib

sys.path.insert(0, "/tmp")
from flux_png_ascii2 import read_png  # noqa: E402


def ico_entries(path):
    d = open(path, "rb").read()
    n = struct.unpack("<H", d[4:6])[0]
    off = 6
    out = []
    for _ in range(n):
        w = d[off] or 256
        h = d[off + 1] or 256
        size = struct.unpack("<I", d[off + 8:off + 12])[0]
        start = struct.unpack("<I", d[off + 12:off + 16])[0]
        out.append((w, h, start, size, d[start:start + size]))
        off += 16
    return out


def render_rect(px, w, h, cols):
    for oy in range(h):
        cells = []
        for ox in range(cols):
            x0 = ox * w // cols
            x1 = max(x0 + 1, (ox + 1) * w // cols)
            nw = nb = nt = 0
            for y in range(oy, oy + 1):
                for x in range(x0, x1):
                    r, g, b, a = px[y][x]
                    if a < 40:
                        nt += 1
                    elif r > 200 and g > 200 and b > 200:
                        nw += 1
                    else:
                        nb += 1
            tot = nw + nb + nt
            if tot == 0:
                cells.append(" ")
            elif nw * 2 > tot:
                cells.append("#")
            elif nt * 2 > tot:
                cells.append(" ")
            else:
                cells.append(".")
        print("".join(cells))


def bmp_from_ico(data, w, h):
    # ICO BMP: BITMAPINFOHEADER, then XOR mask (bottom-up BGRA), then AND mask
    hdr = struct.unpack("<IiiHHIIiiII", data[:40])
    bpp = hdr[4]
    assert bpp == 32, "expected 32bpp, got %d" % bpp
    stride = w * 4
    xor = data[40:40 + stride * h]
    px = [[None] * w for _ in range(h)]
    for y in range(h):
        src = (h - 1 - y) * stride  # bottom-up
        for x in range(w):
            b, g, r, a = xor[src + x * 4:src + x * 4 + 4]
            px[y][x] = (r, g, b, a)
    return px


def main():
    path = sys.argv[1]
    want = int(sys.argv[2]) if len(sys.argv) > 2 else 16
    for w, h, start, size, data in ico_entries(path):
        if w != want:
            continue
        print("=== %s entry %dx%d (%d bytes) ===" % (path, w, h, size))
        if data[:8] == b"\x89PNG\r\n\x1a\n":
            open("/tmp/_ico_entry.png", "wb").write(data)
            pw, ph, px = read_png("/tmp/_ico_entry.png")
            render_rect(px, pw, ph, want)
        else:
            px = bmp_from_ico(data, w, h)
            render_rect(px, w, h, want)
        return
    print("no %dx%d entry in %s" % (want, want, path))


main()
