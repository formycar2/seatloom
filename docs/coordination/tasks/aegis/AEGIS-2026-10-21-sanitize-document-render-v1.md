# 净化文档渲染:排版照旧,剥掉执行能力

| Field | Value |
|---|---|
| id | AEGIS-2026-10-21-sanitize-document-render-v1 |
| target_seat | seat-mira-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-21 |
| baseline | track/infra-foundation |
| 张先生裁定 | 2026-10-21 批准,原话:「功能完全不变 —— 你看到的文档还是那个文档,只是里面的纸条不再被执行」 |

## 为什么派这一单

`ui/src/app-v2/views/DocumentsWorkspace.tsx:301`

```ts
const rendered = useMemo(() => {
  // marked returns HTML; cast to string (async mode disabled).
  return marked.parse(body, { async: false }) as string;
}, [body]);
```

输出在 `:359` 原样注入页面:

```tsx
dangerouslySetInnerHTML={{ __html: rendered }}
```

**关联已在主线实测确认**(不是推测):

```
marked 输入                          marked 输出
正常文字                          → <p>正常文字</p>
<script>alert("执行")</script>     → <script>alert("执行")</script>   ← 原样保留
<img src=x onerror=alert(1)>       → <img src=x onerror=alert(1)>      ← 原样保留
[点我](javascript:alert(2))        → <a href="javascript:alert(2)">    ← 原样保留
```

`body` 来自数据库的文档正文(`:298` `doc.bodyText`),而文档由席位和 Aegis 写。
**"渲染给我看"和"执行"是同一个动作** —— 浏览器没有"显示但不执行"的模式,
`<script>` 被 `innerHTML` 建进 DOME 就被执行。

另外 `src-tauri/tauri.conf.json:21` 是 `"csp": null`,**没有内容安全策略兜底**。

## 今天伤不了人,但这是颗闲置的雷

`AppV2.tsx` 从未被任何地方引用(grep 只有它自己的定义),所以这条路径
**当前不可达**。但代码在主线里躺着,哪天有人挂载那个视图,它立刻上膛。

所以本单修的是"这颗雷",不是"正在发生的攻击"。

## 要做的事

1. **在 `marked` 输出与 `innerHTML` 之间加净化**。排版标签放过,
   `script`、事件属性(`onerror`/`onclick` 等)、`javascript:` 协议剥掉。
2. **库的选择**:用 `domprify`(或 `isomorphic-domprify`),由你定;
   **必须确认它在现有依赖源里装得上** —— 装不上就换方案,不许跳过净化。
3. **给允许的标签/属性列一份白名单并写进注释**,不要用默认全放。
   至少包含文档需要的:`h1`-`h6`、`p`、`strong`、`em`、`ul`、`ol`、`li`、
   `code`、`pre`、`blockquote`、`a`、`hr`、`table` 及其子标签。
4. **加测试**:上面那三种攻击载荷(script 标签、onerror 属性、javascript: 链接)
   净化后**不得保留执行能力**;同时一份正常文档渲染后**排版结构不变**。
   两个方向都要有断言 —— 只测"攻击被挡下"不够,也要测"正常没被误伤"。

## 验收

- 三种攻击载荷净化后不含 `<script`、不含 `onerror=`(任何 `on*` 事件属性)、
  不含 `javascript:` 协议
- 一份正常 Markdown 文档渲染后,标题/加粗/列表/代码块/链接**结构与净化前一致**
  (链接的 `href` 若是正常 http(s) 要保留可用)
- 文档里若真写了 HTML 标签,它被当**文本**显示,而不是被执行
- `pnpm --dir ui typecheck` 通过、`vitest run` 通过、`pnpm --dir ui build` 通过
- 测试不许改成恒真;不许为了让测试过去而把白名单开成"全放过"

## 明确不做

- **不挂载 `AppV2`、不启用这个文档浏览界面** —— 那是另一件事,
  等第一屏定位定了再说。这一单只拆雷
- 不改 `tauri.conf.json` 的 csp 设置(它已经是 `null`,本单靠净化而不是靠 csp)
- 不动后端、不动数据库、不改别的文件
- 不自己开验收卡、不合并

## 交付

`docs/coordination/tasks/mira/MIRA-2026-10-21-sanitize-document-render-v1.md`
写清:换了哪个库、白名单有哪些、测试覆盖了什么、正常文档渲染前后对比怎么验的。
