# Vela — disp-acceptance-cards-plain-language-vela 交付说明(验收卡人话重写)

- **派单**:`disp-acceptance-cards-plain-language-vela`(issuer: Lyra/Aegis 线)
- **席位**:`seat-vela-001`(Vela,语料清洗)
- **分支 / 提交**:`packet/vela/washing-pipeline` @ 本提交(含一次 base 同步合并)
- **结果**:**人话重写做完了(两张卡的计划都已通过工具的形状校验),但「重开」被工具面缺陷堵死,
  本单以 `failed` 结单并已上报 Aegis。** 细节见 §4–§6。

---

## 1. 派单要什么

把 `acc-disp-wash-vela-p1` 和 `acc-disp-wash-vela-p2` 两张验收卡**各作废重开一次**,
「怎么验」改成人话。人话判据(缺一条不算合格):

1. 全中文,不出现 S0/S1/S2/S2.5/L2/typed record/confidence/quoted/inferred 等术语
2. 写清:看什么 → 看到什么算对 → 什么算打回
3. 从没读过代码的人能照着走完
4. 不用自造缩写,不引用派单标题

## 2. 我做了什么

1. **两张旧卡已作废**(`cancel_acceptance`,pending → cancelled 终态),理由如实记录:
   「张先生 2026-10-14 原话:这些都没办法验收,我看不懂 —— 「怎么验」重写成不读代码、
   不熟悉流水线术语的人也能照着走完的话;旧卡作废重开,技术判据不变。」
2. **两张人话计划已写完**,并通过了 `request_acceptance` 的形状校验
   (plan_version/steps/expected/preconditions 全部合法 —— 报错发生在之后的数据库插入,
   不在计划内容上;换句话说,内容这一关是过的)。
3. **一次必须先做的分支同步**:重开时验收门以
   `branch "packet/vela/washing-pipeline" does not contain base "track/infra-foundation"`
   拒绝。我把 base 合并进本分支(合并提交 `94c6318`,61 个文件 / +10,431 行,
   唯一冲突 `.gitignore` 已解法:两侧新增都保留)。合并后门禁通过,暴露出 §4 的真问题。

## 3. 一处必须披露的对参考写法的偏差

派单参考写法(p1 第 2 步)说:「找一条带密钥样式的文本,确认它显示的是
`⟪REDACTED:类型:指纹⟫` 而不是原文」。**我实测库里没有这个现象**:对真正入库的文本
(`records.writein.jsonl`,87 条) grep,脱敏标记 **0 条**、密钥样式残串 **0 条**、
内网域名 **0 条** —— 提炼时把含密那段整个没写进表,标记也就无从显示。

所以我把这一步改成**可观测的事实**:「想一条你确定出现在自己历史会话里的密钥/令牌/内网
地址,在面板里翻覆盖那段对话的主题,这条原文应该一条都找不到;看到任何一条密钥原文就是
打回。」这比原参考更强(密钥根本没进表,而不是进了表被打码),也没把张先生引向一个
界面上不存在的现象。

## 4. 堵死重开的东西(verbatim 实证)

```
P1: dispatch rejected: db error | ERROR: duplicate key value violates unique constraint
    "acceptances_pkey" DETAIL: Key (id)=(acc-disp-wash-vela-p1) already exists.
    | sqlstate=23505 constraint=Some("acceptances_pkey") table=Some("acceptances")
P2: dispatch rejected: db error | ERROR: duplicate key value violates unique constraint
    "acceptances_pkey" DETAIL: Key (id)=(acc-disp-wash-vela-p2) already exists.
    | sqlstate=23505 constraint=Some("acceptances_pkey") table=Some("acceptances")
```

**根因(工具面内部不一致)**:`request_acceptance` 没有 acceptance id 入参,id 由
dispatch_id 确定性推导(`acc-<dispatch-id>`);而 `cancel_acceptance` 把行置为终态
**不删行**(账本纪律)。于是:

- `request_acceptance` 的守卫「dispatch 不得有 pending 的 acceptance」**预设了
  作废后可以重开**(cancelled 不是 pending,守卫确实放行了);
- 但确定性主键让重开必然撞已存在的终态行。

即:**作废重开这条路,守卫放行、主键堵死**。派单(和 Aegis 文档里「正解是派单让提请席位
作废重开」的判断)都建立在「能重开」这个未经实测的假设上。这不是我绕得过去的——
没有 raw SQL、没有旁路文件、没有直接改 live 库(那是 Aegis 的事),我只有 `emit_event`
一个上报通道。

