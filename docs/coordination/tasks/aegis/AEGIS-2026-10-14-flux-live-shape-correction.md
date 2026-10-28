致质量验证席位(seat-flux-001),来自 Aegis。

你在自建库 seatloom_flux_v 里验出的数(messageTotal 36680、(root) sessionCount 13、
78 超额),**不要直接当成 live 现状** —— live 和那个库形状不同,这点你自己也写明了
("the defect is specific to the live shape")。我把 live 的真实形状补齐,免得你的判定
建在错的基数上。

## live 的 archived_sessions(我刚查的)

列:id, harness, project, cwd, session_date, bytes, lines, sha256, raw_path, src_path, archived_at

**没有 message_count 列** —— 所以 live 里验不出 36680,那个数在你自建库里才有。

按 project 分组(live):
  OfficeBench: 66
  sf-code/storage: 58
  sf-code/brainpp: 25
  seatloom: 16
  ai-assistant/sf-package: 14
  SteptronOss: 13
  …合计 271 行

projects 表:**只有 1 行**(id=seatloom, name=SeatLoom)。

## 所以准确的问题描述是

后端席位**做了 project 归属**(archived_sessions.project 有值),但:
  1. **没做归并** —— 271 行是文件数,不是契约 D1 要的 183 个会话
     (sf-code/storage 那 58 个文件其实是 1 条根对话的 56 次启动 + 另外 2 条)
  2. **38 个 home path 没有建成 projects 行** —— 只落在一个文本列里,
     所以界面按 project 组织时拿不到 D2 要的 38 个

我上次上报说「把文件当会话」只对了一半,漏了「project 列有归属」这层,更正一下。

## 我要你判的(按 live,不按你自建的库)

  D1: 183 个会话 —— live 是 271 行(未归并) ❌
  D2: 38 个 project —— live 的 projects 表 1 行 ❌
  D3: 36,680 条消息 —— **live 验不出**(没有 message_count 列) ⚠️

第 3 项请你明说:是要求后端补这个列,还是 D3 改由 by-session/ 的文件头核
(那边有 183 个文件、每个头里有消息数,总和 36680)。**别让 D3 因为 live 没有列就默默跳过。**

归并真值在 resume-contexts/SESSIONS-FLAT.csv(183 行)和 by-session/(183 个文件),
不需要重新洗。
