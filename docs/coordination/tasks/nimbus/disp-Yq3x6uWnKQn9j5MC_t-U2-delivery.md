# 交付说明 — disp-Yq3x6uWnKQn9j5MC_t-U2(session_work 读模型进 track)

分支 `packet/nimbus/session-work-readmodel`,HEAD `1572154`(2 个提交在 track 之上)。

## 任务 A:只把读模型带进 track

**没有做整分支合并。** 源分支 `packet/nimbus/gate-vs-announcement` 相对 track 改了
**120 个文件**,而它和 track 的共同祖先 `b6751a1` 之后,track 上每个共享文件都已经各自
往前走了(`db/mod.rs` 1 次、`models.rs` 6 次、`repositories.rs` 8 次、`lib.rs` 4 次、
mcp-emit 的 `main.rs` 5 次、`protocol.rs` 7 次、`commands/mod.rs` 8 次、src-tauri 的
`main.rs` 14 次)。整分支合并不是"多带了一些东西",而是把 track 一年的工作回退掉。

所以:**3 个新文件整体取,共享文件逐处手工登记。**

实际带进来的 11 个文件(相对 track 的完整 diff,一个都不少):

| 文件 | 性质 |
|---|---|
| `crates/seatloom-core/src/db/session_work.rs` | 新,655 行,9 个测试 |
| `crates/seatloom-mcp-emit/src/session_work.rs` | 新,270 行,5 个测试 |
| `src-tauri/src/commands/session_work_cmds.rs` | 新,110 行 |
| `crates/seatloom-mcp-emit/proof/session_work_readmodel_proof.sh` | 新,证明脚本 |
| `crates/seatloom-core/src/db/mod.rs` | 只加 `pub mod session_work` |
| `crates/seatloom-core/src/db/models.rs` | 只加 `MemorySessionJoinRow` |
| `crates/seatloom-core/src/db/repositories.rs` | 只加 6 个方法 + 1 个 SELECT 常量 + 1 个映射 helper + 3 个回归测试 |
| `crates/seatloom-mcp-emit/src/protocol.rs` | 只加 2 个 tool 构造函数的注册 |
| `crates/seatloom-mcp-emit/src/main.rs` | 只加 `mod` + 2 个 dispatch 分支 |
| `src-tauri/src/commands/mod.rs` | 只加 `pub mod session_work_cmds` |
| `src-tauri/src/main.rs` | 只加 import + 2 个命令注册 |

**测试文件**:源分支的测试就在上面两个 `session_work.rs` 里(9 + 5 = 14 个),没有独立的
测试文件。14 个全部在,全部通过。源分支的 UI 从未调用过 `session_work` 命令
(`git show ...:ui/src/lib/api.ts | grep session_work` 为空),所以 tauri 命令本身就是
UI 接口,`api.ts` 没有要带的东西。

## 任务 B:Flux 的 F1(panic)

**F1 是真的,已修。** `row_to_memory_session_join` 用 `r.get::<bool>` 读
`a1_resolves`/`a2_resolves`,NULL 会让它 panic,而 panic 会杀掉进程——连同整个 session
的血缘。这正好发生在这个读模型存在的目的所要捕捉的失败模式上。

**但对 live 库实测:现在不可达。** 022 的表达式是 `(s1.id IS NOT NULL)`,永远不为 NULL;
`select count(*) filter (where a1_resolves is null)` = **0 / 90**。

**仍然加固了**,因为这个列的布尔性住在一条迁移里。以后改视图让它可空,这里不会有任何
编译错误。修法是 `r.get::<_, Option<bool>>(n).unwrap_or(false)`。

F1 的第二条 must_fix(补一个回归测试)也做了。但要如实说明它的强度:`tokio_postgres::Row`
的构造器是私有的,没有数据库就没法喂一个 NULL 进去,所以**没有办法写一个真正的行为测试**。
写了三个能写的:两个方向共用同一个 SELECT(防漂移)、SELECT 只读存在的列(这正是当初漏掉
的那个检查)、以及一个源文本断言——把 `Option<bool>` 改回 `bool>` 会直接编译失败。
最后一个是源文本检查,比行为测试弱,注释里写明了。

**变异验证**:把 `Option<bool>` 改回 `bool>`,该测试 1 红。

## 过程中发现并修掉的一个更根本的问题

源分支的读模型是**对着一个从未存在过的 27 列视图**写的:它自己分支的
`infra/postgres/schema/` 只到 014,上面根本没有 022。track 上、live 里的 022 是
**20 列**,列名也不同(`resolved_by` 而不是 `join_state`,不暴露 `a2_harness`/`a2_id`/
`a1_corpus_path`/`a1_path_rows`,没有 `status`/`created_at`/行范围)。

