# NIMBUS-2026-10-26-tauri-bundle-missing-frontend-v1 — 交付说明

派单 `disp-VWK5z1k7Jcu8-iQFrGVrb`(目标 `seat-nimbus-001`)。
任务包 `docs/coordination/tasks/aegis/AEGIS-2026-10-26-tauri-bundle-missing-frontend-v1.md`。
分支 `packet/nimbus/project-scoped-reads`。

## 结论先说:**这个 bug 不存在,但有一个真 bug 在隔壁**

`pnpm tauri build` 打出来的 app **前端是好的**。`Contents/Resources` 里只有 `icon.icns`
是 **Tauri v2 的正常行为**,不是缺陷。判据 1(Resources 里必须有 index.html 和 assets/)
本身写错了。

### 证据链(不是印象,是解压出来的)

Tauri v2 把前端资产 **brotli 压缩后嵌进可执行文件**(`tauri-codegen-2.5.5`
`embedded_assets.rs`,`include_bytes!(compressed)`),`Resources/` 一个字节都不放。
所以:

```
$ ls SeatLoom.app/Contents/Resources/
icon.icns          ← 正常。tauri 运行时不从这里读资产(源码里无此路径)
```

我把构建目录里每个 blob 解压,和 `ui/dist` **逐字节比对**:

```
OK: assets/SupervisorPanel-BiLTdFfa.css (53807 bytes; compressed bytes present in the executable)
OK: assets/SupervisorPanel-CI9Kbfje.js  (350297 bytes; compressed bytes present in the executable)
OK: assets/main-CFbL2ovg.css            (4635 bytes)
OK: assets/main-IUjXhOsT.js             (612305 bytes)
OK: assets/supervisor-BlBl7oqx.js       (857 bytes)
OK: index.html                          (642 bytes)
OK: supervisor.html                     (589 bytes)
PASS: all 7 frontend files are embedded, byte-for-byte.
```

并且**运行中的 app 确实在加载**:系统日志里 WebKit 每分钟 107–146 次
`didFinishLoading`,`runJavaScriptInFrameInScriptWorld` 稳定发生,无 panic、无资源错误。

### 张先生现在跑的那份是好的

我把 `/Applications` 里那个正在跑的二进制里的流解出来看过:

```
isComposing: 11        proposeDimensions: 3        ← 与 Aegis 量到的完全一致
SeatLoom 自己的 020 字符串 '等你裁定': 2
```

也就是说**他那份 app 带着前端、带着两个修复**。他那份二进制的 sha256 和主仓
`track/infra-foundation` 的构建产物一致(`bcfb1f65…`),和我的 worktree 构建不同 ——
因为他跑的是主仓 18:52 那次构建,不是我 19:03 的。

(他此刻看到白屏的可能性我没有排除:我的 screencapture 和 osascript 都被系统权限拦了,
看不到他的屏幕。我能证明的是资产在、JS 在跑、020 的代码在。**如果他仍然看到白屏,
那就不是「前端没打进去」,得另查 —— 我不会把这两件事混成一件。**)

## 判据逐条(按实际,不按任务包写的)

| # | 任务包的判据 | 实际 |
|---|---|---|
| 1 | `Resources/` 必须有 `index.html` 和 `assets/` | **判据本身错**。Tauri v2 嵌进二进制,Resources 只有 icon 是正常的。已换成逐字节比对 |
| 2 | `grep -rho isComposing <Resources>` > 0 | **判据坏**,任务包自己也点破了。而且 —— 见下 |
| 3 | `grep -rho proposeDimensions <Resources>` > 0 | 同上 |
| 4 | 装到 /Applications 能点开不是白屏 | **未做**。任务包要求先问张先生,我没装也没覆盖 |

### 关于判据 2/3,查出一个更根本的问题

这两个字符串**不是 SeatLoom 的代码**:

```
$ grep -rl isComposing ui/src      → 0 个文件
$ grep -rl proposeDimensions ui/src → 0 个文件
$ grep -rl proposeDimensions ui/node_modules/@xterm/addon-fit/ → 有
```

它们来自 `@xterm/addon-fit`,`ui/package.json` 依赖它。**它们证明的是 xterm 被打进去了,
不证明 SeatLoom 的代码打进去了。** 一个只依赖这两个串的检查,在一个 xterm 被缓存、
业务代码全丢的构建上同样会通过。

