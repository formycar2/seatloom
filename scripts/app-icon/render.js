// SeatLoom 应用图标 —— 设计稿栅格化 + PNG 编码。
//
// 这个文件只做一件事:把一个用几何形状描述的设计,栅格成任意尺寸的 RGBA PNG。
// 它不调任何图片库(这台机器上没有 PIL / sharp / canvas),PNG 编码和 CRC32 都是
// 自己写的,zlib 用 node 自带的。尺寸不是逐个手搓的:每一个输出尺寸都从同一个
// 场景描述重新栅格,超采样数按输出尺寸反算,总采样量恒定。
//
// 为什么不用最近邻缩小:16x16 的程序坞图标和 Cmd+Tab 图标是这个图标最常被看到的
// 尺寸,点采样会把粗杠之间的蓝色缝隙糊掉,整个标记变成一团白。面积平均(这里是
// 超采样后求平均,等价于面积平均)才能让小尺寸下的结构活下来。

import zlib from 'node:zlib';

// ─────────────────────────────────────────────────────────────────────────────
// PNG 编码(8 位 RGBA,无交错,filter 0)
// ─────────────────────────────────────────────────────────────────────────────

const CRC_TABLE = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();

function crc32(buf) {
  let c = 0xffffffff;
  for (let i = 0; i < buf.length; i++) c = CRC_TABLE[(c ^ buf[i]) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function pngChunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const typeBuf = Buffer.from(type, 'ascii');
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([typeBuf, data])), 0);
  return Buffer.concat([len, typeBuf, data, crc]);
}

/** width × height 的 RGBA(每像素 4 字节)Buffer -> PNG Buffer。 */
export function encodePng(width, height, rgba) {
  const sig = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 6; // colour type: truecolour with alpha
  ihdr[10] = 0; // compression: deflate
  ihdr[11] = 0; // filter method 0
  ihdr[12] = 0; // no interlace
  const stride = width * 4;
  const raw = Buffer.alloc((stride + 1) * height);
  for (let y = 0; y < height; y++) {
    raw[y * (stride + 1)] = 0; // filter type None
    rgba.copy(raw, y * (stride + 1) + 1, y * stride, y * stride + stride);
  }
  const idat = zlib.deflateSync(raw, { level: 9 });
  return Buffer.concat([sig, pngChunk('IHDR', ihdr), pngChunk('IDAT', idat), pngChunk('IEND', Buffer.alloc(0))]);
}

// ─────────────────────────────────────────────────────────────────────────────
// 场景:一组画在单位方框(0..1)里的形状,按顺序做 alpha 合成
// ─────────────────────────────────────────────────────────────────────────────

/**
 * @typedef {Object} Shape
 * @property {'rect'|'disc'|'bar'|'tri'} type
 */

/** 圆角矩形。圆角外的方形部分也要算覆盖 —— 先算折到第一象限圆角方块的距离。 */
function rectCoverage(s, x, y) {
  const ax = Math.abs(x - s.cx);
  const ay = Math.abs(y - s.cy);
  if (ax > s.hw || ay > s.hh) return 0;
  const dx = ax - (s.hw - s.r);
  const dy = ay - (s.hh - s.r);
  if (dx <= 0 || dy <= 0) return 1; // 十字区:竖条或横条,直接算内
  return dx * dx + dy * dy <= s.r * s.r ? 1 : 0; // 只剩圆角方块要判
}

function discCoverage(s, x, y) {
  const dx = x - s.cx;
  const dy = y - s.cy;
  return dx * dx + dy * dy <= s.r * s.r ? 1 : 0;
}

/** 粗线段,两端圆头。 */
function barCoverage(s, x, y) {
  const ex = s.x1 - s.x0;
  const ey = s.y1 - s.y0;
  const len2 = ex * ex + ey * ey;
  let t = len2 === 0 ? 0 : ((x - s.x0) * ex + (y - s.y0) * ey) / len2;
  if (t < 0) t = 0;
  else if (t > 1) t = 1;
  const dx = x - (s.x0 + t * ex);
  const dy = y - (s.y0 + t * ey);
  return dx * dx + dy * dy <= s.hw * s.hw ? 1 : 0;
}

/** 三角形(用重心坐标判内外,顺带给出线性边缘覆盖,避免锯齿)。 */
function triCoverage(s, x, y) {
  const { ax, ay, bx, by, cx, cy } = s;
  const d = (by - cy) * (ax - cx) + (cx - bx) * (ay - cy);
  if (d === 0) return 0;
  const u = ((by - cy) * (x - cx) + (cx - bx) * (y - cy)) / d;
  const v = ((cy - ay) * (x - cx) + (ax - cx) * (y - cy)) / d;
  const w = 1 - u - v;
  return u >= 0 && v >= 0 && w >= 0 ? 1 : 0;
}

const COVERAGE = {
  rect: rectCoverage,
  disc: discCoverage,
  bar: barCoverage,
  tri: triCoverage,
};

