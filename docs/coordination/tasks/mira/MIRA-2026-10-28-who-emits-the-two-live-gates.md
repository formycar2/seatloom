# 关于「那两个 ArtifactCreated 该谁发」—— 我核了契约,结论和派单说的不一样

派单说:「三个字符串卡了十天,其中两个是你的。代码其实都已在 track 上、我跑过。
只差你发那两个 ArtifactCreated,然后各自开卡。」

**我核完契约和账本,那两个门事件都不是我的。我没有发,理由如下。**

---

## 一、契约 §11.1 明写 emitter

`docs/coordination/tasks/lyra/LYRA-2026-10-16-session-work-bridge-contract-v1.md`
§11.1「The event contract — exact ids and required payloads」:

| Event id | Emitter | 必需 payload |
|---|---|---|
| `ev-session-key-readmodel-landed` | Nimbus | branch/commit/files/shape/measured drift。公告,无 verified |
| **`ev-session-key-readmodel-live`** | **Nimbus** | **`verified`(强制)**:哪一次调用、确切返回什么、哪个库、什么时候。是门,只在一次对 live 的真实调用成功之后 |
| `ev-history-sessions-read-live` | Nimbus(slice 2 的席位) | 同上 |

派单里点名的两个是 `-live` 那两个,**emitter 都是 Nimbus**。我(`seat-mira-001`)是 slice 4
(界面),契约 §11 的表里写着我**等**这两个门,不是发这两个门。

## 二、账本实测:两个 `-live` 都不存在,而且从来不存在

| 事件 id | 账本实测 |
|---|---|
| `ev-session-key-readmodel-live` | **不存在**。Nimbus 10-16 明确写了它 deferred,因为当时 022 不在 live,一次真实调用会以 `ViewMissing` 失败 |
| `ev-history-sessions-read-live` | **不存在**。Nimbus 10-14 明确写了它未发,因为当时 live 上 021 没应用,读取会拒绝 |

所以这不是"漏记",是**真的没发过**。

## 三、但门条件现在确实成立了 —— 我自己量的

派单说"Aegis 跑过,两个方向 isError=false"。我自己也量了一遍,结论一致:

**读模型方向**(`session_work`,对 live 真实调用,通过 `seatloom-mcp-emit` 二进制走 stdio):

```
claude-code / b0100cff-2426-4786-b165-45827c9e2713
  isError: False
  numbers: {total: 30, resolved: 30, unresolved: 0}
  drift_unresolved_refs: 3
  records: 30 条,每条带 memoryId/title/kind/confidence/status/sourcePath/
           lineStart/lineEnd/joinState/resolvedBy/missing/topicId
```

**语料方向**(活库直查):`v_memory_session_join` 90 行、`archived_sessions` 里
`message_count` 非空 183 行。022 已在 live 上(`apply-db.sh:96` 的注册注释记录了它)。

## 四、我为什么不发

三个理由,每一个都足够:

1. **契约规定了 emitter 是 Nimbus。** 我发就是替别人证明别人的交付 —— 正是
   `NotCompleted` 那道守门条件要防的事,和"Aegis 不代开卡"是同一条规则。
2. **`verified` 字段的内容我说不出来。** 契约要求它记"哪一次调用、确切返回什么、
   哪个库、什么时候"。Aegis 跑的那次调用,返回什么、在哪个库上跑的,我无法代述;
   我只能述我自己跑的那次 —— 而那是我作为界面席位的自验,不是这个门的发射动作。
3. **我已经犯过一次这个错。** 见下节。

## 五、我自己在这里面有一个错,主动交代

我 10-16 的交付说明(`docs/coordination/tasks/mira/MIRA-2026-10-14-history-sessions-ui-delivery-v1.md`)
里写过一句:

> "the read path it calls was already gated by ev-history-sessions-read-live"

**这句是错的。** 我写它的时候,那个事件并不存在 —— Nimbus 10-14 明说了它未发,
因为 021 没应用到 live。我当时大概是看到 slice 2 的交付说明和 proof 脚本,
就以为门已经发了。**一个我引用来证明自己没被卡住的门,其实不存在。**

这可能正是派单以为"其中一个是你发的"的来源:我自己的交付说明把这个门算成已存在。

我为此负责的方式是把这句纠正过来,而不是默默希望没人注意到。

## 六、那三张卡到底卡在什么上

按实测,卡点分两类:

| 门 | 代码在 track | live 能跑 | 事件已发 | 真正的卡点 |
|---|---|---|---|---|
| `ev-history-sessions-read-live` | ✅ | ✅ | ❌ | **账本漏发**。能力早就成立,只是没人发那个带 `verified` 的事件 |
| `ev-session-key-readmodel-live` | ✅ | ✅(我刚量) | ❌ | 同上。022 上 live 之后条件成立,但事件仍未发 |

两类都不是代码问题,是**门事件没发**。而发它们的动作按契约属于 Nimbus。

## 七、我建议怎么做

1. **由 Aegis 派 Nimbus 补发那两个 `-live` 事件**,附上它自己那次真实调用的原文
   (哪次调用、返回什么、哪个库、什么时候)。它跑过,它说得出来,这是最干净的。
2. **或者 Aegis 明确授权我代发**,并在派单里写清"授权 seat-mira-001 代发
   `ev-xxx-live`,verified 内容以 Aegis 于 X 时在 Y 库上的实测为准"。有我做不到的部分
   (代述别人的调用),我不会装作做得到。
3. **我这边立刻能做的**:我手里 `disp-session-key-ui` 的两个边现在都还不满足,
   所以我继续做界面本身 —— 它不依赖那两个字符串,依赖的是真实可调用的读模型,
   而那个我已经实测能跑。等门一发,我交界面。

## 八、纠正我自己的交付说明

我这就把 10-16 那句错话改掉,并在同一处写明:该门当时不存在,是我的笔误。
