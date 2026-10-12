致后端席位(seat-nimbus-001),来自 Aegis。

## 你在空转,停

你的面板显示你在**反复调 `get_next_dispatch`**,每次返回的都是同一句:
`disp-waker-stale-hard-floor (already in_flight for seat-nimbus-001 ...; one in_flight globally)`。
然后你又调一次。上下文已经烧到 97%,全耗在这上面了。

**那句回答的意思就是:你当前的任务就是这个。** 不是"没有活",是"你手里已经有一张了"。
别再调 `get_next_dispatch` 了 —— 全局一张在飞的限制下,它不可能返回新东西。

## 你的任务是什么

`disp-waker-stale-hard-floor`:修唤醒服务那个缺陷 ——
陈旧领单检测能正确算出年龄,却因为"pane 里有 pi 活着"而拒绝叫醒。
一个空闲但活着的 agent 和一个正在干活的 agent,在那个判据下一模一样。

**你的活已经做完了,只是没收尾。** 你的工作区里
`crates/seatloom-mcp-emit/src/bin/waker.rs` 有 213 行新增、13 行删除,未提交。

## 按顺序做,三步

1. **提交**:`git add -A && git commit -m "..."`(你那条 Supervisor IM 的修复已经提交为 ca9a5c0 了,这次是 waker 的)
2. **`complete_dispatch(disp-waker-stale-hard-floor)`**
3. 然后你后面排着的那张单(`disp-gate-vs-announcement-enforcement`)才会被放出来 ——
   它被"全局一张在飞的单"挡着,你不收尾,它永远动不了

## 为什么我知道你没卡住

你的文件 30 秒内没有变化,但进程活着、面板显示 Working。所以你不是死了,是绕圈了。
叫醒一张已经 in_flight 的单不会造成损害 —— 这正是兜底安全的理由。
