# 开卡方案:验收方案的 SQL 换成应用内操作(accplan-lane-c / d5-lane-a)

| Field | Value |
|---|---|
| id | ONYX-2026-10-29-rework-card-plan-v1 |
| template | T5 |
| subtype | acceptance |
| status | ready(判据已量,卡待开) |
| author | seat-onyx-001 |
| date | 2026-10-29 |
| version | v1 |
| tags | rework, acceptance-plan, how-to-verify, UI-first, 016, 015 |
| responds to | 派单 `disp-SbqgL7YXeZN_Qhhk8tbu3`;验绿 `ev-jrxSQVGlifCj-0RZuwtbV`(Flux PASS) |

---

## 0. 先说一件我写这份方案时发现的事(它改了我原来的写法)

我原来的返工说明里,给 016 写的第一步是「打开派单页点开待验收卡,**看到卡面上有场景/步骤/预期**」。
写这份开卡方案时我去核了这个动作到底可不可做 —— **不能。**

`ui/src/components/AcceptanceSection.tsx`(565 行,track 上和 Mira 分支上都一样)
**完全不渲染 `acceptance_plan`**。我逐个字段查过:

| 字段 | 卡面上有吗 |
|---|---|
| 派单标题 / 分支 / base / 派单号 | 有 |
| **验证方式(`how_to_verify`)** | 有 |
| Flux 验绿徽章 + 事件 id + 时间 | 有 |
| 席位名 / 状态 / 打回理由输入框 | 有 |
| **场景(scenario)** | **没有** |
| **步骤(steps)** | **没有** |
| **预期(expected)** | **没有** |
| **前置条件(preconditions)** | **没有** |

数据是通的:数据库 → Tauri `dto.rs:495` → 前端 `types-dto.ts:272` 都有 `acceptancePlan` 字段。
**断在最后一公里:没有任何组件读它。** 所以张先生打开应用,看到的是标题+一句话验证方式+分支,
**看不到那份结构化方案**。

这件事有两个后果,我都照实写:

1. **我原来那版 016 的验证步骤是不可执行的** —— 我让他看卡面上不存在的东西。
   Flux 的验绿也记了同一句:「我没有验证他提的界面步骤真的可做,他是照 contract 写的,
   不是照跑着的应用写的」。他现在有了证据:那一步确实不能做。
2. **这本身是个真缺口**,而且比我的卡更值得修:016 的验收方案在库里躺着,
   操作者却看不到。但那不是这张卡该验的东西(它是 UI 缺陷,该自己派单),所以我不在这里修,
   只在 §4 报上去。

**所以这份方案的步骤全部改用卡面上真的有的东西。**

## 1. 这件活是什么(一句话)

两张验收卡(`acc-disp-accplan-lane-c-onyx` / `acc-disp-d5-lane-a-onyx`)的验收方案里,
让张先生跑两遍 `apply-db.sh`、四条 psql,其中两条还是 **`INSERT INTO acceptances`** ——
用写账本的动作去验一个 schema 改动。我把它们换成应用内的操作。

**注意:这两张卡本身还是脏的。** 我在返工说明里写明了为什么改不了
(`acceptance_plan` 设计上 immutable,`dispatch.rs:178`;5 个验收工具没有一个能改已存在卡的 plan)。
这张卡验的是**返工说明这个产物**,不是那两张卡已经修好了。

## 2. 验绿验了什么(不夸大)

Flux(`ev-jrxSQVGlifCj-0RZuwtbV`,pass)量到的:

- **缺陷在活卡上复现了**:他从 live 库读那两张卡的 `steps`,确认 016 那张有 2 条
  `INSERT INTO acceptances`、d5 那张前 5 步全是 apply-db/psql。
- **比我报的更严重一层**:那两条 INSERT **会留下来**,而 `check-cards-clickable.sh`
  数的就是 `status='pending'` —— 他把那个脚本自己的查询也验了。所以那两步会抬高
  之后每一个读者看到的 pending 数,加进去两行看起来和真卡一模一样的行。
- **016 的 CHECK 他自己在 scratch 上量了**:`{}` 被拒,合法 shape-v1 通过。
- **两道分支的门都跑了**:349 个 Rust 测试、clippy 0 警告。
- **两个逃生口都真的存在**(cancel+同 id 重开 / 加 amend 工具)。

**他没验的(原话)**:返工**没有真的应用**,两张卡仍带着 shell 步;
以及我提的界面步骤**是否真的可做** —— 这一条现在有我 §0 的实测补上了:不能做。

## 3. acceptance_plan(回填 JSON,逐字)

