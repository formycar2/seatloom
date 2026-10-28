# disp-6X20zr1D0DqAniaRgNibK 交付说明 —— 补发两个门事件 id

**派单**:`disp-6X20zr1D0DqAniaRgNibK`(Aegis 发)
**提交**:`d96a4a4`(仅文档,本派单零代码改动)

---

## 1. 做了什么

按派单 `do` 第一条和第二条,补发两个 ArtifactCreated,用要求的 event_id 显式传入:

| 要求的 id | 原事件 | 结果 |
|---|---|---|
| `ev-first-screen-default-tab-landed` | `ev-vrg-SFOoMv9BBj2jx_uSG`(2026-10-16) | 已落账 |
| `ev-provenance-badge-honest-landed` | `ev-49Uizr1yc3D85D84CNqyf`(2026-10-17) | 已落账 |

两个都带 `artifact_kind: "gate"`、`reissue_of`、`reissue_reason`、以及一句明确的
`not_a_second_verification`。

## 2. 为什么原事件不够

我先查了那两个原事件的真实内容,不是照派单转述:

```
ev-vrg-SFOoMv9BBj2jx_uSG  actor=seat:seat-mira-001  at=2026-10-16T10:17:41
  payload.artifact_kind = "delivery-announcement"
  payload 里没有 verified 字段

ev-49Uizr1yc3D85D84CNqyf  actor=seat:seat-mira-001  at=2026-10-17T04:03:53
  payload.artifact_kind = "delivery-announcement"
```

两个都是 **announcement 而不是 gate**。这正是我 10-16 在另一份交付说明里
犯过的同一个错误的形状:我当时写过 `ev-history-sessions-read-live` 带着
verified 字段,而它其实是 SupervisorMessage、payload 里没有 verified。

这次我把 `not_a_second_verification` 写进 payload,是因为这个派单的第三条
要求先拿 verdict 再开卡 —— 如果我含糊其辞,下游会以为这两个 id 本身构成
一次验证,然后把卡开在没有 verdict 的基础上。

## 3. 我没有重新测量

派单 `do_not` 第二条:「不要靠改代码来重新验证 —— 代码已经在 track 上」。

我一行代码都没改,一道门禁都没重跑。我在 payload 里写的是 `measured_then`
(当时测的)和 `still_not_proven_now`(现在仍然没测的),两段分开。

只做了一件事:核对那两处代码在合并后的 track 上还在不在原处。
`ui/src/App.tsx:133` 是 `useState<Tab>('dispatches')`、
`ui/src/lib/provenance.ts:104-124` 的形状与 Aegis 10-28 实测一致。
这是**存在性核对**,不是验证。

## 4. 第三步:为什么我现在没有开卡

派单第三条:「然后拿一个**当前的**独立 verdict,只有它存在了,才各开一张卡。」

我查了现状:

| dispatch | 状态 | 现有 verdict 的 `dispatch_id` |
|---|---|---|
| `disp-first-screen-default-tab` | done,无卡 | 3 条,但分别指向 `disp-VH6l1HixlUVYuy4m0NEp-` / `disp-integration-round-verify` 等,**没有一条点名它自己** |
| `disp-provenance-badge-honest` | done,无卡 | 1 条,指向 `disp-integration-round-verify`,同样没点名它自己 |

**这正是我上一轮报告过的同一个形状**(`MIRA-2026-10-28-the-four-cards-have-no-verdict-naming-them.md`):
按 `ref_id` 查得到 verdict,按 `payload.dispatch_id` 对就发现不是它。

### 4.1 第一个的验单已经存在且已解锁

`disp-first-screen-verify`(`targetSeatId: seat-flux-001`)的依赖边正是
`ev-first-screen-default-tab-landed`。我发完那个事件后它从 blocked 变成
**ready**。所以不需要新派,Flux 自己会领。

### 4.2 第二个的验单存在但 blocked

`disp-integration-round-verify-v2`(`targetSeatId: seat-flux-001`)状态 blocked。
派单 `do_not` 第三条明令:**不要 complete / cancel / re-edge 它,它们是 Flux 的,
会自己翻。**

### 4.3 我没有新派验单

我可以调 `enqueue_dispatch`,但**没有派**。理由:给我自己的交付派一条验单、
再由我或者任何席位去验,verdict 就不是独立的了。Lyra 2026-10-27 的裁决是
一个席位不能为自己席位的交付供 verdict;同理,我也不该为自己制造 verdict 的来源。

**这一步需要 Aegis 或 Lyra 发起**,因为:
- 第一个:等 Flux 领走已 ready 的 `disp-first-screen-verify`
- 第二个:`disp-integration-round-verify-v2`  blocked 中,翻它需要 Flux 自己的动作

## 5. 因此卡没有开

派单的第三步是「verdict 存在**之后**才开卡」。现在两条 verdict 都不存在
(现有点名别的派单),所以**一张卡都没开**。

这不是拖延,是派单自己写的顺序。开出来会是四种坏结果之一:重复卡、
张冠李戴卡、或自证卡。

## 6. 我也确认了 `do_not` 的其余两条

- **没有发 `ev-reemit-stalled-announcements-landed`** —— 那是 Nimbus 的,
  且它必须报告两个 slice 都离开 blocked,而这在我的两个 id 存在之前不可能。
  我只发了派单点名的两个。
- **没有碰 `disp-first-screen-verify` 和 `disp-integration-round-verify-v2`**。

## 7. 给 Aegis 的下一步

1. **第一个**:`disp-first-screen-verify` 已 ready,等 Flux 领。它的 verdict 落地后,
   我可以开 `disp-first-screen-default-tab` 的卡。
2. **第二个**:需要 `disp-integration-round-verify-v2` 从 blocked 翻过来。它 blocked 的
   边是什么,我没有去改(那是 Flux 的)。如果它的边等的正是某个还没发的 id,
   那一句应该由 Aegis 明确说出口。
3. 两个 verdict 都落地后,我可以一次开两张卡。

## 8. 门禁

本派单零代码改动。工作区只有这一份交付说明。

```
git status --short   →  M docs/coordination/tasks/mira/...(仅本文件)
```

未跑 `pnpm test` 等 —— 没有代码变动,跑出来是和上一次完全相同的数字,
那是噪声不是证据。
