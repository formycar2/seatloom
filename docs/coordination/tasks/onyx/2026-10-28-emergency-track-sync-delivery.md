# 紧急同步派单 —— delivery

Onyx. 2026-10-28. 按张先生直接指示做的同步单(3 张紧急同步单里没有一张派给 Onyx,
见第 1 节;这单是 operator 直接指令,不走 dispatch)。

**结论:已 merge track 并 push 到两个 remote,两条判据都过。合并中发现并修掉一个
真问题 —— 迁移编号撞车。**

---

## 1. 先说清楚这单的来历(不把我的猜测写成事实)

Aegis 04:17:46 派的 3 张紧急同步单,目标席位是:

| 派单号 | 目标 | 卡数 |
|---|---|---|
| `disp-tVzSMjxpWpBQrW3dToQej` | flux | 8 |
| `disp-YepFZb2hc62cZEO1UkJFX` | nimbus | 4 |
| `disp-Hn6S8umZMWB3_FKYgkhqq` | mira | 11 |

**没有一张给 Onyx。** 我 `get_next_dispatch` 三次都是 `nothing ready`,
账本上也没有 `SeatDispatchIssued` 指向 seat-onyx-001。

我另查了账本上 23 张 pending 验收卡,**没有一张是 Onyx 的**(全是 flux/nimbus/mira)。
所以「你分支上所有 pending 卡现在都点不了」这个前提,对 Onyx 不成立 —— 我 0 张卡。

**但那个底层条件是成立的,而且我查出来了**:我的分支落后 track **374 个提交**。
所以这活我照做了(张先生直接指令),只是前提要按实测说,不照抄。

## 2. 三个 track  tip(这条要报,因为它会骗人)

| ref | commit | 日期 | 是不是真的 |
|---|---|---|---|
| 本地 `track/infra-foundation` | `3881d57` | 2026-10-28 | **活的**(Aegis 今天的推送) |
| `origin/track/infra-foundation` | `4c13ffc` | 2026-09-30 | 落后,别用 |
| `gitlab/track/infra-foundation` | `b09ec6e` | 2026-10-09 | 落后 **387** 个提交,别用 |

ENGINEERING_STANDARDS 写「GitLab is the primary」,但 gitlab 的 track ref 停在
10-09,本地 ref 才是今天的状态。**我用的是本地 ref** —— 这也是 Nimbus 已经做完的那单
(`a7353e9` merge)用的同一个 ref,和已完成的那单对得上。

## 3. 合并怎么解的(三个冲突)

### 3.1 `022_session_work_bridge_view.sql`(add/add)

两边都落了 022。track 的是同一个 migration、10-26 重新量过的更新版
(多了 `resolved_by` / `missing` / `a1_path` 列,为 `ev-session-key-readmodel-live`
门写的)。视图名、承载数字都一样。**取 track 的。**

### 3.2 `scripts/apply-schema-to-live.sh`

track 的版本**自己从目录里发现迁移**,不写死文件名 —— 正是治写死列表那个老毛病
(它自己的注释写着 10-11 写死 015-018 之后 019 一落地整条链又卡了一次)。
**取 track 的。** 我这分支的写死列表里那个 `020_gate_criterion.sql` 在冲突 3.3
之后已经不存在了,取 track 版正好把这个死引用一起带走。

### 3.3 `scripts/apply-db.sh` —— **迁移编号撞车(真问题)**

两边各自独立落了一个编号 **020** 的 migration:

| 分支 | 文件 | 内容 |
|---|---|---|
| track | `020_operator_decisions.sql` | operator decisions 对象 |
| 我的 | `020_gate_criterion.sql` | gate vs announcement 判据 |

一个编号只能有一个文件。**把 我的 改成 `023_gate_criterion.sql`**(023 两边都空着),
改的只是**文件名和这个 migration 自己的自引用**(文件头、RAISE NOTICE 字符串、
COMMENT ON 正文)—— 没有任何 SQL 对象改名,依赖关系不变。然后在
`POST_SEED_SCHEMA` 里登记 023,排在 022 之后最后应用。

**没有去动 track 的任何源码。** 特别是:track 的
`crates/seatloom-mcp-emit/src/gate.rs:350` 注释里引用了这个文件的**旧名字**
`020_gate_criterion.sql`,现在是**过时引用**。我没有改它 —— 从我这条 packet 分支改
track 的源文件,等于把别人的文件塞进我的合并里。这个记在第 5 节,留给 Aegis 派单。

## 4. 提交前按规矩先编译(AGENTS.md 第 6 条教训)

| 门 | 结果 |
|---|---|
| `cargo check --workspace` | **PASS**(需要 `ui/dist/index.html` stub,gitignore 覆盖,未提交) |
| `cargo fmt --all -- --check` | **PASS** |
| `cargo clippy -p seatloom-core --all-targets -- -D warnings` | **PASS** |
| `apply-db.sh` 完备性守卫 | **PASS**(磁盘上每个迁移都已登记) |
| 全树冲突标记 | **0** |

## 5. 判据(派单原文的两条,在**已 push 的远端分支**上量的)

```
$ git rev-list --count FETCH_HEAD..track/infra-foundation
0                                    ← 一条不缺

$ git merge-base --is-ancestor track/infra-foundation FETCH_HEAD
exit 0                               ← 卡可点
```

远端 tip = `332ebbc`(merge commit),和本地一致。

`git rev-list --count HEAD..track` 我在合并**前**量到的是 374,合并后 0。
注意:合并过程中本地 track ref 又往前走了一个提交(`9b5173a` → `3881d57`),
我的 merge commit 的第二个 parent 就是 `3881d57`,所以拿的是最新状态,不是旧的。

## 6. 需要你/ Aegis 处理的三件事(不替我圆)

1. **track 的 `gate.rs:350` 注释还写着 `020_gate_criterion.sql`** —— 文件名已改成
   023。只是注释,不影响编译,但引用是错的。该由 track 侧改,不由我从 packet 分支改。
2. **迁移编号没有分配者**。今天两个席位各拿了一个 020,是我改号避开的;
   下一次还是会撞。要么 Aegis 统一发号,要么落号前先查 `apply-db.sh` 的登记表。
   (这条已用 `remember` 记了。)
3. **gitlab 的 track ref 落后 387 个提交**,而标准里写着 GitLab 是 primary。
   谁再照 gitlab ref 做合并,会把今天的活全丢掉。

## 7. 边界

- 没往 track 合并我的分支(那是 operator 的 Accept)✅
- 没 rebase、没删改写提交 ✅
- 没开新卡 ✅
- 没改 track 的源码 ✅
- 没动 live 库 ✅(全程只读 git;psql 本机没有,也没试)

## 8. 前向记录

记了一条 `failure`:迁移编号没有分配者,两个席位各落一个 020 的根因与修法。
