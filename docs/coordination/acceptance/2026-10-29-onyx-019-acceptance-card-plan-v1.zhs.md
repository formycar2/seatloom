# 019 开卡参数与验收方案 —— 给张先生的人话版

| 字段 | 值 |
|---|---|
| template | T5 |
| subtype | acceptance |
| id | ONYX-2026-10-29-019-acceptance-card-plan-v1 |
| status | ready(判据已量,卡待开) |
| author | seat-onyx-001 |
| date | 2026-10-29 |
| version | v1 |
| tags | 019, TopicOpened, acceptance, how-to-verify, plain-language |
| responds to | 派单 `disp-EbFxq6Usb_-41I9hMF6Os`(implementation) |

> 为什么有中文版:派单原话 —— 张先生两次说过看不懂席位黑话,
> 「一个他不会做的验证步骤不算验证步骤」。所以 `how_to_verify` 的人话放这里。
> 本文档 §2 的 JSON 就是回填内容,§3 是为什么这么写。

---

## 0. 先说一件必须说清的事(不然后面的验收会验错东西)

**019 这件活在 track 上已经有一份了,而且不是我这份。**

Aegis 自己那条 `5bd500d`(feat(d5): land 019 TopicOpened, open_topic, and the
knowledge read model on track)把同一个文件放上了 track,内容**逐字节相同**
(`019_topic_opened.sql` 两边都是 `c3b2124`)。我的提交 `deb946e` **不在** track 的
历史里。

所以:迁移本身没问题(live 里 51 条 `TopicOpened` 事件、CHECK 已放宽,都是真的),
**从来没有的是那张验收卡**。这单要补的是卡,不是再交付一次迁移。

**另一件**:我这条分支上还带着另外两件的活 —— `023_gate_criterion.sql`
(派单 `disp-5YCwjJF-oIzwb-d1rmrs5`,验绿 `ev-verdict-onyx-gate-release-check` PASS)
和种子 005(派单 `disp-bc20UzOznJQ8hCgbHw876`,验绿
`ev-verdict-disp-bc20UzOznJQ8hCgbHw876` PASS)。点「接受」会把整条分支并进 track,
这两件会一起进去。它们各自都有独立 PASS 验绿,所以合并门不会拦,
但**你得知道接受的不只是 019**。这一点写进 §2 的 preconditions,不藏在正文里。

## 1. 判据(全部我量的,不是抄派单的)

| 项 | 量到的 | 说明 |
|---|---|---|
| 派单 `disp-019-onyx` | `done`(10-10 18:42 完成) | 迁移活干完了 |
| 验绿 | `ev-019-topic-opened-flux-verdict`,`pass`,点名 `disp-019-onyx` | 工具三道校验全过 |
| 门 | `ev-live-019-applied`(10-11 17:42,SupervisorMessage) | Aegis 已确认 live 生效 |
| 迁移在 track | 在,`c3b2124`,与我的逐字节相同 | Aegis 独立落的 |
| 019 卡 | **0 张**(140 张卡里没有任何一张点名 019) | 挂了 19 天的那一步 |
| live 生效证据 | `TopicOpened` 事件 **51 条**;`topics` 表 **51 行** | 工具真的能用 |
| 我的分支 | 落后 track **0**,track 是它的祖先 | **卡点得下去** |

最后一条是我这次补的关键动作:量的时候发现分支落后 207 个提交、track 不是祖先 ——
那样开出来的卡和 10-28 那批一样点不了。先 merge 了(无冲突,`merge-tree` exit 0),
现在 behind 0。

## 2. acceptance_plan(回填 JSON,逐字)

