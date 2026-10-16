"""Render a PNG as ASCII by DOMINANT colour per cell, so white-on-blue marks are legible.

Pure-python PNG decode (zlib + unfilter) — no PIL on this machine.
Usage: python3 png_ascii2.py <file.png> [--cols N]
Legend: '#' = white (the mark), '.' = brand blue (the ground), ' ' = transparent.
"""
import struct
import sys
import zlib


def read_png(path):
    d = open(path, "rb").read()
    assert d[:8] == b"\x89PNG\r\n\x1a\n", "not a PNG"
    off = 8
    idat = b""
    pal = None
    trns = None
    while off < len(d):
        ln = struct.unpack(">I", d[off:off + 4])[0]
        typ = d[off + 4:off + 8]
        data = d[off + 8:off + 8 + ln]
        if typ == b"IHDR":
            w, h, depth, ctype, comp, filt, inter = struct.unpack(">IIBBBBB", data)
        elif typ == b"IDAT":
            idat += data
        elif typ == b"PLTE":
            pal = [tuple(data[i:i + 3]) for i in range(0, len(data), 3)]
        elif typ == b"tRNS":
            trns = data
        off += 12 + ln
    raw = zlib.decompress(idat)
    channels = {0: 1, 2: 3, 3: 1, 4: 2, 6: 4}[ctype]
    assert depth == 8 and inter == 0
    bpp = channels
    stride = w * bpp
    rows = []
    prev = bytearray(stride)
    pos = 0
    for y in range(h):
        f = raw[pos]
        pos += 1
        line = bytearray(raw[pos:pos + stride])
        pos += stride
        for i in range(stride):
            a = line[i - bpp] if i >= bpp else 0
            b = prev[i]
            c = prev[i - bpp] if i >= bpp else 0
            if f == 1:
                line[i] = (line[i] + a) & 0xFF
            elif f == 2:
                line[i] = (line[i] + b) & 0xFF
            elif f == 3:
                line[i] = (line[i] + ((a + b) >> 1)) & 0xFF
            elif f == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[i] = (line[i] + pr) & 0xFF
        rows.append(bytes(line))
        prev = line
    px = []
    for y in range(h):
        row = []
        base_row = rows[y]
        for x in range(w):
            base = x * bpp
            if ctype == 2:
                row.append((base_row[base], base_row[base + 1], base_row[base + 2], 255))
            elif ctype == 6:
                row.append((base_row[base], base_row[base + 1], base_row[base + 2], base_row[base + 3]))
            elif ctype == 0:
                g = base_row[base]
                row.append((g, g, g, 255))
            elif ctype == 4:
                g = base_row[base]
                row.append((g, g, g, base_row[base + 1]))
            else:
                r, g, b = pal[base_row[base]]
                a = trns[base_row[base]] if trns and base_row[base] < len(trns) else 255
                row.append((r, g, b, a))
        px.append(row)
    return w, h, px


def render(path, cols):
    w, h, px = read_png(path)
    print("%s  %dx%d  (legend: # = white mark, . = brand blue, ' ' = transparent)" % (path, w, h))
    for oy in range(h):
        cells = []
        for ox in range(cols):
            x0 = ox * w // cols
            x1 = max(x0 + 1, (ox + 1) * w // cols)
            y0 = oy * h // h
            y1 = max(y0 + 1, (oy + 1) * h // h)
            nw = nb = nt = 0
            for y in range(y0, y1):
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
    op = sum(1 for y in range(h) for x in range(w) if px[y][x][3] > 8)
    print("opaque pixels: %d / %d (%.1f%%)" % (op, w * h, 100.0 * op / (w * h)))


if __name__ == "__main__":
    path = sys.argv[1]
    cols = 48
    for i, a in enumerate(sys.argv):
        if a == "--cols":
            cols = int(sys.argv[i + 1])
    render(path, cols)