**附带影响**:同一张派单调了 mira(`disp-acceptance-cards-plain-language-v2`,
卡 `acc-disp-supervisor-intent-ui-wiring`)。mira 若走同样的「作废→重开」,会撞同一个
PK。已在上报里点名,建议 Aegis 先定机制再让 mira 动作,或提醒 mira 先别作废。

## 5. 上报(已发)

`emit_event(SupervisorMessage)` → Aegis,带上两段 verbatim 报错、根因分析、mira 的连带
风险,以及三个可选修法(任选,都由 Aegis/Nimbus 侧动手,不经过我):

1. 给 `request_acceptance` 加 id 入参(或让 id 在后端按序追加后缀),使作废后重开可行;
2. Aegis/Lyra 发一张**新的 rework 派单**(新 dispatch_id → 新卡 id),我把这两份人话计划
   原样挂到新卡上 —— 这条路现在就能走,不需要改任何代码;
3. (最后手段)数据库层删掉两条 cancelled 行 —— 这是改 live 库,只有 Aegis 能裁。

## 6. 两份人话计划(原样保留,机制一修好即可一键重开)

### acc-disp-wash-vela-p1(清洗前半段)

**怎么验(一句话)**:打开 SeatLoom 知识面板:主题树里每个主题是独立的工作主题,翻不到
任何一条密钥原文,点开任一条记忆的行号能打开你的会话原文 —— 三样都成立即通过;主题切错
或看到密钥原文,在卡上写打回理由。

```json
{
  "plan_version": 1,
  "scenario": "张先生确认清洗流水线的前半段产出可信:你的 16 个历史会话被完整读进来(没有哪个文件被跳过)、敏感信息在进库之前就被处理掉、主题树里的每个主题都是一个独立的工作主题、内容被大幅精简但你随时能顺着记忆给的行号回到当时的原文 —— 值得继续花算力往下跑抽取。",
  "preconditions": [
    "SeatLoom 应用已经打开,知识面板能正常显示(面板打不起先找 Aegis,那不是这张卡要验的事)",
    "你手上有自己被清洗的那批历史会话的原始文件(用来对行号、确认密钥存在过)",
    "前半段的交付文档在手:docs/coordination/tasks/vela/VELA-2026-10-10-washing-pipeline-p1-delivery-v1.md"
  ],
  "steps": [
    {"action": "打开 SeatLoom 的知识面板,看主题树。这些主题是从你的 16 个历史会话里分段出来的。逐个问自己:这是一个独立的工作主题吗?", "input": "知识面板的主题树"},
    {"action": "想一条你确定出现在自己历史会话里的密钥、令牌或内网地址(比如你公司那个内部域名)。在面板里点开和那段对话有关的主题,翻它们的记忆:这条原文应该一条都找不到。", "input": "你自己的原始会话文件 + 知识面板"},
    {"action": "点任意主题进去,看它的记忆条数,再和这个主题对应的原始会话比:记忆应该少很多。然后点开任意一条记忆给出的文件路径和行号,打开你当时的会话原文,确认还打得开。", "input": "任选一个主题"},
    {"action": "回到主题树:如果有哪个主题明显切错了 —— 两个不相关的活被并成一个主题,或者一个活被拆得七零八落 —— 在这张卡上写打回理由。", "input": "无"}
  ],
  "expected": [
    {"output": "主题树(从你 16 个历史会话切出来的全部主题)", "expect": "每个主题是一个独立的工作主题,没有把两个不相关的活并成一个。看到并错的,在这张卡上写打回理由 —— 主题切错,以后所有的查找都建在沙上。"},
    {"output": "你翻到的所有记忆正文", "expect": "没有任何一条密钥、令牌、内网地址的原文。脱敏发生在进库之前,你看到的是当时工作的提炼。只要看到任何一条密钥原文,就是打回,这是红线。"},
    {"output": "任选主题的记忆条数 + 任一条记忆的文件路径和行号", "expect": "记忆条数比原始会话少很多(原始内容被砍掉了八成到九成);顺着路径和行号能打开你当时的会话原文。行号打不开原文,就是打回。"},
    {"output": "整棵主题树对照你的记忆", "expect": "你记得的实打实的工作,树里都找得到一个主题收着。某段干过的活完全找不到影子,写打回理由(只是登录过、没有实质内容的会话不算)。"}
  ]
}
```

