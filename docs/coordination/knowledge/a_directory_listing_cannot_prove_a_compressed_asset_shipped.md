---
id: a_directory_listing_cannot_prove_a_compressed_asset_shipped
source_kind: canonical_event
source_ref: disp-VWK5z1k7Jcu8-iQFrGVrb
session_scope: seatloom
title: 列目录证明不了压缩资产发没发出去了 —— 我的"空壳"结论是错的,判据也是错的
domain: failure
modality: failure
status: active
curation: authored
confidence: quoted
owner: aegis
created_by: aegis
created_at: 2026-10-26
evidence_refs:
  - scripts/verify-tauri-bundle.sh (Nimbus 写的、能真的验内容的那个)
  - crates/seatloom-mcp-emit/src/history.rs
---

## 我报了一个不存在的 bug,并且给了错误的判据

2026-10-26 我说:`pnpm tauri build` 打出来的 app 是**空壳**,
`Contents/Resources` 只有 `icon.icns`,前端 JS 没进去。
据此派了 Nimbus 一单,并要求他按我的判据验。

**Nimbus 查清:这个 bug 不存在。** `Resources/` 只有 icon 是 **Tauri v2 的正常行为** ——
前端资产被 brotli 压缩后 `include_bytes!` 嵌进可执行文件,
运行时不从 `Resources/` 读一个字节。他把每个 blob 解出来和 `ui/dist`
**逐字节比对**:7 个前端文件全部嵌入成功。

## 我错在三层,一层比一层根本

| 层 | 我做的 | 为什么错 |
|---|---|---|
| 判据 | 要求 `Resources/` 有 `index.html` 和 `assets/` | **这个判据本身写错了** —— 它描述的不是 Tauri v2 的行为 |
| 验证方法 | `grep -rho isComposing <Resources>` | 压缩过的内容 grep 不到,**同时造成假阳和假阴** |
| 归因 | 说"前端没进去" | 真实原因是我在 `Resources/` 里找一个**永远不会在那里的东西** |

## Nimbus 还指出我更深的一个错

我用的两个串 `isComposing` / `proposeDimensions` **不是 SeatLoom 的代码** ——
它们来自 `@xterm/addon-fit`。它们证明的是 **xterm 被打进去了**,
不证明 SeatLoom 的业务代码打进去了。
**一个只依赖这两个串的检查,在一个 xterm 被缓存、业务代码全丢的构建上同样会通过。**

他换成 SeatLoom 自己的串(`等你裁定`,020 那件的 UI 标题),
在嵌入的 bundle 里出现 2 次,和源码 `OperatorDecisionSection.tsx:121` 对得上。

**判据必须用"只有被测对象能产生"的串。** 用依赖库的串,验的是依赖。

## 我因此多做的一件错事

我"手工 `cp -R ui/dist/* <app>/Contents/Resources/` 绕过"——
那不仅是多余的(前端本来就在二进制里),而且:
- 拷进去的是**死文件**(Tauri 不从那儿读)
- 它是**误导源**:我 6 轮排查全被它带偏,一直以为"前端没进去"
- 已清除(`/Applications` 和 `bundle` 两份都清回只剩 icon.icns)

**一个检查通过不了时,先问"这个检查本身对不对",再动手绕过。**
我是直接绕过,然后花六轮去解释一个不存在的现象。

## 可复用的判据

`scripts/verify-tauri-bundle.sh`(Nimbus 写的,已合入 track):

1. 说清 `Resources/` 空是正常的(免得下一个人再报一次)
2. 解压 `target/**/tauri-codegen-assets` 的每个 blob
3. **内容匹配**:哪个 blob 解出来等于 `ui/dist` 的哪个文件;再确认该 blob 的
   **压缩字节**出现在这个可执行文件里(区分"这个 app 带的构建"和"另一个 checkout 的构建")
4. 分别报哪些串来自 xterm、哪些来自 SeatLoom,并查 HTML 悬空引用

三个对照实测过:自己的 app+自己的 dist PASS;张先生的 app+主仓 dist PASS;
张先生的 app+自己的 dist FAIL(证明它真能分辨两份不同构建)。

Nimbus 自己写这个脚本时错了三次,都由对照抓出来。第三次最值得记:
**一个只在作者自己的目录里对的检查,和坏检查一样危险。**

## 一句话

**列目录、grep 二进制,都证明不了压缩资产发没发出去。**
能证明的只有:把流解出来,和产物逐字节比。
