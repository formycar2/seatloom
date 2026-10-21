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
