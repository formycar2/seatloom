# NIMBUS-2026-10-27-open-missing-acceptance-cards-v1 — 交付说明

派单 `disp-Tg64EtzktcysA_ZAVl_tG`(目标 `seat-nimbus-001`)。
任务包 `docs/coordination/tasks/aegis/AEGIS-2026-10-27-open-missing-acceptance-cards-v1.md`。

## 结论:一张卡都没开,因为一件 verdict 是 fail、两件没有 verdict

按任务包第 3 节的三条规则逐件核过库,没有一件满足开卡条件。
**没有编 ref,没有拿别的单的 verdict 凑,没有静默跳过。**

| dispatch | 内容 | verdict 状态 | 动作 |
|---|---|---|---|
| `disp-XnCxvGI88oAbdGcyeefhl` | 022 会话工作桥视图 | `ev-FLUX-AUDIT-GATE-C-SESSION-WORK-BRIDGE` = **fail** | 不开卡;阻塞原因实测已解除,派 Flux 重验 |
| `disp-98QeI7oCe_posblRFa_nf` | 双渲染席位流 | **无** | 不开卡;派 Flux 验 |
| `disp-lu24NPShI-80fqWuvNZeq` | 补开验收卡 | **无** | 不开卡;派 Flux 验 |

## 为什么 fail 那一件仍然不开卡(即使阻塞已经解除)

Flux 的 fail verdict 给了两条具体原因。我逐条实测,两条现在都不成立:

**原因一:「022 不在 track/infra-foundation 上,apply-schema-to-live.sh 走不到」**

```
$ git branch --contains f7bfe23
  packet/mira/knowledge-panel
* packet/nimbus/project-scoped-reads
  track/infra-foundation        <-- 现在含它
```

`f7bfe23`(022 落地那个提交)已由 `3e290177643e9fca1f525af56bfc0a7d515fd628`
合入 track。

**原因二:「Registering 021 是唯一解阻塞的改动」**

track 上 `scripts/apply-db.sh` 的 `POST_SEED_SCHEMA` 里,
`021_history_sessions_corpus.sql` 有一条**生效**注册行(不是注释)。

**以及 Flux 自己写的判据原话:**

> a real call against live would fail with ViewMissing, so emitting the gate
> would assert something false

现在实测那个调用是真的能成功:

```
select count(*) from v_memory_session_join;      --> 90
information_schema.views where table_name='v_memory_session_join'  --> 1 行
```

而且 Flux 当时独立测的 `both 87 / a1_path 3 / unresolved 0`,我在 live 上
复现**完全一致**。

**但 fail 就是 fail。** 我不改 verdict、不换一个 ref 去凑、不把"阻塞已解除"
当成"verdict 已翻"。verdict 是关于那个时间点的事实,翻它只能由 Flux 重出。
所以派了 Flux 重验(见下),等新的 pass 落库再开卡。

## 做了什么:派 Flux 验三件

`enqueue_dispatch` → `seat-flux-001`,得到
**`disp-06a95xPk7h6E2pj6FDo4O`**(status=ready)。

payload 里对三件各附了:交付说明路径、commit、以及给 Flux 的具体提示
(022 那件把 fail 的两条原因和现在的实测值都写进去了,省得它重新推一遍)。

**并且明确要求三件各出一条独立 verdict、`dispatch_id` 逐条对上。**
理由写进了 payload:合成一条会让另外两件永远开不了卡 —— 一张验收卡只能指向
一条 verdict,合成之后那两件的 `flux_verdict_ref` 无从指起。
这不是我谨慎,是 `request_acceptance` 的守卫会拦(`VerdictNotAboutThisDispatch`)。

## 判据(任务包第 3 节第 1 条)

> 判据:`select dispatch_id,status from acceptances` 能看到对应行 `pending`。

实测 —— **三件都没有行,这是符合规则的现状,不是漏工**:

```
$ select dispatch_id, status from acceptances
  acc-disp-5WQ7UeLjxhEd6j8PsuOxL | accepted
  acc-disp-dwSmYkULRU4HdSDJCDWH9 | accepted
  acc-disp-VWK5z1k7Jcu8-iQFrGVrb | pending
  acc-disp-NMcTvureVxwPh_3bFAGxx | pending
```

派给我的这三件一条都没有。等 Flux 的 verdict 落库后,按任务包第 3 节第 1 条
用 `base=track/infra-foundation` 开卡。

## 需要上报

1. **同一个 packet 在 `dispatches` 表里有两条,内容完全相同**:
   - `disp-SNGDYQnlpe2SA4QHwaYqb` — status=**ready**
   - `disp-Tg64EtzktcysA_ZAVl_tG` — status=**in_flight**(我正在做的这条)

   如果不是有意的重发,`ready` 那条会被再执行一遍,而它是纯流程单,
   重复执行的后果是再派一次 Flux、再发一次上报。已写进上报事件
   (`ev-tnKNlow-4CDScH2FOk7RN`)请张先生定。

2. **`ev-session-key-readmodel-live` 这个门从来没被发过。**
   ledger 里只有 `ev-session-key-readmodel-deferred`(10-16)和
   `ev-session-key-readmodel-landed`(10-16)。live 上的视图现在已经是真的存在,
   所以这个门的 live 条件已满足 —— 但发门是 Aegis 的动作,不是我这条派单的
   范围,我没有发。

3. 另外两件 pending 卡(`acc-disp-VWK5z1k7Jcu8-iQFrGVrb`、
   `acc-disp-NMcTvureVxwPh_3bFAGxx`)仍在等张先生裁决,与本次无关,顺带确认还在。

## 提交

`docs(acceptance): report three dispatches still unverifiable, and ask Flux to verify them`
(绝对哈希见 `git log --oneline -1`)。仅本说明文件,无代码改动 ——
这一单的正确动作本来就不是改代码。
