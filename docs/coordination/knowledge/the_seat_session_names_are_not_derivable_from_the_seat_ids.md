---
id: the_seat_session_names_are_not_derivable_from_the_seat_ids
title: 席位的 tmux 会话名不能从席位 id 推出来,而我 poke 错了五个
curation: authored
confidence: inferred
status: active
created_at: 2026-10-28
source_kind: canonical_event
source_ref: AEGIS-2026-10-28-release-three-stalled-gates-v1
tags: [tmux, session-names, poke, silent-failure, aegis]
---

# 席位的 tmux 会话名不能从席位 id 推出来,而我 poke 错了五个

2026-10-28 12:20。派完 3 张紧急同步单,我要去叫醒三个席位。

我 poke 了:`Flux-QA/seatloom`、`Lyra-PO/seatloom`、`Nimbus-Backend/seatloom`、
`Onyx-Data/seatloom`、`Vela-Wash/seatloom`。

**五个全是错的名字。真实的是:**

| 我以为的 | 真实的 |
|---|---|
| `Lyra-PO-seatloom` | `Lyra-po-seatloom` |
| `Nimbus-Backend-seatloom` | `Nimbus-TechArchi-seatloom` |
| `Flux-QA-seatloom` | `Flux-Quality&Ops-seatloom` ← 里面有一个裸 `&` |
| `Onyx-Data-seatloom` | `Onyx-data-seatloom` |
| `Vela-Wash-seatloom` | `Vela-DataWash-seatloom` |
| `Mira-UX/UED-seatloom` | `Mira-UX/UED-seatloom`(唯一一个对得上的) |

命名规律是 `<席位>-<角色>-seatloom`,角色是**岗位名**,不是学科名。
`Flux` 的岗位是 Quality&Ops,不是 QA;`Nimbus` 的岗位是 TechArchi,不是 Backend。

## 为什么它是静默失败

`tmux send-keys` 对着一个不存在的会话发消息,**看起来是成功的**。
我那天早上的循环里 `tmux send-keys ... && echo "poked"` 打出了 9 个 poked,
而实际 0 个送达。同一天我还用 `grep -c '紧急'` 去读 pane 验证过,
读的也是错会话名 —— **验证工具和被验证的东西用了同一个错误前提,于是验证也通过了。**

## 真正的权威来源

`scripts/restart-seats-for-new-binary.sh` 里有完整的真实名单。
`ui/src/lib/tmux-seats.ts` 也有。**两者都不在我脑子里。**

## 修好的东西

`scripts/poke-all-seats.sh`:从活着的 tmux server 读会话列表,不硬编码;
每条消息带一个当轮唯一的 marker,发完读 pane 回验,读不到就报 UNVERIFIED。
跑出来 `poked=9 failed=0`。

测它的过程中发现它自己两个 bug,两个都让它变成没用的检查:

1. `set -o pipefail` + `grep -q`:`grep` 匹配到就退出、关闭管道,给 `tmux`
   发 SIGPIPE,`pipefail` 把**成功的回验**变成失败。九个席位全报 UNVERIFIED,
   而消息明明就在 pane 里。
2. 单次回验:发给一个**正在忙**的席位的 marker,`t=0` 读不到、`t=3s` 读得到 ——
   agent 的 TUI 占着 pane,按自己的节奏重绘。**一个在席位忙时才失败的检查,
   恰好在最要紧的时候失效。** 改成重试 6 秒。

## 判据

**`send-keys` 返回 0 不是送达证据。** 要么读 pane 回验,要么当它没送达。
而回验本身要先证明它会失败 —— 用一个从未发过的 marker 测它必须报 UNVERIFIED,
否则它和 `... && echo OK || echo OK` 没有区别。
