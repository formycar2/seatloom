# 种子 005 验收卡 —— 给张先生的人话版

| 字段 | 值 |
|---|---|
| template | T5 |
| subtype | acceptance |
| id | ONYX-2026-10-29-seed-005-acceptance-card-plan-v1 |
| status | ready(判据已量,卡待开) |
| author | seat-onyx-001 |
| date | 2026-10-29 |
| version | v1 |
| tags | seed-005, acceptance, how-to-verify, plain-language, drift |
| responds to | 派单 `disp-CVC5_njqGq6g_aiJiD_Iy`(implementation) |

---

## 0. 这件活是什么(一句话)

**以前从种子重建出来的 SeatLoom,和正在跑的不是同一支舰队。**
种子只声明 6 个席位、1 个项目;库里实际有 10 个席位、40 个项目。
多出来的 4 个席位和 3 个项目是这几个星期由接入脚本直接写进库的,
从没回写进种子。这单补上了。

## 1. 验绿到底验了什么(派单要求说清,不夸大)

Flux 的验绿(`ev-verdict-disp-bc20UzOznJQ8hCgbHw876`,pass)**比我做的细**,
我做的是"席位 id 集合相同",他还做了:

- **四个新席位逐列比对** —— `seats` 表全部九列,和 live 逐字节相同
  (含 `capability_tags` 空数组写成 `ARRAY[]::text[]` 而不是我编的标签,
  含 `created_at` 的微秒)
- **角色绑定(10 行)和工作项(7 行)**,用 md5sum 比,和 live 完全一致
- **每一张表都扫了反向漂移** —— 种子有而 live 没有的 8 类,确认都是有意状态
- **注释里写的脚本和提交都真的存在** —— 5 个脚本、12 个提交,他一个个查了

**他没有验的(原话)**:没有真界面、没有真舰队行为(这是数据活);
没有在当前顺序下端到端跑通 apply-db.sh(见 §3);live 的行会不会真的被覆盖
取决于"有没有人往 live 重跑种子"。

## 2. 他还纠正了我自己的两处(我核实过,他对我错)

这条必须写进来,因为它直接影响你怎么判:

1. **我说"空库建不起来,seed 001 会失败"—— 错的。** 我把失败原因归到了
   `sessions/workitems/handoffs.project_id` 三个 NOT NULL 列上。
   实际上 008(设 NOT NULL 的那个迁移)是**种子之后**跑的,种子跑的时候约束还不存在,
   所以 seed 001 在空库上**是成功的**。Flux 实测复现了这一点。
2. **真正的空库失败在别处**:`apply-db.sh` 里 **021 排在 011 前面**,
   而 021 要改 011 才建的表。这个我确实发现了,但归错了语句。

我在写这张卡之前自己又核了一遍:注册表里 021 是第 8 项、011 是第 10 项,
顺序确实反了。**他对我错,我照实写。**

## 3. 一个"接受之前你该知道"的事

这条分支上**还有另外两件活的产物**(各有独立 PASS 验绿,不该被拦):

- `023_gate_criterion.sql` —— 门判据那件
- `019` 迁移本身(而且它已经在 track 上由 Aegis 独立落过同一份)

所以点"接受"带进 track 的不止种子 005 这一件。这不影响合并门,但你该知道。

## 4. acceptance_plan(回填 JSON,逐字)

```json
{
  "plan_version": 1,
  "scenario": "张先生确认「从种子重建出来的 SeatLoom」和「正在跑的 SeatLoom」是同一支舰队:十个席位、四十个项目、十条角色绑定,一个不多一个不少。在此之前,谁按种子重建,得到的是四月的旧舰队。",
  "preconditions": [
    "分支 packet/onyx/topic-opened-019 此刻落后 track/infra-foundation 0 个提交",
    "验绿 ev-verdict-disp-bc20UzOznJQ8hCgbHw876 存在、结论 pass、点名派单 disp-bc20UzOznJQ8hCgbHw876",
    "接受这条分支会同时带入另外两件活的产物:023_gate_criterion.sql(门判据)和 019 迁移(后者已在 track 上由 Aegis 独立落过同一份)。两件各有独立 PASS 验绿,合并不该被拦。"
  ],
  "steps": [
    {
      "action": "在 SeatLoom 里看席位列表",
      "input": "项目 seatloom(默认)"
    },
    {
      "action": "数一下席位:应该有 10 个 —— aegis / flux / lyra / mira / nimbus / onyx / vela / ql / networkorchestrator / stepaicloud",
      "input": "无"
    },
    {
      "action": "看 vela 这个席位:它应该有「语料清洗」相关的能力标签(corpus-ingest / extraction / redaction / data-washing),状态 active",
      "input": "点开 vela"
    },
    {
      "action": "看 ql 这个席位:它挂在 ql_delivery 项目下,角色是 custom(worker),不是 SeatLoom 的正式席位",
      "input": "项目选 ql_delivery"
    },
    {
      "action": "想从命令行确认的话(可选,只读):podman exec -e PGPASSWORD=seatloom seatloom-postgres psql -U seatloom -d seatloom -tAc \"select count(*) from seats;\"",
      "input": "应返回 10"
    }
  ],
  "expected": [
    {
      "output": "席位列表列出 10 个席位,不是 6 个",
      "expect": "四个月前只有 6 个,新接入的四个(vla / ql / networkOrchestrator / stepaicloud)现在也在表里"
    },
    {
      "output": "vela 带着语料清洗的能力标签,状态 active",
      "expect": "标签是脚本当时真实声明的,不是我编的 —— 空标签的席位(如 networkorchestrator)就是真空,没有假装有"
    },
    {
      "output": "ql 在 ql_delivery 下,角色 custom(worker)",
      "expect": "外部项目的接入角色和 SeatLoom 正式席位区分得开"
    },
    {
      "output": "如果席位数和这不符,或某个席位信息不对",
      "expect": "别点接受,把看到的写进打回反馈。这件活的意义就是「种子说得准」,说不准时接受它没有意义。"
    }
  ]
}
```

## 5. `how_to_verify`(一行,给卡面)

打开 SeatLoom 看席位列表,应该是 10 个(aegis / flux / lyra / mira / nimbus / onyx /
vela / ql / networkorchestrator / stepaicloud),不是 6 个;点开 vela 能看到语料清洗的
能力标签。数量或信息不对就别点接受。

## 6. 开卡参数

```
dispatch_id      = disp-bc20UzOznJQ8hCgbHw876          (我的,done)
branch           = packet/onyx/topic-opened-019         (落后 track 0)
base             = track/infra-foundation
flux_verdict_ref = ev-verdict-disp-bc20UzOznJQ8hCgbHw876  (pass,点名本派单)
acceptance_plan  = §4 逐字
how_to_verify    = §5
```

## 7. 边界

- 没合并任何东西进 track ✅
- 没动 live 库 ✅(全程只读;10 席位 / 40 项目,和验绿时一致)
- 没给 021/011 顺序问题在这里开卡 ✅(派单明确禁止,那是独立缺陷)
- 一张卡,只覆盖种子 005 这一件 ✅
