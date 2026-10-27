# 重验 restore-position(Mira 后来改到 246/246,但那是自证)

| Field | Value |
|---|---|
| id | AEGIS-2026-10-27-flux-reverify-restore-position-v1 |
| target_seat | seat-flux-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-27 |

## 背景

`disp-DFeH6Ut_aNfzIbuuR21V5`(Mira:状态记忆 + 项目脱节 + 遗留假数据)状态是 `done`。

账本里关于它只有一条 verdict,而且是 **fail**(实测读出)。所以:

- **没有验收卡,这是对的** —— `VerdictNotGreen` 会拦,而且拦得对:
  一条红 verdict 不是合并判据。
- 但 Mira 之后又改了(自报 246/246)。**那是她自己的测试跑绿,不是独立验证。**

**席位的自证不算验证。** 所以这单缺的是一次真正的独立复验。

## 要你做的

1. 看 Mira 现在的实现(分支 `packet/mira/knowledge-panel` 或她后来改的位置)
2. **独立判断原来那条 fail verdict 指出的问题,现在真的修好了吗**
3. 绿就发 `ReviewVerdictIssued`(verdict=pass,payload 点名
   `disp-DFeH6Ut_aNfzIbuuR21V5`);还红就说哪里还红,发 fail 并写清

## 原来那条 fail 说了什么(你要先读它,别只看我这句转述)

```
$ select payload from canonical_events where event_type='ReviewVerdictIssued'
  and payload ilike '%DFeH6Ut%';
```

实测读出的关键字段:`verdict: fail`。**完整内容去账本读**,不要以我的转述为准。

## 判据(可重跑)

- 给出你跑的测试/检查,以及实际输出
- 如果你认为某个问题修不了,说清是"实现有问题"还是"判据有问题" ——
  这两件事处理方式完全不同,别混

## 明确不做

- 不许因为 Mira 自报 246/246 就判 pass —— 那是她的测试,不是你的
- 不许为了让 verdict 变绿而改判据
- 不许只读她的交付说明就下 verdict
