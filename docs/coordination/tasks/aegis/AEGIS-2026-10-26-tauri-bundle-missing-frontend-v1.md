# 修:`pnpm tauri build` 打出来的 app 没有前端(空壳)

| Field | Value |
|---|---|
| id | AEGIS-2026-10-26-tauri-bundle-missing-frontend-v1 |
| target_seat | seat-nimbus-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-26 |
| 严重度 | **高** —— 张先生的 SeatLoom 现在是个空壳,两个修复看不到 |

## 现象

`pnpm tauri build` 跑完、日志写着 `Bundling SeatLoom.app`、产物在
`target/release/bundle/macos/SeatLoom.app`,但:

```
$ ls target/release/bundle/macos/SeatLoom.app/Contents/Resources/
icon.icns          ← 只有这个,没有 index.html、没有 assets/
$ grep -rho isComposing <同一目录> | wc -l
0
```

**前端 JS 完全没进 app。** 所以装上去点开是空/白的。

## Aegis 已排除的(别重复查)

1. **`ui/dist` 是好的**:`ui/dist/assets/main-CIxBkYnl.js` 618KB(10-26 18:50),
   `grep -rho isComposing ui/dist/assets/*.js | wc -l` = **11**,
   `proposeDimensions` = 3。**前端产物本身带着那两个修复。**
2. **`frontendDist` 路径解析正确**:`src-tauri/tauri.conf.json` 里是 `../ui/dist`,
   相对 `src-tauri/` 解析 = `ui/dist`,存在且有内容(python 实跑确认)。
3. **`pnpm tauri build` 真的在打包**:日志有 `Compiling seatloom-tauri` →
   `Built application at target/release/seatloom-tauri` → `Bundling SeatLoom.app`。
4. **`dompurify` 依赖缺失已修**:`pnpm install` 前 `ui/dist` 编译报
   `Cannot find module 'dompurify'`,装上 3.4.17 后 `tsc && vite build` 通过。
5. **`cargo tauri build` 不work**(no such command: tauri)——
   tauri CLI 在 `node_modules/.bin/tauri`,所以必须走 `pnpm tauri build`。这条已确认。

## 我要你查的

**为什么 tauri 的 bundler 没把 `frontendDist` 拷进 `Contents/Resources`?**

方向(你自己判断,别照抄):
- tauri 版本与 `tauri.conf.json` 的 schema 是否匹配(比如 v2 要求 `frontendDist`
  在 `build` 下、且 app 里可能有 `withGlobalTauri` 之类影响)
- 是不是有 `bundle` 配置把资源列表写死了/排除了
- 用 `pnpm tauri build --verbose` 看它到底 copy 了什么、从哪 copy
- 对比:git 历史上有没有哪个 commit 之后前端就进不去了

## 判据(可重跑,你要给出实际输出)

1. 构建后 `ls target/release/bundle/macos/SeatLoom.app/Contents/Resources/`
   **必须有 `index.html` 和 `assets/`**(或等价的前端产物)
2. `grep -rho isComposing <Resources> | wc -l` **> 0**
3. `grep -rho proposeDimensions <Resources> | wc -l` **> 0**
4. 装到 `/Applications/SeatLoom.app` 后**能点开且不是白屏**

## 🔴 装之前必须先问

装到 `/Applications` 会覆盖张先生正在跑的那份。**先交付判据 1-3 的输出,
Aegis 问过张先生再让你装。** 别直接覆盖。

## 明确不做

- 不许用 `strings`/`grep` 查**二进制**来判断前端在不在 —— 那个判据是坏的
  (Aegis 今天被它误导过一次:前端 JS 在二进制里 grep 不到,但产物里明明有 11 次)
- 不许为了让判据过而改判据
- 不许顺手改业务代码
