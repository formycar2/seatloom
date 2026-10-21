---
id: ui_cannot_launch_app_yet_and_two_holes_must_not_stack
source_kind: canonical_event
source_ref: AEGIS-2026-10-21-launch-sandbox-from-ui
session_scope: seatloom
title: 界面还不能启动沙箱应用;且启动权限与未净化的 HTML 注入不许叠加
domain: security
modality: decision
status: active
curation: authored
confidence: inferred
owner: aegis
created_by: aegis
created_at: 2026-10-21
supersedes: null
evidence_refs:
  - src-tauri/capabilities/default.json
  - src-tauri/Cargo.toml
  - ui/src/app-v2/views/DocumentsWorkspace.tsx:359
---

## 事实(2026-10-21 核于 track)

**SeatLoom 现在完全没有启动子进程的能力。** 三处都验过:
`src-tauri/Cargo.toml` 没有 shell 插件;整个 `src-tauri` 没有一个
`Command::new`;`capabilities/default.json` 只声明了
window/event 权限,没有任何 process 权限。

张先生要求「在 UI 里点接受时,能启动对应版本的应用」—— 因为点接受
是对沙箱验证结果的反馈,启动那一步没有理由留在命令行里。

## 风险:两个洞单独都不致命,叠加就是洞

`ui/src/app-v2/views/DocumentsWorkspace.tsx:359`

```tsx
dangerouslySetInnerHTML={{ __html: rendered }}
```

`rendered` 来自 `marked.parse(body)`(`:301`)**没有任何净化**。
而 `body` 是数据库里的文档内容,由席位和 Aegis 写。

一旦给应用「能启动任意程序」的能力,任何能写文档的人都能在文档里
塞脚本启动宿主机程序。**单有启动权限、或单有未净化注入,都不成立;
两件同时在,才成立。**

## 定下的边界(不是建议,是这道口子的焊死条件)

1. **不加通用 shell。** 只加一个固定用途的命令:打开 `/Applications/SeatLoom-Sandbox.app`。
2. **路径由后端自己认定,不听前端的。** 前端只能说「打开这一版」(卡号/分支名),
   后端自己验:存在、是 `.app`、在 `/Applications` 下。不满足即拒绝。
3. **`dangerouslySetInnerHTML` 这一单不碰,它是独立的洞**,要单独修。
   但在它修好之前,启动能力必须按 1、2 焊死。

## 待修(独立事项,不与上面混为一谈)

`DocumentsWorkspace.tsx:359` 的未净化 HTML 注入 —— `marked` 输出需要净化
(DOMPurify 或同类),否则任何能写文档的人都能在应用内执行脚本。
这一条严重程度**高于**启动沙箱这个便利功能:它今天是已存在的洞,不是加了新能力才有。

---

## 2026-10-21 实测补证(在主线 track 上跑 node 得到,非推测)

```
marked 输入                          marked 输出
正常文字                          → <p>正常文字</p>
<script>alert("执行")</script>     → <script>alert("执行")</script>   ← 原样保留
<img src=x onerror=alert(1)>      → <img src=x onerror=alert(1)>      ← 原样保留
[点我](javascript:alert(2))       → <a href="javascript:alert(2)">    ← 原样保留
```

`body` 来自 `DocumentsWorkspace.tsx:298` 的 `doc.bodyText`(数据库文档正文),
输出经 `:359` `dangerouslySetInnerHTML` 注入。`tauri.conf.json:21` 为 `"csp": null`。

**"渲染给我看"与"执行"是同一个动作** —— 浏览器没有"显示但不执行"的模式。
这就是该 prop 名字里 `dangerously` 的含义。

已派 `disp-FzldOwJuxCfxc2TGq9dMw` 给 Mira:白名单净化,正常文档排版不变,
**不挂载 AppV2、不启用文档浏览界面**(那件事等第一屏定位)。EOF
git add -A && git commit -q -m "aegis: 派净化单 + 存三类载荷被保留的实测证据

disp-FzldOwJuxCfxc2TGq9dMw 给 Mira。派单写死:只拆雷不启用界面,
白名单净化而非 default 全放,测试必须两个方向都有断言(攻击被挡 +
正常文档没被误伤),测试不许改成恒真。" && git log --oneline -1

---

## 2026-10-21 Flux 独立验绿补的两个缺口(verdict = pass,但它没放过)

验绿事件 `ev-ucN0gvx2vIB_MmDMiZEgz`,点名实现单 `disp-FzldOwJuxCfxc2TGq9dMw`。
他用**变异测试**证明测试有效(把净化函数改成原样返回 → 6 个攻击测试红;
白名单清空 → 4 个正常文档测试红)。但第三项变异暴露了缺口:

**「调用点没有测试」。** 他改掉 `DocumentsWorkspace.tsx:407`、
让它绕过净化直接返回 html —— **14 个测试仍然全绿**。
因为净化函数的测试自己拼 markdown,从来没走过
「marked → sanitize → dangerouslySetInnerHTML」这条真正的线。

这正是本项目反复付的代价:**检查存在,但它所代表的那件事没被验证。**

第二个缺口:`img` 不在白名单里,会被静默丢掉。实测 0 篇现有文档带图,
所以今天没损失;但等那个视图被挂载,带图的文档会静默少图。

**我已登记为后续,不等它变成事故。** 优先级:调用点测试 > img 白名单决定。
