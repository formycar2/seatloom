# 独立核 Flux 的验单 `disp-i_ZLKc4E2S0Bm78K-Zmdy`

Mira 的审计(`ev-mira-audit-flux-i_ZLKc01`)。被核对象:Flux 对两个已合并 Nimbus 修复的审计 verdict `ev-hh9xak8wlN5fJst6107cy`。

规则来源:`docs/coordination/acceptance/2026-10-27-verification-gate-class-rule-decision.md` —— Flux 是发 verdict 的席位,不能给自己的验单发 verdict,所以由"没做过这个活的工人席位"核。

---

## 结论:**pass**

Flux 这个验单做得对。**它的每一条承重声明我都在 track 代码、活库、或当前构建产物上重跑过**,没有一条是读它的交付说明转抄的。

而且 —— 这条比 pass 更重要 —— **它纠正了我自己的一个错误上报**,我为此单独列一节。

---

## 一、逐条核 Flux 的声明

### 声明 1:"Resources 只有 icon.icns 是 Tauri v2 的正常行为,前端被 brotli 压进可执行文件"

**这是整份 verdict 里最要紧的一条,因为它直接推翻了我上一单的上报。**

我的核实路径(不是读 Flux 的说明,是自己从源码推到实测):

1. **读 tauri-codegen 2.5.5 源码** `embedded_assets.rs`:
   `compress_file()` 把每个资产 brotli 压缩后写到 `OUT_DIR/tauri-codegen-assets/<blake3>.<ext>`,
   `ToTokens` 里 emit 的是 `#key => { include_bytes!(#input); include_bytes!(#output) }`
   —— 即 phf map 的 key 是路径串,value 是**压缩后的字节**。所以路径串出现在 exe 里、内容不在同一位置,是格式决定的。
2. **在 exe 里定位到路径串**(`assets/main-ByA8nAZb.js` 等 5 个,偏移 6.20MB–6.36MB),
   确认路径后面跟的是 `\x1bX\x03\x00` + 二进制,不是明文内容 —— 与上面的格式一致。
3. **决定性实验**:从 `target/release/build/*/out/tauri-codegen-assets/` 取出 16:02 那一批
   (本次构建实际嵌入的 8 个),逐个 brotli 解压:

   | 扩展名 | 压缩后 | 解压后 | SeatLoom 特征串 |
   |---|---|---|---|
   | js | 136202 | **624173** | **7 个命中** |
   | js | 93869 | 351682 | 0 |
   | css | 9092 | 54746 | 0 |
   | html | 259 | 589 | 0 |
   | html | 260 | 642 | 0 |
   | js | 497 | 857 | 0 |
   | svg | 382 | 794 | 0 |
   | (空) | 1408 | 0 | 0 |

   解压后 624173 字节的那个 js 命中 `实时席位`/`scrollToBottom`/`WebLinksAddon`/`isComposing` 等 —— **是真实前端**。
4. **跑 Nimbus 写的 `scripts/verify-tauri-bundle.sh`**(Flux 的发现促使他重写了标签),
   它对 8 个 dist 文件逐一做"压缩字节是否逐字出现在本 exe 里"的检查。

**结论:前端确实嵌在可执行文件里。Flux 对,Aegis 最初的报告是误报。**

### 声明 2:"Nimbus 的脚本把 isComposing 标成 '@xterm/addon-fit — NOT SeatLoom',这是错的"

实测 `ui/src/hooks/useGlobalShortcuts.ts:31` 就是 `if (event.isComposing) return true;`。
Flux 说得对,而且 Nimbus 已经按这条改了脚本(现在脚本第 4 节明确写 `isComposing → SeatLoom
useGlobalShortcuts.ts:31 (was mislabelled xterm-only)`)。**Flux 的发现被采纳并落地了。**

顺带我核了另一半:`proposeDimensions` 在 `ui/src` 里只出现在 SessionTerminal 的注释和
LiveSeatsView 的注释里,真实实现只在 `@xterm/addon-fit/lib/addon-fit.js`。Flux 这条也准。

### 声明 3:tilde 修复真实存在且全链打通

- `ui/src/lib/dto-to-v1.ts:158` 现在是 `path: d.homePath ?? null`,注释明写旧代码是
  `'~/' + d.name`、以及它为什么是假的。
- Rust 侧 `models.rs` 有 `home_path: Option<String>`、`repositories.rs` 三处 SELECT 带
  `home_path`、`dto.rs` 透传;前端 `types-dto.ts:22` 与 `types/index.ts` 有 `homePath: string|null`。
  **Flux 说的"SQL → ProjectRow → ProjectDto → TS DTO → mapper → 三个调用点"逐层都在。**

### 声明 4:"39 projects, 0 with home_path IS NULL"

活库实测:

```
40|40|0     -- total | with_home | null_home
```

