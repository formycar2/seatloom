# 把你这批活同步到当前主干,然后才谈验收

| Field | Value |
|---|---|
| id | AEGIS-2026-10-21-mira-sync-to-track-v1 |
| target_seat | seat-mira-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-21 |
| 状态 | 你的五单都做完且干净,但分支状态挡住了验收 |

## 先说清楚:你做对了,问题不在你

五单全部交付、工作区干净、净化测试 14 个全过、沙箱标识条按派单要求实现
(后端只读环境变量、不新增能力、前端独立组件)。这些我都逐条核过。

**但你的分支落后主干 19 个提交。** 现在开卡合并,等于把你的项目往回拖 19 个提交。

## 为什么会这样,以及为什么必须现在处理

你那条分支是从一个旧主干起出来的。落后的 19 个里,**真正动代码的只有 3 个**:

```
2c69adb  项目过滤:绑定席位发现 + 陈旧响应守卫(boundMirrorNames 在这)
4149b7f  同上的文档与来源说明
cfda740  项目过滤合并进主干
```

其余 16 个是 Aegis 今天的脚本和文档,和界面无关。

**但那个 `2c69adb` 是要命的**:项目过滤的代码在它里面。你的分支上没有它,
所以你(和我)在沙箱里看到的是"所有席位不分项目平铺显示"。

张先生因此以为功能坏了。**实际不是坏了,是版本旧。** 这就是为什么必须先同步。

## 要做的事

1. 把当前主干(`track/infra-foundation`,头 `c078707`)并进你的分支
2. 冲突按"**主干赢**"处理,除了你自己那三件事的文件:
   - `src/lib/sanitizeHtml.ts` 和 `DocumentsWorkspace.tsx`(净化)
   - `SandboxBanner.tsx`、`App.tsx`、`runtime_cmds.rs`(标识条)
   - 状态记忆 / 事件过滤 / 假数据那三处
   **这三个范围里你的改动要保住,别被覆盖掉**
3. 同步后重新验证:
   - `boundMirrorNames` 在 `LiveSeatsView.tsx` 里**必须在**(这是判断有没有同步成功的硬标志)
   - 净化测试仍 14 个全过
   - 类型检查、构建通过
4. 交付说明里写:同步了哪些、冲突怎么解的、`boundMirrorNames` 命中的证据
   (`grep -c boundMirrorNames ui/src/views/LiveSeatsView.tsx`,要给出数字,
   **为 0 就是没同步成功**)

## 验收

- `git rev-list --count packet/mira/knowledge-panel..track/infra-foundation` 输出 **0**
  (落后 0 个才算同步干净)
- `grep -c boundMirrorNames ui/src/views/LiveSeatsView.tsx` 输出 **非 0**
- 净化测试 14 个仍全过
- 构建通过;测试不许改成恒真

## 明确不做

- 不改你自己的实现去适应旧代码 —— 是要主干的新代码进来,不是把你的活改旧
- 不动数据库、不合并、不自己开验收卡
- 不做 rebase 之外的功能改动

## 一句

你之前那批活质量是够的,我现在退回是因为**账本状态不允许在旧基础上开卡**。
同步完这一单,验收就能走。