这不是推测:拿原查询打 live,报 `ERROR: column "a1_harness" does not exist`。

所以没有去改那条迁移(改 live 库不是这个派单的事),而是**从 live 视图的 20 列 + memories
join 推导出那些额外列**:`a1_corpus_path`/`ref_harness`/`ref_id`/`a2_harness`/`a2_id` 用
视图自己那套表达式;`a1_path_rows` 数 `archived_sessions`(这才让歧义路径可报告);
`status`/`created_at`/行范围从 `memories` 取。

**推导恢复不了一件事**:两个键之间真正的分歧。视图把 `s1`/`s2` 合并了,两个都解析时它只说
`both`,不说它们是否指向同一个 session。所以 `join_state` 在这里永远不会是 `disagree`。
这个限制写在了常量的文档注释里,没有粉饰。

## 门禁(全部实测)

| 门禁 | 结果 |
|---|---|
| `cargo build --workspace` | 0 error |
| `cargo check --workspace --all-targets` | 0 error |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 error(F5 一并修了) |
| `cargo fmt --all -- --check` | clean |
| `cargo test --workspace` | **322 passed / 0 failed** |
| `session_work_readmodel_proof.sh` | **ALL POINTS PASS(8/8)** |

证明脚本的 scratch 是从 **track 的 022** 建的,不是从 Onyx 的 `0e32054`——那是另一个从未
上过 track 的 27 列视图。第一次跑脚本时它默认取的 `0e32054`,于是 point 1 过了、point 2 挂了,
报 `session_work rejected: postgres error`。那个失败只存在于 scratch 里,live 上没有。
脚本现在默认从 track 取,并留了注释说明为什么不能取 `0e32054`。

顺带:`session_work_join_states` 原来也在问 `join_state`,live 视图没有这一列。改成
`resolved_by`。它产出的那条 note 也改用视图自己的词——视图说不出"分歧",读模型说得出,
把 note 叫 `join_state` 就是在声称一个数据库做不出的区分。**变异验证**:把列名改回
`join_state`,证明脚本在 point 2 失败。

## live 库实测(只读)

- `v_memory_session_join`:**90 行**
- `resolved_by` 分布:**both = 87,a1_path = 3**
- `a1_resolves` / `a2_resolves` 为 NULL 的行:**0 / 90**
- 我这条 SELECT 打 live:90 行,`join_state` 得 `both_agree=87 / path_only=3`
- scratch 数据库已 `DROP DATABASE` 清掉

## 需要上报的

1. **【更正,2026-10-28】我原来在这里写的是错的,已划掉重写。**

   原文:「门事件 `ev-session-key-readmodel-live` 现在具备条件了:两个读接口都在 track 上」。

   **那句话是假的。** 两个读接口**不在 track 上**,它们只在
   `packet/nimbus/session-work-readmodel` 这条分支上(track tip 不含我的 HEAD,
   `git merge-base --is-ancestor HEAD track/infra-foundation` 为 false)。派单 A 的原话是
   「把那两个读取界面弄进 **track**」,我做成了「弄进一条 packet 分支」——分支名读起来像
   track,实际不是。门**没有发,也不能发**:程序里没有任何东西能读 live 那 90 行。

   实测(`git cat-file -e track/infra-foundation:<file>`):

   ```
   NOT ON TRACK: crates/seatloom-core/src/db/session_work.rs
   NOT ON TRACK: crates/seatloom-mcp-emit/src/session_work.rs
   NOT ON TRACK: src-tauri/src/commands/session_work_cmds.rs
   ```

   把「在我分支上」写成「在 track 上」,和派单 C 里禁止的「应该可以」是同一种错误:
   一个看起来像事实的句子,拦不住任何人,只会让下一棒以为前提已经成立。
   **任务 A 没有完成。** 需要把这 3 个文件(以及配套的登记)真正并进
   `track/infra-foundation`,那之后门才具备条件;并 trunk 是 Aegis 的判断,不是我自己
   动手的事。
2. **F2–F5 我没做**:F2(是否该报 `source_present`)、F3(零记录时用 session 自己的行事实而不是
   `rows.first()`)、F4(重写 session 级 note 让它对歧义情形也准确)都是行为判定,属于读模型的
   设计方;F5(clippy)已随手修掉,因为不修工作区门禁就是红的。
3. **`a1_harness`/`a1_id` 是从视图的 `session_harness`/`session_id` 取的**——视图已经把 A1
   解析到的 session 合并进去了,所以这两个别名只是改名。如果上游要的是"A1 的路径指向的那个
   session"而视图合并的可能是 A2 的,这里会不一致。live 上 87/90 是 `both`,区分不出来。