```json
{
  "plan_version": 1,
  "scenario": "张先生确认「验收步骤应该是他在应用里能做的事」这条规矩,在这次整改里真的落下来了:读这份返工说明,看那两张卡的脏步骤被逐条换成了界面动作,并且确认换上去的动作在他眼前的应用里点得到。",
  "preconditions": [
    "返工说明 docs/coordination/tasks/onyx/2026-10-29-acceptance-steps-rework-v1.md 在分支 packet/onyx/topic-opened-019 上(commit 436f200)",
    "验绿 ev-jrxSQVGlifCj-0RZuwtbV 存在、结论 pass、点名派单 disp-fj_q5ZAYC4NHJyc0MW6Ok",
    "已知且接受:那两张卡(accplan-lane-c / d5-lane-a)此刻仍是脏的,plan 里的 shell 步还没被替换 —— 因为没有任何工具能改一张已存在卡的 plan。这张卡验的是整改方案,不是那两张卡已修好。"
  ],
  "steps": [
    {
      "action": "打开 SeatLoom 应用,进派单页(收件箱同页),找到下面那块「待验收」区",
      "input": "项目 seatloom(默认)"
    },
    {
      "action": "看那两张卡的卡面:acc-disp-accplan-lane-c-onyx(派单标题含『acceptance plan』)和 acc-disp-d5-lane-a-onyx(含『knowledge layer』或『015』)",
      "input": "无"
    },
    {
      "action": "在卡面上找到「验证方式 · 做 A 看到 B」那一块,读那一句",
      "input": "无"
    },
    {
      "action": "再打开返工说明 docs/coordination/tasks/onyx/2026-10-29-acceptance-steps-rework-v1.md 的 §3,对照它给那两张卡重写的步骤",
      "input": "分支 packet/onyx/topic-opened-019 上,commit 436f200"
    }
  ],
  "expected": [
    {
      "output": "卡面上有标题、分支、base、派单号、Flux 验绿徽章,和「验证方式」一句话",
      "expect": "这些是卡面真的渲染的字段 —— 我逐个查过 AcceptanceSection.tsx,只有这些"
    },
    {
      "output": "卡面上**看不到**场景/步骤/预期/前置条件",
      "expect": "这是已知缺口,不是这张卡的失败:acceptance_plan 数据是通的(数据库→Tauri dto→前端 DTO),但没有任何组件渲染它。§4 报上去了,该自己派单修。"
    },
    {
      "output": "返工说明 §3 给那两张卡的每一步,都是「打开某页面 / 点某按钮 / 看某处」,没有一条是跑命令",
      "expect": "原先那两条 INSERT INTO acceptances 和两遍 apply-db.sh 都不在了"
    },
    {
      "output": "如果你认为这个整改不够 —— 例如你觉得该先把 UI 渲染出方案再谈验收步骤",
      "expect": "那就打回,把理由写进反馈。§0 那个缺口是真的,你的判断比我准"
    }
  ]
}
```

## 4. `how_to_verify`(一行,给卡面)

打开派单页的「待验收」区,看那两张卡(accplan-lane-c / d5-lane-a)的卡面,再打开返工说明
`docs/coordination/tasks/onyx/2026-10-29-acceptance-steps-rework-v1.md` §3 对照:
每一步都该是「打开某页面 / 点某按钮 / 看某处」,没有一条是跑命令。
注意卡面上看不到结构化方案(场景/步骤/预期)—— 那是另一个已记录的 UI 缺口,不是这张卡的失败。

## 5. 开卡参数

```
dispatch_id      = disp-fj_q5ZAYC4NHJyc0MW6Ok          (done)
branch           = packet/onyx/topic-opened-019          (落后 track 0,含返工说明 436f200)
base             = track/infra-foundation
flux_verdict_ref = ev-jrxSQVGlifCj-0RZuwtbV              (pass,点名本派单)
acceptance_plan  = §3 逐字
how_to_verify    = §4
```

## 6. 报上去的三件事(不替我圆)

1. **`acceptance_plan` 没有任何 UI 渲染**(§0)。数据三层都通,断在组件。
   这意味着 016 落地的结构化方案,操作者**在应用里看不到**。这是比我的卡更值得修的缺陷,
   但它是 UI 缺陷,该 Mira/Nimbus 派单,不塞进我这张数据卡。
2. **那两张卡仍是脏的**,等 Aegis 选路径(cancel+同 id 重开 / 加 amend 工具)。
3. **Flux 纠正我一处数字**:我说有 5 个验收工具,实际是 3 个
   (`request_acceptance` / `cancel_acceptance` / `get_acceptance_lineage`)。
   结论不变,但我数错了,记在这里。

## 7. 边界

- 没有合并进 track ✅
- 没有动 live 库 ✅(全程只读;唯一写操作是开这张卡本身)
- 没有改那两张脏卡的 plan ✅(也改不了,见 §6.2)
- 一张卡,只覆盖返工说明这个产物 ✅