**40 个项目,40 个有 home_path,0 个 NULL。** 总数比 Flux 记的多 1(之后可能又加了项目),
但承重结论"0 NULL"一致 —— 即 `d.homePath ?? null` 的 null 分支今天只能靠测试触达,Flux 这条判断成立。

### 声明 5:`networkOrchestrator` 的 home_path 是真实绝对路径

```
project_delivery/networkOrchestrator|networkOrchestrator|/Users/…/project_delivery/networkOrchestrator
```

且全库 `count(*) where home_path like '~/%' or home_path !~ '^/'` = **0**,无合成路径残留。
Flux 说"cause_reproduced 对照过活库" —— 对。

### 声明 6:变异测试的杀伤力

我重跑了 Flux 列的两个变异(在 `AllProjectsView.hydration.test.tsx` 上):

| 变异 | Flux 声明 | 我的实测 |
|---|---|---|
| 恢复 `path: '~/' + d.name` | 2 tests red | **2 red**,同一节 |
| lazy fix `path: d.name` | 2 tests red,同样那两条,因为测试断言真实路径逐字出现 | **2 red,同样那两条** |

**两个都与 Flux 的声明逐字吻合。** 第二条尤其重要:它证明测试钉的是"真实路径出现",
不是"tilde 消失",所以那个看起来更干净的 lazy fix 也过不去。

---

## 二、Flux 站得住的地方,除了结论对

它做了一件容易漏的事:**在自己 verdict 里记录自己的错误**。`my_own_error_recorded` 写它第一次
跑全量 suite 时把 node_modules symlink 进另一个席位的 worktree,两套 suite 因为 pnpm store
布局不同而解析不到 dompurify;它改用主检出的 install 重跑(278/278),而不是把那两个失败
当成代码缺陷报上来。

**这条我核不了**(无法复现它当时的 worktree 状态),但它写进 verdict 本身就是正确的做法 ——
一个把"我这次测量方法出错了"记下来的验单,比一个只报绿结论的验单可信。

---

## 三、我自己的错误,以及它为什么值得单独一节

我上一单(`disp-H66BOKrYda4xltEgIwvuG`)把"bundle 的 Contents/Resources 只有 icon.icns"
上报成"比本派单更严重的发现:构建出的 .app 里没有前端资产",并说"主检出同样如此"、
"这能解释 operator 看到的应用行为为什么和代码不一致"。

**这个上报是错的,而且错在我用来判定的方法上:**

1. 我看了 `find .../Contents -type f`,看到只有 `icon.icns` + `Info.plist` + 可执行文件,
   就断定前端没打包。
2. 我 `grep -ra 'cmd_open_external_url' Resources/` 得到 0,进一步"确认"。
3. **我没有去读 Tauri 的嵌入机制,也没有尝试解压。**

两个方法错误:

- **在错误的目录里找。** Tauri v2 把压缩后的前端嵌进**可执行文件**,不在 Resources。
  在一个它本来就不会出现的地方搜"它不在场",搜不到是必然的,那个 0 什么都不能证明。
- **文件名比内容先让我下了结论。** exe 里确实有 `assets/main-ByA8nAZb.js` 这类路径串,
   我当时没注意到,或者看到了也没往"这是 phf map 的 key"上想。

**Flux 的做法是我该做的**:读 `tauri-codegen` 源码确认格式,再把压缩 blob 解压开看内容。

我把这一节写进 payload,是因为一个说不清的 pass 比 fail 更糟 —— 反过来,**一个说不清的
自我纠错也比不记更糟**。下游需要知道我这个上报曾错在哪,否则同样的误判会再发生一次
(它在 10-26 已经以"Aegis 最初的 bug 报告"的形式发生过一次了)。

---

## 四、我核不了的、以及为什么不影响结论

| 核不了的 | 为什么 | 影响 |
|---|---|---|
| Flux 当时那两套 suite 的 dompurify 失败 | 无法复现它当时的 worktree 状态 | 无,它记录的是自己的方法错误,不是代码结论 |
| Flux 的"278 passed (23 files)"复现 | 那是 track 上某个提交的数字,不是我分支的数字 | 无,与两个修复的真实性无关 |
| operator 是否真的看到了修复后的界面 | 需要人打开 app | 无,Flux 自己在 `did_not_do` 里明确说了没打开过 |

---

## 五、给下游的

`disp-i_ZLKc4E2S0Bm78K-Zmdy` 的 verdict **pass** 依据充分:两个修复在 track 上真实存在、
全链打通、变异测试的杀伤力与它声明的逐字吻合、活库状态与它声明的一致。

**但它纠正的那个误报需要被记住**:判断"资产有没有打包"不能靠 `find Resources`,要靠解压
exe 里的压缩 blob。项目里现在有 `scripts/verify-tauri-bundle.sh` 做这件事, exit 1 表示
真的没嵌进去。
