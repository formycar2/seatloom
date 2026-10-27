---
id: v5_honesty_ranks_first_because_it_makes_other_values_trustworthy
source_kind: canonical_event
source_ref: disp-RWEf5ATPvICLkGtF__fEY
session_scope: seatloom
title: 界面的诚实性排第一,因为它决定另四条价值可不可信
domain: frontend
modality: preference
status: active
curation: authored
confidence: quoted
owner: aegis
created_by: aegis
created_at: 2026-10-20
supersedes: null
evidence_refs:
  - docs/coordination/tasks/lyra/LYRA-2026-10-19-top-level-value-design-v1.md
---

张先生 2026-10-20 的话:「很多模块都是坏的」「我其实没看懂了,跟不上你说的内容」
「有一些怀疑内容的准确性」。三条都指向同一件事:**他无法分辨哪一页是真的。**

这不是功能缺失,是界面显示了不存在的数据。三处已逐行确认(基线 track@cfda740):

- `ui/src/components/SupervisionDashboard.tsx:62` — `const budgetHealthPercent = 72;` 写死常量,
  却作为「预算健康度」呈现。
- 同文件 `:104` — `new Date().toLocaleTimeString()` 把**渲染时刻**当「最后同步时间」显示。
- `ui/src/app-v2/dashboard/GlobalDashboard.tsx:11,44` + `mock-data.ts:292` —
  `MOCK_GLOBAL_SUMMARY` 纯假数据(含 p-1/p-2/p-3),由 `SupervisorPanel.tsx:521` 渲染,完整可达。

**判据**:界面显示的每个数字必须能回答「数的是谁、在什么范围、截止什么时候、从哪来」。
答不出来就不该显示。宁可写「暂无/未知」,不能用占位数字冒充 —— 一个永远不会报错的检查,
比没有检查更有害。

**为什么排第一**:价值设计里另四条(知道全队在干什么、点开就懂为什么、只有我能定的自动来找我、
我说过和做到的对得上)都以「产品显示的东西可信」为前提。这一步不成立,后面全是空中楼阁。

注意路径:`SupervisionDashboard.tsx` 在 `ui/src/components/`,不在 `ui/src/app-v2/panel/` ——
2026-10-19 的记录写的是旧路径,直接 `sed` 会拿到空还以为是干净的。
