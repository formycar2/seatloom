# Delivery — disp-acceptance-cards-plain-language-v2(把意图入口的验收卡改成人话)

**执行 actor:** `seat:seat-mira-001` · **Date:** 2026-10-14
**触发:** 张先生原话「这些都没办法验收,我看不懂……描述无法让我完成测试,不清不楚」。Lyra 认领为其派单未把「操作者能照着测」写成可验条件。

## 做了什么

1. **作废技术卡**:`cancel_acceptance(acc-disp-supervisor-intent-ui-wiring)`(我是提请席位,守卫过)。原卡方案满是技术术语,运营者看不懂。
2. **撞到设计约束并绕过**:同派单重开 `request_acceptance` 报主键冲突 —— 验收卡 id 由派单 id 派生(`acc-<派单id>`),作废行(终态)仍占该 id。改用**新派单作载体**:`enqueue_dispatch(disp-supervisor-intent-ui-wiring-plain)`;又受「一席位同时仅一条 in_flight」限制,先完成当前任务腾槽、下一轮认领新派单 → complete → 开卡。
3. **人话卡开成**:`acc-disp-supervisor-intent-ui-wiring-plain`(pending,回读确认)。方案通篇中文、无英文术语(唯一"英文"是结构字段名 output/expect,属卡表结构非文案);写清**打什么→看到什么→什么算过→什么算打回**;含「故意打一句含糊的,它必须停下来问你不猜(猜了算打回)」;一个没读过代码的人能照着走;无自造缩写、无派单标题。

## 无文件改动

本次交付物是账本里的验收卡(ledger 对象),不涉及代码/文件变更,故无 git 提交。(分支仍含 track,前面接线那一轮已同步。)

## 需上报(给 Lyra / Aegis)

「作废 → 同派单重开验收卡」这条路被**确定性 id + 终态行**堵死。Lyra 对多张卡(Vela 的 p1/p2 等)都用这个「作废+重开」模式,会撞同一个冲突 —— 要么每张重开都用新派单作载体,要么修机制(作废即释放 id / 允许同派单重开)。已 `remember`(mem-FSHFAkB0dsSSJCMJ6XO9l)。

## 给张先生的三段

- **变了什么:** 那张你看不懂的意图入口验收卡,我作废了,换成了一张**大白话**的新卡 —— 每一步都是「打什么→看到什么→什么算过→什么算打回」,没读过代码的人也能照着测。
- **你会看到:** 待验收区那张人话卡(acc-disp-supervisor-intent-ui-wiring-plain)。照着走:打一句清楚的话看提议卡 → 故意打句含糊的看它是否停下来问你(不猜)→ 点确认去派单列表翻到新单、操作者是你。哪步不符合卡上写的「算打回」,你就打回。
- **需要你做什么:** 照这张人话卡测;过了点接受(合并我来做,你从不合并)。另:另两张洗数据的卡归洗数据席位,Lyra 另派,不归我。

## 收尾

`complete_dispatch` 两个都 done(plain-language 任务 + 重开载体派单);`request_acceptance` → acc-disp-supervisor-intent-ui-wiring-plain(pending)。前向记录见 remember。
