# `disp-H66BOKrYda4xltEgIwvuG` — 实时席位链接点不开

## 一句话

**根因就是 Aegis 查到的那个，已修，两侧都修了。但我在真机验证时发现一个比本派单更严重的问题：构建出来的 `.app` 里根本没有前端资产 —— 这不是我引入的，主检出同样如此，需要你决定怎么处理。**

---

## 改了什么

### 1. `ui/src/app-v2/panel/SessionTerminal.tsx` — WebLinksAddon 传 handler

```ts
term.loadAddon(
  new WebLinksAddon((_event: MouseEvent, uri: string) => {
    api.openExternalUrl(uri).catch((err) => {
      console.error('[SessionTerminal] openExternalUrl failed:', uri, err);
    });
    return true;
  }),
);
```

`return true` 是必需的：它告诉 addon"我已经认领了这个点击"，否则 addon 还会跑它自己的默认 handler（就是那个被 Tauri 挡住的 `window.open`）。少了这个返回值，修了等于没修。

### 2. `ui/src/layouts/StatusBar.tsx` — 同一个病，而且是两个病

原代码：

```tsx
<div ... tabIndex={0} onKeyDown={(event) => event.key === 'Enter' && window.open('https://github.com/seatloom')}>
```

这里有两个独立问题：

- **`window.open` 被 Tauri 挡**（和终端链接同一个病）
- **完全没有 `onClick`** —— 鼠标点它从来就没绑过任何东西。派单说"同一个病"，实际比那更糟一点：鼠标路径是缺失的，不是坏的。

两处都修了，顺带把 Space 键也接上（`role="link"` 按惯例要响应 Space，原来只有 Enter），并加了 `title` 显示真实 URL（可见文字是本地路径 `~/Documents/GitHub/seatloom`，和点击去向不一致，原来这个不一致是看不见的）。

### 3. `src-tauri/src/commands/history_cmds.rs` — 新 command

```rust
pub async fn cmd_open_external_url(url: String) -> Result<(), String>
```

**为什么走这条路而不是 `@tauri-apps/plugin-shell` 的 JS `open()`**：派单让我"用项目里已经在用的那个，不要另起一套"。我查了 —— `ui/package.json:22` 确实声明了 `@tauri-apps/plugin-shell`，但 **Rust 侧完全没注册它**（`main.rs` 里没有任何 `.plugin(...)` 调用）。用它意味着新增 Cargo 依赖、新增 capability 权限、并且出现第二种打开方式。

项目**已经有的**做法是 `cmd_open_local_path`：Rust 侧 `std::process::Command` + `open`(macOS)/`xdg-open`(Linux)。我跟了这个，一个机制，已经在二进制里。

### 4. 注册与封装

- `src-tauri/src/main.rs` — 2 行（import + handler 注册）
- `ui/src/lib/api.ts` — `openExternalUrl: (url) => invoke('cmd_open_external_url', { url })`

### Tauri 侧配置：没有改

不需要改 `tauri.conf.json` 或 `capabilities/default.json`。理由：这个 command 不走 Tauri 的 shell 插件，所以不触及其 permission 模型；它和 `cmd_open_local_path` 走的是同一条路径，而后者已经在跑，没有额外 capability。

---

## 非 http(s) 的决定：拒绝，并把 scheme 名报出来

派单要求给决定并说理由。

**决定：只开 `http` 和 `https`。`file://` / `mailto:` / `javascript:` 一律拒绝，错误信息里带上 scheme 名。**

**理由**：URL 是 wrapped CLI 打印出来的，也就是"任何能往 operator 终端里写字的人都能选它"。

- `file://` 会让终端输出打开任意本地文件 —— 这正是 `cmd_open_local_path` 用 context-root 检查在防的事，而这条路径没有那个检查的等价物
- `mailto:` / `javascript:` 等于把终端文本交给另一个应用的解析器

**为什么是显式报错而不是静默丢弃**：静默丢弃会复刻原 bug 最坏的性质 —— 点了没反应，且没有任何地方说明为什么。报错后前端 `.catch` 会打 console.error 并带上 URL，operator 或下一个查这个问题的人能看到"refused to open file:///etc/passwd: only http and https are opened"。

**没有做白名单域名**：那会让"点一个没预登记过的链接"变成静默失败，而 operator 要的是打开。scheme 是这一层能负责任地判定的全部。

Rust 侧 9 个测试（新增 6 个）覆盖：http 放行、file/mailto/javascript 拒绝且错误含 scheme 名、`https://` 无 host 拒绝、userinfo 取最后一个 `@` 之后的部分。

---

## 测试（前端 9 条 + Rust 6 条，全部变异验证）

### 前端 `SessionTerminal.links.test.tsx`（4 条）

驱动真实 xterm + 真实 WebLinksAddon，handler 从 addon 实例上读出来再调用 —— 不是测试里重建一个 handler 自己调自己。

| 变异 | 结果 |
|---|---|
| M1 `new WebLinksAddon()` 不传 handler（回到 bug） | **红**，杀 2 |
| M2 handler `return false` | **红**，杀 1 |
| M3 handler 吞掉错误 `.catch(() => {})` | **红**，杀 1 |

### 前端 `StatusBar.test.tsx`（5 条，新文件）

| 变异 | 结果 |
|---|---|
| M4 去掉 `onClick` | **红**，杀 2 |
| M5 只留 Enter、去掉 Space | **红**，杀 1 |
| M6 去掉 `title` | **红**，杀 1 |
| M7 吞错误 | **红**，杀 1 |

### Rust `history_cmds.rs`（新增 6 条）

