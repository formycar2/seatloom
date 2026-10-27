# 实时席位里的链接点不开 —— WebLinksAddon 用了默认 handler,而 Tauri 拦 window.open

| Field | Value |
|---|---|
| id | AEGIS-2026-10-27-xterm-links-unclickable-v1 |
| target_seat | seat-mira-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-27 |

## 现象(张先生 2026-10-27)

页面上的超链接文字,在 tmux 里的 saih 中可以点开,
**在 SeatLoom UI 的实时席位里点不开**。

## Aegis 查到的(有文件行号)

`ui/src/app-v2/panel/SessionTerminal.tsx`:

```ts
:32   import { WebLinksAddon } from '@xterm/addon-web-links';
:116  term.loadAddon(new WebLinksAddon());     // ← 没传 handler
```

`@xterm/addon-web-links@0.11.0` 的**默认 handler**(我读了它的产物):

```js
function i(e,t){ const n=window.open(); if(n){ try{n.opener=null}catch{} n.location.href=t } else console.warn(...) }
```

也就是说:`activate: i` → **`window.open()`**。

而 **Tauri 的 webview 默认拦 `window.open`**(安全设计,新窗口要么声明白名单、
要么走 shell 插件)。所以在 SeatLoom 里:

- 链接**被识别了**(插件在跑,URL 正则在工作)
- 点击**也收到了**
- 但打开动作是**空的** —— `window.open()` 返回 null,只留一条 console.warn

**tmux 里能点开是因为 iTerm 自己处理 URL,不经过浏览器。**

**全仓现在没有任何 `openExternal` / `openUrl` / `shell.open` 调用**
(我 grep 过 `ui/src/` 和 `src-tauri/src/`,只有 `StatusBar.tsx:15` 一处 `window.open`,
那是 GitHub 链接,同一个病)。

## 要你做的

1. **给 WebLinksAddon 传一个 handler**,让它在 Tauri 里真的能打开外部浏览器
   - 先查这个项目现有的做法:Tauri v2 用 `@tauri-apps/plugin-shell` 的 `open()`,
     或者 tauri.conf.json 里配 allowlist。**用项目里已经在用的那个**,
     不要另起一套
2. **`StatusBar.tsx:15` 那处 `window.open` 一并处理** —— 同一个病
3. **修完要在真机上验**(build + 装 + 点一个链接),不是只让测试绿

## 判据

- 在装好的 app 里点实时席位的一个 URL,**真的打开了浏览器**
- 非 http(s) 的(比如 `file://`、`mailto:`)怎么办?给个决定,并说清理由
- 如果 Tauri 侧要改配置(allowlist / capability),说明改了哪里、为什么

## 明确不做

- 不许只加 `target="_blank"` 或改成 `<a>` 标签 —— 这是 xterm 的 canvas 渲染,
  不是 DOM,那条路走不通
- 不许为了让点击"有反应"就弹个提示说"已复制链接" —— 张先生要的是打开
- 不许改视觉风格
- 不许顺手改别的快捷键或交互

## 顺带:Aegis 在这件事上错了一次,记在这里免得你重走

我一开始看 `node_modules/@xterm/addon-web-links` 只有 `lib/`,说"包没装全"。
**错了** —— pnpm 的软链目标在 `.pnpm/` 下,typings 在那边,包是完整的,
`tsc` 也过。我看了软链目录没看解析后的真身。