/**
 * 把场景栅格成一张 RGBA Buffer。
 *
 * @param {Shape[]} scene 已按单位坐标描述的形状(颜色 [r,g,b] 0-255)
 * @param {number} size   输出边长(正方形)
 * @param {number} ss     每像素每边的采样数
 * @returns {Buffer}      size*size*4 字节
 */
export function render(scene, size, ss) {
  const rgba = Buffer.alloc(size * size * 4);
  const n = scene.length;
  // 形状参数按 size 预乘一次,内循环只做加减乘。
  const sc = scene.map((s) => ({ ...s, _s: size }));

  for (let py = 0; py < size; py++) {
    for (let px = 0; px < size; px++) {
      let r = 0;
      let g = 0;
      let b = 0;
      let a = 0;
      for (let sy = 0; sy < ss; sy++) {
        const y = (py + (sy + 0.5) / ss) / size;
        for (let sx = 0; sx < ss; sx++) {
          const x = (px + (sx + 0.5) / ss) / size;
          let cr = 0;
          let cg = 0;
          let cb = 0;
          let ca = 0;
          for (let i = 0; i < n; i++) {
            const s = sc[i];
            if (COVERAGE[s.type](s, x, y) > 0) {
              cr = s.color[0];
              cg = s.color[1];
              cb = s.color[2];
              ca = 255;
            }
          }
          r += cr;
          g += cg;
          b += cb;
          a += ca;
        }
      }
      const k = ss * ss;
      const o = (py * size + px) * 4;
      rgba[o] = Math.round(r / k);
      rgba[o + 1] = Math.round(g / k);
      rgba[o + 2] = Math.round(b / k);
      rgba[o + 3] = Math.round(a / k);
    }
  }
  return rgba;
}

/** 把一张 RGBA 图降成 ASCII,用来在终端里直接看小尺寸下还认不认得出来。 */
export function asciiArt(rgba, size, ramp = ' .:-=+*#%@') {
  const lines = [];
  for (let y = 0; y < size; y++) {
    let line = '';
    for (let x = 0; x < size; x++) {
      const o = (y * size + x) * 4;
      const a = rgba[o + 3] / 255;
      const lum = (rgba[o] * 0.299 + rgba[o + 1] * 0.587 + rgba[o + 2] * 0.114) / 255;
      const idx = Math.min(ramp.length - 1, Math.max(0, Math.round(a * lum * (ramp.length - 1))));
      line += ramp[idx];
    }
    lines.push(line);
  }
  return lines.join('\n');
}

// ─────────────────────────────────────────────────────────────────────────────
// PNG 解码(只解 8 位 RGBA / RGB,无交错)—— 给验证和对比用
// ─────────────────────────────────────────────────────────────────────────────

function paeth(a, b, c) {
  const p = a + b - c;
  const pa = Math.abs(p - a);
  const pb = Math.abs(p - b);
  const pc = Math.abs(p - c);
  if (pa <= pb && pa <= pc) return a;
  return pb <= pc ? b : c;
}

/** PNG Buffer -> {width, height, rgba}(rgba 每像素 4 字节)。
 *  支持 filter 0-4、8 位、colour type 2(RGB)与 6(RGBA)、无交错。 */
export function decodePng(buf) {
  let off = 8; // 跳过签名
  let width = 0;
  let height = 0;
  let colorType = 6;
  const idat = [];
  while (off + 8 <= buf.length) {
    const len = buf.readUInt32BE(off);
    const type = buf.toString('ascii', off + 4, off + 8);
    const data = buf.subarray(off + 8, off + 8 + len);
    if (type === 'IHDR') {
      width = data.readUInt32BE(0);
      height = data.readUInt32BE(4);
      colorType = data[9];
    } else if (type === 'IDAT') {
      idat.push(Buffer.from(data));
    } else if (type === 'IEND') {
      break;
    }
    off += 12 + len;
  }
  const channels = colorType === 6 ? 4 : 3;
  const stride = width * channels;
  const raw = zlib.inflateSync(Buffer.concat(idat));
  const out = Buffer.alloc(width * height * 4);
  const line = Buffer.alloc(stride);
  const prev = Buffer.alloc(stride);
  for (let y = 0; y < height; y++) {
    const filter = raw[y * (stride + 1)];
    const src = raw.subarray(y * (stride + 1) + 1, y * (stride + 1) + 1 + stride);
    for (let i = 0; i < stride; i++) {
      const a = i >= channels ? line[i - channels] : 0;
      const b = prev[i];
      const c = i >= channels ? prev[i - channels] : 0;
      let v = src[i];
      if (filter === 1) v += a;
      else if (filter === 2) v += b;
      else if (filter === 3) v += (a + b) >> 1;
      else if (filter === 4) v += paeth(a, b, c);
      line[i] = v & 0xff;
    }
    for (let x = 0; x < width; x++) {
      const s = x * channels;
      const d = (y * width + x) * 4;
      out[d] = line[s];
      out[d + 1] = line[s + 1];
      out[d + 2] = line[s + 2];
      out[d + 3] = channels === 4 ? line[s + 3] : 255;
    }
    line.copy(prev);
  }
  return { width, height, rgba: out };
}