```json
{
  "plan_version": 1,
  "scenario": "张先生确认「开主题」这个能力在正在运行的 SeatLoom 里是真的能用的:不是代码里有,不是分支上有,而是现在这个库、现在这个界面,能开出一个主题并看见它。这件事从 10-10 干完、10-11 验绿、门也量过,但验收卡一直没开,所以它从来没到过你手上。",
  "preconditions": [
    "SeatLoom 应用是当前构建,连的是正在跑的那个库(live seatloom)",
    "接受这条分支会把三件活一起并进 track:019 迁移(Aegis 已在 track 上独立落过同一份,内容相同)、023_gate_criterion.sql(disp-5YCwjJF-oIzwb-d1rmrs5 的活)、种子 005_adopted_seats_and_external_projects.sql(disp-bc20UzOznJQ8hCgbHw876 的活)。后两件各有独立的 PASS 验绿,合并不该被拦,但它们确实会一起进来。",
    "分支 packet/onyx/topic-opened-019 此刻落后 track/infra-foundation 0 个提交"
  ],
  "steps": [
    {
      "action": "打开 SeatLoom 应用,进「督导面板」(SupervisorIntent),点「读取主题」",
      "input": "项目 seatloom(默认)"
    },
    {
      "action": "看主题列表:应能列出主题,其中 10-28 那天新开的有三个 —— mira-testing-discipline、mira-verification-method、mira-ui-states,都是 open 状态",
      "input": "无"
    },
    {
      "action": "想自己再开一个的话:在面板里开一个新主题,起个名字,确认它出现在列表里",
      "input": "比如名字写 zhang-check-019"
    },
    {
      "action": "想从命令行确认的话(可选,只读,不改任何东西):podman exec -e PGPASSWORD=seatloom seatloom-postgres psql -U seatloom -d seatloom -tAc \"select count(*) from topics;\"",
      "input": "应返回 51 或更多(每开一个主题会加一)"
    }
  ],
  "expected": [
    {
      "output": "主题列表读得出来,不是空的、不是报错",
      "expect": "「读取主题」有反应,列出主题名和状态"
    },
    {
      "output": "10-28 新开的那三个主题在列表里,状态 open",
      "expect": "证明主题不只是能读,而且是这段时间真的在被用"
    },
    {
      "output": "自己开的新主题出现在列表里",
      "expect": "「开主题」这个动作在你手里是能完成的 —— 这才是 019 要的能力"
    },
    {
      "output": "如果上面任一步失败(读不出来/开不了)",
      "expect": "那就是真问题,不要点接受,把看到的报错写进打回反馈里。019 的意义就是这个能力可用,不可用时接受它是没有意义的。"
    }
  ]
}
```

## 3. `how_to_verify`(一行,给卡面)

打开 SeatLoom 的督导面板点「读取主题」,能看到主题列表 —— 其中 10-28 新开的
mira-testing-discipline / mira-verification-method / mira-ui-states 三个是 open 状态;
再自己开一个主题,它出现在列表里。读不出来或开不了就是失败,别点接受。

## 4. 开卡参数(量好的)

```
dispatch_id      = disp-019-onyx                    (我的,done)
branch           = packet/onyx/topic-opened-019      (落后 track 0)
base             = track/infra-foundation             (本地存在)
flux_verdict_ref = ev-019-topic-opened-flux-verdict  (ReviewVerdictIssued, pass,点名本派单)
acceptance_plan  = §2 的 JSON,逐字
how_to_verify    = §3
```

## 5. 为什么这么写(不写成「跑一下验证查询」)

派单说「怎么验」要能照着做。所以每一步都满足:

- **不出现席位黑话**:不写 `ev-`、`disp-`、`acc-`、`packet/`、`behind by`。
- **是"看得到"而不是"读得懂"**:让他打开界面点一下,看到列表里有东西。
- **有他刚经历过的东西当锚**:10-28 那天 Mira 开了三个主题,那是真实发生的事,
  不是一个我编的例子。
- **失败也写明怎么办**:验证步骤必须能失败,否则不叫验证。
- **保留了一条可选的命令行旁路**,但标了「只读」,并且不是必须的。

我没有写「跑 `cargo test`」「跑迁移」这类 —— 那些是席位的活,不是他的活。

## 6. 边界

- 没合并任何东西进 track ✅(接受是张先生点的)
- 没动 live 库 ✅(全程只读;topics 那条 `select count(*)` 是只读)
- 没开第二张卡 ✅(019 一张)
- 没给种子那件(`disp-bc20UzOznJQ8hCgbHw876`)在这里开卡 ✅(派单明确禁止,
  那件等它自己的验绿流程)
