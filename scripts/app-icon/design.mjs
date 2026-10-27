// SeatLoom 应用图标 —— 设计稿(含候选方案对比)。
//
// 一张图标,说清它是什么、为什么是它。定稿那一个写在这里,其余候选保留在
// CANDIDATES 里 —— 16 像素下的取舍不是嘴上说的,是下面 asciiPreview 打出来看的。
//
// 硬约束(张先生 2026-10-16 点名这条派单时给的):
//   · 必须在小尺寸下还认得出来。程序坞可以缩到很小,Cmd+Tab 更小。
//   · 别用照片、别用渐变堆细节、别放文字 —— 缩到 16 像素全都糊成一团。
//   · 画什么由我定,但要在交付里说清画的是什么、为什么;好不好看是他的判断。

import { asciiArt, encodePng, render } from './render.js';

/** 产品的品牌蓝,和 ui/src/app-v2/styles/tokens.css 的 --sl-brand 一致。 */
const BRAND = [0x2f, 0x6f, 0xeb];
const WHITE = [0xff, 0xff, 0xff];

/** macOS Big Sur 之后的圆角半径,约边长的 22.37%。 */
const TILE_RADIUS = 0.2237;

function tile() {
  return { type: 'rect', cx: 0.5, cy: 0.5, hw: 0.5, hh: 0.5, r: TILE_RADIUS, color: BRAND };
}

function bar(x0, y0, x1, y1, hw, color = WHITE) {
  return { type: 'bar', x0, y0, x1, y1, hw, color };
}

function disc(cx, cy, r, color = WHITE) {
  return { type: 'disc', cx, cy, r, color };
}

// ─────────────────────────────────────────────────────────────────────────────
// 候选方案
// ─────────────────────────────────────────────────────────────────────────────

const CANDIDATES = {
  // A:三道平行粗杠 + 右边一道更粗的汇杠。粗杠之间留住蓝缝。
  A: () => [
    tile(),
    bar(0.185, 0.29, 0.5, 0.29, 0.058),
    bar(0.185, 0.5, 0.5, 0.5, 0.058),
    bar(0.185, 0.71, 0.5, 0.71, 0.058),
    bar(0.475, 0.5, 0.815, 0.5, 0.1),
  ],

  // B:三道杠真的斜着汇进右边一道竖杠 —— 整块是一个连通的形状(一个轮廓),
  //    而且关于横轴对称。定稿用这一个,理由写在交付里:
  //      · 一个连通轮廓比四个独立元素更难在小尺寸下糊掉;
  //      · 三道等粗 + 一道更粗,把「多个席位」和「那一个地方」区分开 ——
  //        三臂等长的对称星形会说成「三个一样的东西碰头」,那不是这个产品干的事;
  //      · 横轴对称,程序坞和 Cmd+Tab 里缩小看着不歪。
  B: () => {
    const trunkX = 0.6;
    const armHw = 0.068;
    return [
      tile(),
      // 那一个地方:右边一道更粗的竖杠。
      bar(trunkX, 0.235, trunkX, 0.765, 0.092),
      // 三个席位:等粗,从左边斜着汇进来。
      bar(0.205, 0.235, trunkX, 0.4, armHw),
      bar(0.205, 0.5, trunkX, 0.5, armHw),
      bar(0.205, 0.765, trunkX, 0.6, armHw),
    ];
  },

  // C:同 A,但整体收小、留白更多,看小尺寸下结构是否更清楚。
  C: () => [
    tile(),
    bar(0.22, 0.3, 0.48, 0.3, 0.05),
    bar(0.22, 0.5, 0.48, 0.5, 0.05),
    bar(0.22, 0.7, 0.48, 0.7, 0.05),
    bar(0.46, 0.5, 0.78, 0.5, 0.088),
  ],

  // D:一个圆心(那一个地方)+ 三条辐条(席位)。两元素,重心在中间。
  D: () => [
    tile(),
    disc(0.5, 0.5, 0.15),
    bar(0.5, 0.5, 0.83, 0.5, 0.062),
    bar(0.5, 0.5, 0.22, 0.24, 0.062),
    bar(0.5, 0.5, 0.22, 0.76, 0.062),
  ],

  // E:一个实心的漏斗形 —— 左边一整条竖边,右边收成一个尖。就一个形状。
  E: () => [
    tile(),
    { type: 'tri', ax: 0.2, ay: 0.22, bx: 0.2, by: 0.78, cx: 0.82, cy: 0.5, color: WHITE },
  ],

  // F:三道 120° 的粗杠交于一点 —— 对称,一个轮廓,16 像素下最认得出来的一种形状。
  F: () => {
    const cx = 0.5;
    const cy = 0.565;
    const hw = 0.078;
    return [
      tile(),
      bar(cx, cy, cx, 0.17, hw),
      bar(cx, cy, 0.225, 0.775, hw),
      bar(cx, cy, 0.775, 0.775, hw),
    ];
  },

  // G:居中的一道粗竖杠(那一个地方)+ 左边三道短杠(席位)。
  G: () => {
    const hw = 0.055;
    return [
      tile(),
      bar(0.5, 0.22, 0.5, 0.78, 0.088),
      bar(0.16, 0.3, 0.5, 0.3, hw),
      bar(0.16, 0.5, 0.5, 0.5, hw),
      bar(0.16, 0.7, 0.5, 0.7, hw),
    ];
  },
};

/** 定稿。 */
export const CHOSEN = 'B';

export function scene(variant = CHOSEN) {
  const build = CANDIDATES[variant];
  if (!build) throw new Error(`unknown icon variant: ${variant}`);
  return build();
}

/** 渲染一个尺寸,返回 PNG Buffer。ss 按尺寸反算,总采样量恒定。 */
export function renderPng(size, variant = CHOSEN, totalSamples = 2048) {
  const ss = Math.max(1, Math.round(totalSamples / size));
  const rgba = render(scene(variant), size, ss);
  return { png: encodePng(size, size, rgba), rgba, ss };
}

if (process.argv[1] && import.meta.url.endsWith(process.argv[1].replace(/^.*\/scripts\//, 'scripts/'))) {
  const variant = process.argv[2] ?? CHOSEN;
  const sizes = (process.argv[3] ?? '16,32,64').split(',').map(Number);
  for (const size of sizes) {
    const { rgba, ss } = renderPng(size, variant);
    console.log(`\n# ${variant} @ ${size}x${size} (ss=${ss})`);
    console.log(asciiArt(rgba, size));
  }
}