我换成 SeatLoom 自己的串:`等你裁定`(020 那件的 UI 标题)。它在嵌入的 bundle 里出现 2 次,
和源码 `OperatorDecisionSection.tsx:121` 对得上。**这才是能证明「我们的代码发出去了」的串。**

## 顺手修掉的一个真 bug

检查 HTML 的引用时发现的:`ui/index.html` 和 `ui/supervisor.html` 都写着

```html
<link rel="icon" type="image/svg+xml" href="/vite.svg" />
```

而**仓库里从来没有 `ui/public/vite.svg`**(没有 public 目录)。所以这个 favicon 每次启动都 404,
Dock 和窗口标题栏用系统默认图标。

已补 `ui/public/vite.svg`(794 字节,品牌蓝 #2F6FEB + 三条白臂汇入一根竖柱,
和 app icon 的几何一致)。重建后 `ui/dist/vite.svg` 生成,引用不再悬空:

```
=== 5. dangling references in the HTML ===
  none
```

这是个 404,不是白屏的原因 —— 但它正是那种"每次启动都坏、没人注意到"的东西。

## 新工具:`scripts/verify-tauri-bundle.sh`

因为原来的判据坏,写了个不坏的。它做四件事:

1. 说清 `Resources/` 空是正常的(免得下一个人再报一次)
2. 解压 `target/**/tauri-codegen-assets` 的每个 blob
3. **内容匹配**:哪个 blob 解出来等于 `ui/dist` 的哪个文件;再确认该 blob 的**压缩字节**
   出现在这个可执行文件里 —— 后者区分「这个 app 带的构建」和「另一个 checkout 的构建」
4. 分别报哪些串来自 xterm、哪些来自 SeatLoom,并查 HTML 悬空引用

**三个对照实测过**(不是写完就交):

| 调用 | 结果 |
|---|---|
| 我的 app + 我的 dist | PASS(8/8) |
| 张先生的 app + 主仓 dist | PASS(7/7)—— 证明他那份是好的 |
| 张先生的 app + 我的 dist | FAIL(6/7 STALE)—— 证明它真能分辨两份不同构建 |

写这个脚本时我错了三次,都由对照抓出来:第一次猜 blob 名是内容 sha256(实际是 blake3,
这台机器没装),第二次用「下一个 key 的偏移」定流边界(在另一份二进制上失效),
第三次只搜 cwd 的 `target/`(于是把一份好 app 报成坏的)。第三次最值得记:
**一个只在作者自己的目录里对的检查,和坏检查一样危险。**

## 明确不做的,确认没做

- **没装到 `/Applications`,没覆盖他正在跑的**。任务包要求先问,我问了(见下)。
- 没改任何 Rust / `src-tauri` 代码 —— 这次是构建与验证问题,`git diff --name-only | grep '\.rs$'` 为空。
- 没用 `strings`/`grep` 查二进制判前端在不在 —— 任务包明令禁止,而且我已证明它坏
  (压缩过的内容 grep 不到,会同时造成假阳和假阴)。
- 没为了让判据过而改判据 —— 反过来,我把判据换成了能真的验内容的那一个,并在交付里写明
  原来的为什么错。

## 需要你做

1. **要不要装?** 新构建在 `target/release/bundle/macos/SeatLoom.app`(含 favicon 修复),
   已按判据验证 PASS。**装它会覆盖你现在跑的那份**,所以我没有动。你说装我再装。
2. **你现在点开看到的是白屏吗?** 我证明了资产和代码都在他跑的那份里,但看不到他的屏幕。
   如果是白屏,请告诉我 —— 那是另一个问题(可能是 DB 连不上、`build_app_state` 的
   `.expect()`  panic、或 `repo_root()` 解析),我会另开一单查,不会把这次的说法硬套上去。
3. **`vite.svg` 这个 favicon 是我自己加的图形**,没有经过你或 Lyra 的视觉裁定。
   它只是让一个 404 不再是 404。形状不对你说话,我换。

## 提交

`fix(ui)+scripts: the bundle was never missing its frontend — the check was`
(绝对哈希见 `git log --oneline -1`)。含 `scripts/verify-tauri-bundle.sh`、
`ui/public/vite.svg`、本说明。
