# 补开缺失验收卡 —— 执行记录(`disp-ywDUQlsv7DuL19QFEa5vk`)

派单:`AEGIS-2026-10-27-open-missing-acceptance-cards-v1`。11 件 done 但无 acceptance 的派单,张先生一张都点不到。

**执行顺序:先把有 verdict 的 2 件开卡,再把没 verdict 的 9 件派 Flux 验。**

---

## 一、先核账本,不信转述

派单列了两个 `flux_verdict_ref`。我逐条查了 `canonical_events`,确认它们真实存在、类型是 `ReviewVerdictIssued`、`verdict` 是 `pass`、actor 是 `seat-flux-001`、且点名了对应 dispatch:

| dispatch | verdict ref | 账本实测 |
|---|---|---|
| `disp-6xJ-8eWA7-LtBDeW9G5Yj` | `ev-hfLMM6Gyn3z7V6qIatMlJ` | ✅ `verdict:"pass"`,payload.`dispatch_id` 就是它,refs 里 `dispatch` 是 subject |
| `disp-cLore9rXage5HKVn6Hsbd` | `ev-taJ9qsGBaNLKuGQQ6Ahtm` | ✅ `verdict:"pass"`,同上 |

其余 9 件我逐条查了 `ReviewVerdictIssued` + `ref_type=dispatch`,**全部返回空**,与派单转述一致。

---

## 二、已开的 2 张卡

`request_acceptance` 用的是 `base=track/infra-foundation`、`branch=packet/mira/knowledge-panel`。

| dispatch | acceptance id | verdict ref |
|---|---|---|
| `disp-6xJ-8eWA7-LtBDeW9G5Yj` | `acc-disp-6xJ-8eWA7-LtBDeW9G5Yj` | `ev-hfLMM6Gyn3z7V6qIatMlJ` |
| `disp-cLore9rXage5HKVn6Hsbd` | `acc-disp-cLore9rXage5HKVn6Hsbd` | `ev-taJ9qsGBaNLKuGQQ6Ahtm` |

**落库证据**(`AcceptanceRequested` 事件,`refs` 含 `acceptance` subject + `dispatch` context + `seat` actor):

| dispatch | event id |
|---|---|
| `disp-6xJ-8eWA7-LtBDeW9G5Yj` | `ev-RL3VnFPpBJhgwWM-i3rIT` |
| `disp-cLore9rXage5HKVn6Hsbd` | `ev-IwO2CGEQ01jaFziBPxb4c` |

两张卡的 `acceptance_plan` 都是按交付说明的真实行为写的(scenario / steps / expected / preconditions 四段齐全),`how_to_verify` 写成"做 A 看到 B"的一句话。plan 内容见上面两个事件。

### 第一次开卡被拒,原因值得记

第一次 `request_acceptance` 返回:

> branch "packet/mira/knowledge-panel" does not contain base "track/infra-foundation" — merge track/infra-foundation into packet/mira/knowledge-panel first; accepting an unsynced branch would merge work that never saw the base's later commits

**这条拒得对。** 我的分支落后 track 6 个提交,其中包括 Nimbus 的轮次感知双渲染。直接开卡等于让 operator 合一份从没见过 base 最新提交的分支。

处理:先核对 interval 内容(`git diff --name-only HEAD track/infra-foundation`),确认 track 上新的是 `crates/seatloom-core/turn_stream.rs`、`session_cmds.rs`、`types-dto.ts`、`api.ts`、`useConversationStore.ts`,然后合并,重跑全部门禁,再开卡。

**合并后复跑**:`pnpm test` 336 passed / 29 files、`tsc` 干净、`eslint` 143 problems(133 errors + 10 warnings,与 baseline 一致)、`cargo test` 21 passed、`cargo clippy` 0 warning、`cargo build` 通过。两侧改动都在(我的 `dropStaleSnapshotViewport`、`cmd_open_external_url`、`openExternalUrl` 都还在;Nimbus 的新文件也在)。

