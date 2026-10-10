| template | T5 |
| subtype | acceptance_plan |
| doc_id | 2026-10-11-lyra-mira-knowledge-panel-headline-acceptance-plan-v1 |
| title | Acceptance plan — 头条:界面里看到从历史会话提炼的知识(主题树 + 记忆卡 + 三层披露 + 行级证据) |
| status | active |
| author | lyra |
| doc_date | 2026-10-11 |
| version | v1 |
| depends_on | AEGIS-2026-10-11-overnight-plan.md, LYRA-2026-10-11-overnight-headline.md, 前三个 lane 的方案 |
| tags | acceptance-plan, headline, knowledge-panel, mira, overnight |

# Acceptance plan — 头条卡:界面里的知识面板

**对象**:seat-mira-001 的 knowledge-panel lane(dispatch `disp-onight-panel`)· 分支
`packet/mira/knowledge-panel`。**这张卡就是张先生醒来点的那一张** —— 它验收的不是一块 UI,
而是整条链:历史会话 → 提炼 → 入库 → 读出 → 可见。shape v1,§2 JSON 逐字入参。

## 1. 这个交付是什么(他醒来要看到的那一件事)

主界面里多一块他从来没见过的面板:一棵 项目 → 主题 的树;点主题,一组记忆卡;每张卡
默认一句话摘要,展开给详述,再展开给**指回原文的行号**。知识层第一次变成他能看见、
能点的东西。同时:坏卡 `acc-disp-me6Nm8e9D6ZRe4fKyVokY` 已作废(它点不了)。

## 2. acceptance_plan(逐字 JSON)

```json
{
  "plan_version": 1,
  "scenario": "张先生在 SeatLoom 主界面里,亲眼看到从他自己的 16 个历史会话提炼出来的知识:一棵主题树、点开是记忆卡、默认一句话、展开有详述、再展开有指回原文的行号 —— 「数据+AI 驱动的基座」第一次是他能看见能点的东西,并且他能亲手点一次「验收」把这块新功能合进来。",
  "preconditions": [
    "全链就绪:门事件 ev-typed-memory-tools-landed(真实写入成功)+ read-model 已落地(其判定 ev-verdict-knowledge-readmodel)+ Vela 写库完成(真实提炼记忆在库)",
    "SeatLoom 构建含知识面板与知识命令(集成构建:panel + read-model + 已入库数据)",
    "旧坏卡 acc-disp-me6Nm8e9D6ZRe4fKyVokY 已作废(cancel_acceptance,理由已记录)"
  ],
  "steps": [
    { "action": "打开 SeatLoom 主界面,找到知识面板(项目仪表盘内,继承既有骨架的新 section)", "input": "项目 seatloom" },
    { "action": "看主题树", "input": "无(看:项目下的主题层级、每个主题的记忆数)" },
    { "action": "点一个主题", "input": "任选一个(优先选记忆数多的)" },
    { "action": "看记忆卡列表(默认态)", "input": "无(看:每张卡只有标题 + 类型徽标(kind)+ 置信(quoted|inferred)+ 一句话摘要)" },
    { "action": "展开一张卡的第一层", "input": "点卡的展开" },
    { "action": "展开同一张卡的第二层(证据层)", "input": "再点" },
    { "action": "按行号回原文:拿 L2 的文件路径 + 行号,打开那个历史会话文件对一眼", "input": "卡上给的文件路径与行号区间" },
    { "action": "回到待验收区,对这张卡点「接受」", "input": "点击「接受」" }
  ],
  "expected": [
    { "output": "知识面板在主界面可见,不长得像别处", "expect": "继承既有主界面骨架(未重排版);零新增界面检查问题(实测结果随卡附上)" },
    { "output": "主题树呈现 project → topic 层级", "expect": "主题来自他真实会话的分段(含洗数据席位切出的工作主题);记忆数非零的主题可点" },
    { "output": "记忆卡默认态收敛", "expect": "默认只有一句话摘要 —— 渐进披露第一层;不一次性倾倒" },
    { "output": "第一层展开给详述", "expect": "L1 概览出现;仍不含原文" },
    { "output": "第二层展开给证据", "expect": "L2:文件路径 + 行号区间(archived_session 来源);「证据在某处」式的卡不存在" },
    { "output": "行号回溯成功", "expect": "按路径+行号打开原文,内容确实是这张卡的来源(抽 1-2 张即可)" },
    { "output": "点「接受」后卡面显示合并 commit,分支 --no-ff 合入 track/infra-foundation", "expect": "他终于能点一次验收;冲突会拒绝并展示 git 原文(不 force)—— 预期不冲突" },
    { "output": "空状态有意义(若某项目无记忆)", "expect": "说清「为什么空、下一步是什么」,不是空框" }
  ]
}
```

## 3. 建卡参数(request_acceptance,执行席位 seat-mira-001)

```
dispatch_id=disp-onight-panel
branch=packet/mira/knowledge-panel
base=track/infra-foundation
how_to_verify=主界面知识面板可见项目→主题树;点主题见记忆卡,默认一句话、展开L1、再展开L2行号;按行号回原文成功;坏卡已作废;点接受见合并 commit
flux_verdict_ref=ev-verdict-knowledge-panel
acceptance_plan=<§2 JSON 逐字>
```

## 4. 血缘(卡上要能看到的前因后果)

open_topic(Nimbus)→ 冒烟门事件(真实写入成功)→ Vela 写库(88 条真实提炼)→ read-model
(Nimbus,ev-verdict-knowledge-readmodel)→ 本面板(Mira)。每段的 dispatch/分支/判定都在
账本里,`get_acceptance_lineage` 可拉全链 —— 血缘不再是卡上缺失的东西。

## 5. 坏卡作废(本 lane 第一动作)

Mira 对本自己的 pending 卡 `acc-disp-me6Nm8e9D6ZRe4fKyVokY` 调 cancel_acceptance,
理由必填(建议:「base 记录为不存在的 'track',内容已由 Aegis 自举合并,按裁决作废,
新功能以带血缘+结构化方案的新卡重新提请」)。作废是终态,不可改。
