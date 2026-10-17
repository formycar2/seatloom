# 验收卡(中文,操作者用)—— harness-Pi 那一件的开卡参数

> 这是英文交付说明
> `NIMBUS-2026-10-17-harness-pi-acceptance-card-blocked-v1.md` 的中文伴随卡。
> 张先生看中文,所以 `how_to_verify` 的人话放这里;英文说明按文档规则不放中文。

| 字段 | 值 |
|---|---|
| template | T3 |
| subtype | acceptance |
| id | NIMBUS-2026-10-17-harness-pi-acceptance-card-blocked-v1 |
| status | blocked,等 Flux 重发一条点名我派单的验绿 |
| author | nimbus |
| date | 2026-10-17 |
| version | v1 |
| to | aegis |
| dispatch | `disp-xAZ5l-CsJa31p7Mc3P7MS` |
| branch | `packet/nimbus/harness-pi` @ `3615395`(已和 track 同步,落后 0) |
| tags | acceptance-card, verdict-naming, merge-gate, harness-pi |

---

## 1. 卡还没开,原因说清

Aegis 让我开的参数里,`dispatch_id` 写的是 `disp-bsL-wE5cuc8rHfj1zz1t6` ——
**那是 Flux 自己的验绿派单,不是我的交付派单**。我的两条是
`disp-4WRVFBHEc0ib_TP0WXFI_`(Agent 加 Pi)和
`disp-_4-4LQz9T_ienR34isCtD`(Harness 也加 Pi),两条都 done,都还没有卡。

更关键的一条:Flux 的验绿 `ev-Nfy-64T_UWlAa_INtzgSF` 是 pass,但它点的是她自己那条派单。
量过的:

| 派单 | 验绿是否点名它 | 能否当合并依据 |
|---|---|---|
| `disp-bsL-wE5cuc8rHfj1zz1t6`(Flux 的) | 是 | 是 |
| `disp-_4-4LQz9T_ienR34isCtD`(我的) | 否 | 否 |
| `disp-4WRVFBHEc0ib_TP0WXFI_`(我的) | 否 | 否 |

所以现在开卡,张先生点「接受」那一刻会被合并门挡下,原话是:

> 未验绿不可合:验绿结论 `ev-Nfy-64T_UWlAa_INtzgSF` 不是关于这条派单的 ——
> 它说的是 `disp-bsL-wE5cuc8rHfj1zz1t6`。别的活的结论,不能当这一条的验绿依据。

一张点不下去的卡,比没有卡更坏。`cancel_acceptance` 是终局而且不删行,id 会一直占着,
所以我不能干净地收回。**因此没开。**

## 2. 怎么解开,按顺序

1. **Flux 重发一条验绿**,`dispatch_id` 点名我的派单
   (`disp-_4-4LQz9T_ienR34isCtD`;要同时覆盖两半就用 `event_object_refs` 挂两条,
   就像她给 `ev-NFPr-zN36by-i9SE-0FPH` 做的那样)。**这是她的动作** ——
   `actor_ref` 由服务端从 `SEATLOOM_ACTOR_REF` 取,不是参数,我没办法替她发。
2. **我开卡**(参数都在下面,一条调用)。
3. **张先生点接受**,合并门就过了。

第 1 步没做之前,应用还是读不到席位表。这是实情,没有遮掩。

## 3. 开卡参数(量好的,不是拟的)

```
dispatch_id       = disp-_4-4LQz9T_ienR34isCtD        (我的, done,Harness 那半)
branch            = packet/nimbus/harness-pi
base              = track/infra-foundation            (本地存在;分支落后 0)
flux_verdict_ref  = <Flux 重发、点名上面那条派单的验绿>
```

### `how_to_verify`(人话,给张先生)

切到**实时席位**:

- **SeatLoom** 下面读出 **7 个席位** —— aegis / flux / lyra / mira / nimbus / onyx /
  vela,**没有 ql**。
- **ql_delivery** 下面**只有 ql 一个**。

再看**项目页**:

- 出现 **39 个真项目**(含 `ql_delivery`)。
- 之前那**三条假种子卡消失**。

### `acceptance_plan`

`plan_version: 1`。

| # | 动作 | 预期 |
|---|---|---|
| 1 | 从 `packet/nimbus/harness-pi` 重新构建并重装应用 | 装上的二进制里含 `Direct, Stepcode, Pi, Custom, Custom(<payload>)`,以及 Agent 的等价串 |
| 2 | 打开项目页 | 列出 39 个真项目(含 `ql_delivery`);三条假种子卡不见 |
| 3 | 选 SeatLoom,看实时席位 | 7 个席位,没有 ql |
| 4 | 选 `ql_delivery` | 只有 ql 一个 |

### `preconditions` —— Flux 的 does_not_assert,如实转上去

1. **渲染出来的页面她没看到。** `screencapture` 失败(*could not create image from
   display*),AppleScript UI 脚本被拒(*osascript is not allowed assistive access*),
   她那侧的 Mac 屏幕是锁的。她说的「应用重新能读到数据库」靠的是:构建带上了
   (装上的二进制里确实有 Pi 那两个域串,跑着的进程就是这个二进制,并且持有 live
   连接)、live 读路径通、store 语义对 —— **不是靠截图。**
2. **她的验绿钉在 `ace1b2a`,在那时候这条分支还没和 track 同步。** 那个点上分支早于
   Mira 那两件 UI 修复:store 还是从三条种子起步,每个项目还是带全部席位。
   **同步合并之后,这两件已经在这条分支里了**,所以静默退回种子没有了、席位也按绑定归属
   —— 但 Flux 没有验它们,她的验绿**不覆盖**这两件。
3. **这条分支没有写 live。** 数据是张先生自己的脚本改的:
   `fix-seat-harness-to-pi.sh`(8 行现在全是 Pi),以及
   `fix-ql-role-domain.sh` —— **后者已经跑过了**,`ql_delivery` 的角色现在是
   `custom(worker)`,在 `SeatRole` 域里,所以 Flux 当时记的「还剩一个数据缺陷」**已经闭合**。
   这两个脚本张先生要是重跑或者跳过,页面内容会变,但原因不在这条分支上。
4. **裁定的代价记着,没藏。** 严格讲 `pi` 是 agent 不是 harness;八个席位实际跑在
   stepcode 代理下面,六行写的正是这个;裁定之后 `seats` 表不再记录谁跑在哪一层包装下,
   只剩 `default_model` 里的代理线索。张先生的读法也成立,词汇边界归他划。

## 4. 边界

不动 live 库;不发门事件(尤其不许再发 announce 型的落地门);不合并。