| 变异 | 结果 |
|---|---|
| M1 删 scheme 门 | **红**，杀 3 |
| M2 删 host 门 | **红**，杀 1 |
| M3 userinfo 取第一个 `@` 而非最后一个 | **红**，杀 1 |

### 两个我自己写错又抓住的地方，写在这里免得重犯

**① 同义反复测试。** userinfo 那条测试最初在测试里重新实现了一遍和产品代码一样的字符串解析，然后断言解析结果 —— 当然永远绿。改成通过 command 自己的错误信息来观察（`https://@github.com/...` 必须不被判成"no host"），这才有了鉴别力。**教训和之前记过的那条同源：测试里重写一遍被测逻辑，等于什么都没测。**

**② 派单里的一处不准确，我按实测改了。** Aegis 写默认 handler"返回 null 只留 console.warn"。实测 `@xterm/addon-web-links@0.11.0` 的默认 handler 是 `function i(e,t){const n=window.open(); if(n){...} else console.warn(...)}` —— **没有 return 语句，返回 `undefined`**。后果一样（falsy），但断言写了代码实际做的事，并在注释里记下这个偏差。

---

## jsdom 的边界（派单要求说明替代物及其差异）

**没能测到的：链接的像素命中区域。** jsdom 不做布局，xterm 的 DOM renderer 算不出链接的起止坐标，合成点击落不到链接上。

**测到的：handler 是否存在、是否返回 true、是否把 URL 路由到 `openExternalUrl`。** 而这三样正是坏掉的部分 —— 原 bug 不是"点偏了"，是"根本没 handler"。

canvas 打了桩（xterm 的 `open` 需要 `matchMedia`，DOM renderer 需要 2d context）。**这意味着这些测试不断言任何渲染相关内容**，测试文件头部写明了这一点，避免以后有人把桩当成"我们测过视觉"。

---

## 🔴 真机验证：只做到一半，且过程中发现一个更严重的问题

### 做到了的

- release 二进制构建成功，`strings` 确认含 `cmd_open_external_url`（出现 2 次，与 `cmd_open_local_path` 同等）
- `ui/dist` 产物确认含新调用（`SupervisorPanel` chunk 里 `cmd_open_external_url` 命中）
- 21 个 Rust 测试、330 个前端测试、typecheck、eslint（143 problems，与 baseline 一致）、build 全过

### 没做到的

**"在装好的 app 里点一个 URL 真的打开浏览器"—— 我没做到。** 我不能点击 GUI。派单把这条列为判据，所以这条判据**未闭合**，需要张先生或有人能在真机上点一下。

### 过程中发现：构建出的 `.app` 里没有前端资产

`cargo tauri build` 结束后，`bundle/macos/SeatLoom.app/Contents/Resources/` 里**只有 `icon.icns`** —— 没有 `assets/`，没有 `index.html`。

`tauri.conf.json` 里 `frontendDist: "../ui/dist"`，`ui/dist` 确实存在且是 16:03 的新构建（含我的改动），但 tauri 没有把它拷进 bundle。我重跑了一次并删掉旧 bundle 排除缓存，结果一样。

**这不是我的改动引入的**：主检出 `~/Documents/GitHub/seatloom` 同样配置、同样构建产物，bundle 里也只有 `icon.icns`。所以 `/Applications/SeatLoom.app` 很可能一直都是这样 —— 这大概能解释 operator 看到的应用行为为什么和代码不一致（Flux 提过五次的"运行中的二进制不含修复"，也许不只是二进制旧的问题）。

**我没有修它。** 改 `tauri.conf.json` 的 `frontendDist` 或构建流程是构建系统变更，影响所有席位的产物，超出本派单范围，也不该由我在一个链接派单里顺手改。

---

## 需要你决定的

1. **`frontendDist` 不生效这件事怎么处理** —— 这比本派单的链接问题更靠前，因为它会让任何前端修复都到不了 operator 眼前。我建议单独开一个派单给负责构建的人，而不是我在这里猜。
2. **链接的真机点击验证** —— 需要有人装上含新代码的 app 点一下。当前 `/Applications/SeatLoom.app` 的二进制是 14:03 的，`strings` 查 `cmd_open_external_url` 为 0，**不含**本修复。
3. **上一条派单遗留**：`dto-to-v1.ts:159` 的 `isPinned: true` 硬编码仍未修。

## 门禁

- `pnpm test`:330 passed / 28 files(baseline 314 + 我 9 + 滚动 7)
- `cargo test`:21 passed(其中 history_cmds 9 个)
- `pnpm typecheck`:干净
- `pnpm exec eslint src`:143 problems(133 errors + 10 warnings)—— **与 baseline 完全一致**,我碰过的文件 0 问题
- `cargo clippy --all-targets`:0 warning(第一版有 1 个,已修:`.split('@').last()` 改成 `.rsplit('@').next()`)
- `prettier`:我新增的 2 个测试文件干净。`StatusBar.tsx` 和 `api.ts` 在我改之前就已经不符合 prettier(stash 对比确认),我未新增脏点
- `pnpm build`:通过
- `cargo build --release` + `cargo tauri build --no-sign`:通过

## 改动文件

- `src-tauri/src/commands/history_cmds.rs` — 新 command + 6 个测试
- `src-tauri/src/main.rs` — 2 行注册
- `ui/src/lib/api.ts` — `openExternalUrl` 封装
- `ui/src/app-v2/panel/SessionTerminal.tsx` — WebLinksAddon handler
- `ui/src/layouts/StatusBar.tsx` — onClick + openExternalUrl + Space + title
- `ui/src/app-v2/panel/SessionTerminal.links.test.tsx` — 新增,4 条
- `ui/src/layouts/StatusBar.test.tsx` — 新增,5 条