其余入参:`base=track/infra-foundation`(已同步,本地分支存在)、
`branch=packet/vela/washing-pipeline`、`dispatch_id=disp-wash-vela-p1`、
`flux_verdict_ref=ev-SWfX7pNZH8PObLcGxg3pY`(账本在,pass)。

### acc-disp-wash-vela-p2(清洗成果)

**怎么验(一句话)**:在知识面板随便点 3 张记忆卡:摘要是提炼不是抄原文,每张卡的文件路径
和行号对得回你的会话原文,标「原话」的原文真这么说过 —— 三样都成立即通过,任何一条对不上
在卡上写打回理由。

```json
{
  "plan_version": 1,
  "scenario": "张先生确认你的历史真的变成了管得住的知识:面板里的每条记忆都是提炼过的结论,不是把原文抄了一遍;每条记忆给出的文件路径和行号,能对回你当时的会话原文;标着「原话」的,原文真的这么说过。知识从一堆聊天记录变成了能查、能信、能回溯的资产。",
  "preconditions": [
    "SeatLoom 应用已经打开,知识面板能正常显示,而且里面已经有记忆(87 条,从你 16 个历史会话里提炼)",
    "你手上有自己的历史会话原始文件(用来对路径和行号)",
    "入库的交付说明在手:docs/coordination/tasks/vela/disp-onight-writein-v3-delivery.md(里面写了入库条数和丢弃原因)"
  ],
  "steps": [
    {"action": "在知识面板里随便点开 3 张记忆卡。每张先看那一句话摘要,再点开看详述,判断这是提炼过的结论,还是把原文抄了一遍。抄原文算打回。", "input": "任选 3 张记忆卡,尽量挑不同主题的"},
    {"action": "每张卡再往下点一层,会给出一个文件路径和行号。拿那个路径和行号,打开你自己的历史会话文件,对一眼:卡上说的这件事,是不是真发生在那些行里。对不上就是打回 —— 这是最重要的一步。", "input": "上一步那 3 张卡的路径和行号"},
    {"action": "卡上会标这句话是「原话」还是「推断」。抽查标「原话」的,回到原文确认它真的这么说过;发现标「原话」但其实是你推出来的,算打回。", "input": "同上 3 张卡"}
  ],
  "expected": [
    {"output": "3 张记忆卡的一句话摘要和详述", "expect": "每张都是提炼过的结论:读完你知道这件事的要点,而不是又读了一遍原文。任何一张是原文搬运,算打回。"},
    {"output": "3 张卡给出的文件路径和行号,对照你的会话原文", "expect": "卡上讲的每件事都在那些行里真发生过,对象和结论对得上。任何一条对不上,算打回 —— 这一步是承重墙,对不上说明这条记忆可能是编的。"},
    {"output": "卡上「原话」「推断」的标注,对照原文", "expect": "标「原话」的,原文真的那么说;标「推断」的,看得出有推断的成分。标注不诚实,算打回。"}
  ]
}
```

其余入参:`base=track/infra-foundation`、`branch=packet/vela/washing-pipeline`、
`dispatch_id=disp-wash-vela-p2`、`flux_verdict_ref=ev-flux-verdict-wash-p2`(账本在,pass)。

## 7. 为什么结单 outcome=failed 而不是 done

派单的成功条件是「两张卡各作废重开一次」。作废做到了;人话写好了;但**重开不存在** ——
张先生现在能看到的是两张 cancelled 的卡,没有可验的新卡。标 done 就是又一次「没有证据的
门式声明」,比我昨天拒绝的那种更接近造假(这次连事件都没有)。失败理由已逐字带进
`SeatDispatchFailed`,机制修好后按 §5 第 2 条(新 rework 派单)或第 1 条(工具加 id 入参)
即可一键重开,两份计划在 §6 原样等着。

## 8. 改动文件

- `docs/coordination/tasks/vela/disp-acceptance-cards-plain-language-vela-delivery.md`(本文件)
- `.gitignore`(合并冲突解法:Python 字节码与 resume-contexts 两侧新增都保留)
- 合并提交 `94c6318`(track/infra-foundation → packet/vela/washing-pipeline,验收门前禁要求)
