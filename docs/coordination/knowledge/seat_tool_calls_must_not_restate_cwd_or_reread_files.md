---
memory_id: seat_tool_calls_must_not_restate_cwd_or_reread_files
project_id: seatloom
topic_id: seatloom/operating-knowledge
kind: failure
confidence: quoted
status: active
tier: hot
source_kind: canonical_event
source_ref: disp-5Wwwsq00WJUfPDAUV5t--
l2_ref_kind: file
l2_file_path: docs/coordination/tasks/aegis/AEGIS-2026-10-22-seat-token-hygiene-v1.md
l2_line_start: 18
l2_line_end: 46
curation: authored
l0_summary: 席位一件事 582 次工具调用,其中 261 次是 `cd <绝对路径> &&` 前缀(信息增量零)、同一文件读 11 次。张先生裁定「Nimbus 做一件事太长了,步骤太多了,也很费 token」。可核判据:同文件 read ≤2、cd 前缀每命令 ≤1、一件事调用 ≤300。省调用不减验证。
---

## 发生了什么

2026-10-22,Aegis 在 Nimbus 的 session 上拆开数(不是印象):

| 项 | 数 |
|---|---|
| 工具调用总数 | **582**(一件事,21 个回合,平均每回合 27.7 次) |
| bash | 370 |
| edit/write | 110 |
| read | 52 |

三类浪费:

1. **`cd /Users/.../nimbus && <真正的事>` 出现 261 次**(另有 71 次是
   `cd .../saih/pi/packages/coding-agent &&`)。约 70 字节,信息增量零。
2. **同一文件反复读**:`dispatch.rs`(2800+ 行)读了 11 次、`repositories.rs` 7 次、
   `compaction.ts` 6 次。每次 read 都把整份文件重新塞进上下文。
3. **bash 命令偏长**:中位 211 字节、最长 2927,总输入 129KB。

我后来在更早一份 session 文件(`~/.pi/agent/sessions/*nimbus*/2026-10-16T04-59-29-536Z_*.jsonl`)
上复测,数字只会更大:bash 1242 次、`cd` 绝对路径前缀 **1209** 次、最常读的文件读 6 次。

张先生的原话(裁定):

> 「Nimbus 做一件事太长了,步骤太多了,也很费 token」

后果不只是 token:那一次因为上下文涨到压缩卡了十几分钟。

## 为什么这不是「自己注意一下」就能改的

一半的成因不在 Nimbus,而在派单方式:Aegis 承认他把一件活拆成
「做 → 重同步 → 开卡」三单,每次都要重建上下文;派单文件写 3-5KB 含大段背景;
每条判据要求重跑验证(这条本身是对的)。Aegis 从这一单起改:一件事一单派完、
派单只写三段。

但另一半确实在工作方式上,而且**它是可以钉死判据的** —— 这就是这一单的意义。

## 三条纪律 + 为什么每条都是判据而不是口号

1. **别重述 `cd`。** 工作目录就是 worktree,bash 工具本身就在那儿起。
   需要切目录时一次 `cd` 之后用相对路径。
   *判据:一条命令里 `cd <绝对路径> &&` 最多出现 1 次。*
2. **别重复读同一文件。** 一次 read 拿全,把要看的部分记在结论里;回头看引用
   自己记下的行号,不重新 read 整份;`grep -n` 拿行号就够的时候别 read 全文。
   *判据:同一文件 read ≤ 2 次。*
3. **批量查询代替逐条。** 同类检查合并成一条命令;已有脚本能一次问完的事
   (分支落后多少、落在哪些目录)别拆成三次 bash。
   *判据:一件事的工具调用数 ≤ 300。*

## 哪一条不许碰

**不为省调用而跳过验证。** 判据重跑是治理,不是浪费。省调用的正确方向是
去掉重复劳动(重述 cwd、重读文件、逐条查询),不是去掉检查。

同理:**不许因为省 token 把测试改弱或改恒真。** 一个永远不会失败的检查比没有
检查更有害 —— 它把「验过了」这件事变成一句无法证伪的话。