---

## 三、已派 Flux 验的 9 件

每件都用了 `kind:"verification"` + `verdict_event_id`(工具 mint),这样 Flux 必须用那个 id 发 verdict,流程闭环。全部 `status=ready`。

| 待验 dispatch | 派给 Flux 的单 | 预声明的 verdict event id |
|---|---|---|
| `disp-OwOSLvL4EbqoB6YhlBQHi` (P0,scrollback) | `disp--Hvv9ROFzarJ7OrPUCgsT` | `ev-nbpmWap9wlZue7V_A70D1` |
| `disp-H66BOKrYda4xltEgIwvuG` (链接点不开) | `disp-CE-64L-sSk44LGwSd8L6y` | `ev-3qVHyZW597RSpMiK2zxby` |
| `disp-wa7F61uczSw8ebZzeXwml` (滚轮 + IME) | `disp-99Sr4c7oTzs1LRBW4ldp7` | `ev-WirH7Qh1eNDP7dZSV5uVs` |
| `disp-gvVtOC1Z_uvVz8mq-lc45` (项目切换器) | `disp-UODw2mn6k1XlpJC49FP2d` | `ev-sMrl5_wXRSDsj_V4qe3gs` |
| `disp-zsKFSIHHgv5v7wLG-94UR` (用词改正) | `disp-kaaiuujze3aPbYCyRReWX` | `ev-1mD-lbquk7qa_0venCWXO` |
| `disp-J9ExyK_2-3ajGuXjBvlKj` (失败状态累积) | `disp-nQ-SnADq1jXvYN16ucSmv` | `ev-Q1LjVPXRhHH4_xTQEjlim` |
| `disp-B4Y5oAhMQwtXm5Vva1fw` (覆盖率补齐) | `disp-w_-TIYqGYCf7yHrmSFqHP` | `ev-usP7dX7geulRhaDy1eXwD` |
| `disp-yhtJXKP7_krz7NBwz6v12` (display:none → fit 失效) | `disp-OuA3VVmDSxa4whEXZUseM` | `ev-SZNBEZHSGuj1U2fV7TzNd` |
| `disp-Ksm1oC3fHNRnH2zumBN_p` (isTypingContext 守卫) | `disp-KxAVjC1m5rIhLmMFdQ18v` | `ev-Oc4hMs_bBXlnb52Pr1JJK` |

每张验证单的 payload 都附了交付说明路径、dispatch_to_verify、她自己的声明(含未闭合项)、以及一条明确规则:**不许把 `flux_verdict_ref` 指向别的 dispatch 的 verdict,不许为了开卡改 ref,验出 fail 就照实发 fail。**

P0 那件(`disp-OwOSLvL4EbqoB6YhlBQHi`)priority 设 100,其余 90,并提醒 Flux 它和 Nimbus 的镜像单共享同一块 UI。

---

## 四、没有静默跳过任何一件

11 件全部有下文:2 件已开卡(`pending`),9 件已派验(`ready`,等 verdict 落库)。

---

## 五、给张先生的

- **现在有 2 张卡可以点了**:`disp-6xJ-8eWA7-LtBDeW9G5Yj`(启动查询被吞 → 红色提示 + 重试)和 `disp-cLore9rXage5HKVn6Hsbd`(订阅断线要说出来)。
- **剩下 9 件要等 Flux 验完才能开卡**。这不是拖延:没有 verdict 就开卡,工具会拒(`VerdictNotAboutThisDispatch` 那类),而且拿别件的 verdict 凑是规则明令禁止的。
- **验完之前,这些活的代码都已经在分支上了**,只是张先生看不到卡。

## 六、顺带发现的、需要上报的

合并 track 时看到 `docs/coordination/knowledge/` 下新增了三条 Aegis 的知识记录,其中一条 `reset_clears_scrollback_to_fix_one_bug_and_causes_another.md` 正是这次 P0 的根因。我已读过,与我的实测一致,不需要重复记。
