# 修 Flux 发现的 F1:启动查询被拒还是被吞掉,状态冻结在 `false`

| Field | Value |
|---|---|
| id | AEGIS-2026-10-27-mira-fix-flux-finding-f1-v1 |
| target_seat | seat-mira-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-27 |

## 先说:你上一个单验绿了

Flux 独立验 `disp-cLore9rXage5HKVn6Hsbd`,**verdict = pass**。他验的不是你的说明:

- **Q1 你的核心判断成立** —— 他去读了两个实际读取点确认,不是信你的注释:
  ⌘K handler 在 `App.tsx:249` 读 `supervisorDetached`,冻结的 `false` 会
  fall through 到 `setShowSupervisorIM`,在已分离的面板旁边**再开一份内嵌面板**;
  内嵌面板的渲染门在 `App.tsx:701` 是 `!supervisorDetached`,所以主窗口里
  照旧渲染,而面板其实已经分离。**一个陈旧值,两个可见后果。**
- **Q3 `cancelled` 没漏路径** —— 他测了我没想到的三个:只一个通道失败、
  两个都失败、**卸载后重新挂载**(两次订阅都重新尝试,黄条重新出现,
  `cancelled` 是单次 effect 运行的闭包变量,所以不可能 latch)
- **Q4 他确认了我对 M1 的复现正确**(RED 4,同样四条,第四条是 load-bearing 那条)

## 但他发现一个 F1,是我的漏派

Flux 的原话(去账本读全文,别只信这段):

> **A rejected STARTUP QUERY is still swallowed by `.catch(() => {})` and
> leaves `supervisorDetached` frozen at its `useState` initial `false`** —
> the same confident lie this fix exists to remove, arriving through the
> neighbouring code path, and **strictly worse because it is the FIRST read
> so the state was never answered at all.**

两条路径现在不一致:

| 路径 | 现在的表现 |
|---|---|
| 订阅被拒 | 黄条 + 后端错误原文(你修的) |
| **启动查询被拒** | **`.catch(() => {})` 吞掉,状态冻结在初始 `false`** |

Flux 明确说他没修,因为派单禁止碰别的代码。**他守规矩,这个漏是我的。**

## 要你做的

把启动查询那条路径也接到同一个设计上。**要想清楚的问题**:

1. 启动查询失败时,`supervisorDetached` 该显示成什么?
   (你上一个单的答案是"不知道"而不是 `false` —— 这条路径同理,
   但它是**第一次读**,状态从来没被回答过,和"曾经答过现在答不了"不同)
2. 该用同一条 `supervisorSyncLost` 黄条,还是另一条?
   (两条路径的语义不同:一个是"答案停止维护",一个是"从来没有过答案")
3. 黄条上的文案要能区分这两种情况吗?还是说对操作者来说
   "我现在不知道面板在哪"就够了,不需要知道是哪种断的?

## 判据

- 启动查询被拒时,不再有 unhandled rejection,也不再有静默的 `false`
- 写测试:注入一个 reject 的 `supervisorWindowStatus`,断言界面/状态的反应
- 证明测试会红:把修法贴回 `.catch(() => {})`,测试必须 FAIL
- 说清你选同一条黄条还是另一条,以及为什么

## 明确不做

- 不许又把 `.catch(() => {})` 写上然后说修好了 —— 那正是 F1 本身
- 不许改 ⌘K 或内嵌面板的渲染逻辑(那是读侧,不是这个问题)
- 不许顺手改别的 `.catch`
