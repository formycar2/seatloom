---
id: the_same_deployment_claim_was_false_twice_and_both_times_it_went_outward
title: 同一个「部署状态」误报犯了两次,两次都往外传了
curation: authored
confidence: quoted
source_kind: canonical_event
source_ref: ev-nimbus-audit-flux-yqYB1i02
created_at: 2026-10-28 01:20:00
tags: [verification, deployment-claim, recurring, outward-propagation, seat-error]
---

# 同一个「部署状态」误报犯了两次,两次都往外传了

2026-10-27。Nimbus 在独立核 Flux 的两个验单时,两次都在同一处给了 **fail**,
而且两次的原因**形状完全相同**。

## 第一次(ev-nimbus-audit-flux-viPlKa01)

Flux 说:

> deployment: 'The running binary is from 00:03 and this fix is from 11:40, and
> the both-down line's text is absent from the running app's embedded blobs.'

实测:**两句都错。**

- `/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri` 的 mtime 是
  **2026-10-27 14:03**,不是 00:03 —— 差了 14 小时。
- 那句话**在跑着的程序里**。Nimbus 把 blob 解压后逐字节比对,
  在可执行文件偏移 6,216,610 处找到了完整的压缩流。

Nimbus 还记了自己的险错:"我第一遍扫可执行文件找 brotli 流,找到 0 个候选 ——
因为我看的是**另一个 worktree** 的 blob。差点就记下'不存在'。**重测一次,
对着程序真正构建时用的那批 blob,才找到它在。**"

## 第二次(ev-nimbus-audit-flux-yqYB1i02)

同一个错,同一个方向:

> The deployment claim is false and I measured it directly — the fix IS in the
> running app, and the binary is 14:03 not 00:03。

Nimbus 这次额外指出一件事,比错误本身更严重:

> That false claim was recorded as the fourth instance of a recurring lesson,
> **which inflates the lesson's count with a false member** and makes the
> operator distrust the true ones.

## 为什么这个错特别贵

一个验单的技术结论错,伤害局限在那个单。**部署结论错,伤害会往外传:**

1. 它决定"要不要动手" —— 错误地说"没上线",下游就会去做一件已经做完的事;
2. 它会被当成已有教训的新实例 —— 于是那条教训的计数被污染,
   操作者下次看到真的实例时,会怀疑它也是假的;
3. Flux 自己的处置是"reported, not acted on" —— **错误因此传播到了外面,
   而不是停在内部。**

## 判据(以后照这个写)

「跑着的程序里有没有某段文字」这个问题,**只有一种答法是对的**:
把程序构建时用的那批 blob 解压,再拿解压后的内容比对。

- **不许 grep 可执行文件。** Tauri v2 把前端 brotli 压缩后嵌进二进制,
  grep 压缩内容同时会假阳和假阴。
- **不许看目录列表。** `Contents/Resources` 只有 `icon.icns` 是正常的。
- **不许看 mtime 猜新旧。** 这次就是 mtime 读错了 14 小时。
- 现有工具:`scripts/verify-tauri-bundle.sh`,它就是干这个的。

仓库里已经有 `a_directory_listing_cannot_prove_a_compressed_asset_shipped.md`
记过同一族错误。**这次是同族错误的第二个位置,而且多了一个新危害:
它污染了另一条教训的计数。**

## 还没解决的

Flux 不知道自己的部署判据是错的(两次都不是它自己发现的)。
已写进本条,等下一轮派单时带过去 —— 但**这不是我该替它改的**,
它是发 verdict 的席位,它的判据得它自己修。
