---
memory_id: waker_idle_agent_looks_identical_to_a_working_one
project_id: seatloom
topic_id: seatloom/operating-knowledge
kind: failure
confidence: inferred
status: active
tier: hot
source_kind: file
source_ref: /tmp/seatloom_waker.log
l2_ref_kind: file
l2_file_path: crates/seatloom-mcp-emit/src/bin/waker.rs
l2_line_start: 676
l2_line_end: 700
curation: authored
l0_summary: 唤醒服务的陈旧领单检测会算出正确的年龄,却因为「pane 里有 pi 活着」而拒绝叫醒 —— 一个空闲但活着的 agent 和一个正在干活的 agent 在这个判据下一模一样。2026-10-11 全队因此停摆 17 小时。
---

## 发生了什么

2026-10-11 16:20 接手时,面板显示「6/6 在线、1 个席位在干活、4 件排队」,看起来健康。
账本不是这么说的:`canonical_events` 最后一条写于 10-10 19:18(库内时钟),之后 17 小时空白。

根因在 `/tmp/seatloom_waker.log` 里,连续 17 小时每 90 秒一行,一字不变:

```
seatloom-waker: seat-vela-001 in_flight disp-wash-vela-p2 for 63344s (pane running pi) — an agent is working; not poking
```

陈旧领单检测**本身是对的**,而且吃进了上一条教训 —— 63344 秒这个年龄是 Postgres 用自己的
`NOW()` 算的,没有拿库的时间戳去跟宿主机 `date` 比(那台 podman 虚拟机此刻慢 123 分钟)。
它卡在第二个判据上:「pane 里有 agent 在干活」被实现成「有个叫 pi 的进程活着」。

**一个空闲但活着的 agent,和一个正在干活的 agent,在这个判据下长得一模一样。**

洗数据席位早在 10-10 17:58 就交付了那份活(commit e72a250),Flux 也验绿了
(`ev-flux-verdict-wash-p2`),它只是从来没调 `complete_dispatch`。
于是:领单永久 `in_flight` → 唤醒服务永久判它「在忙,会自排空」→ 它后面 4 张 ready 单
一张都发不出去 → 整条夜间链路断在这里。

## 为什么会改变未来动作

去年那条教训是「查席位存活要查 pane 子进程,不能裸 `ps | grep pi`」。这条是它的下一层:
**查到了子进程也不等于查到了「在推进」**。进程活着是存活信号,不是进度信号。
排查停摆时,「席位在线」和「席位在干活」必须分开问,而且后者不能用前者回答。

兜底的修法是安全的:叫醒一个已经 `in_flight` 的席位不会造成损害
(`get_next_dispatch` 对持有 in_flight 的席位的回答本来就是「你已经在忙了」)。
所以硬阈值兜底的最坏情况是一次多余的叫醒;不兜底的最坏情况,是这 17 小时。
**绝不**让唤醒服务把陈旧的 `in_flight` 改写成 `failed` 或 `ready` —— 那是改账本,不是它的权限。

## 证据

- `/tmp/seatloom_waker.log` 尾部 17 小时的同一行
- `crates/seatloom-mcp-emit/src/bin/waker.rs` 的 `sweep()`:`(Some(id), _)` 分支里
  年龄过阈值之后的 pane 判据
- 账本:`dispatches` 里 `disp-wash-vela-p2` 状态 `in_flight`、`started_at` 为 10-10 17:34;
  同席位 4 行 `ready`
- 已派单 `disp-waker-stale-hard-floor` 给后端席位修(在 `scripts/land-knowledge-live.sh` 第七段)

相关:[[aegis_never_launder_permissions_through_seats]]
