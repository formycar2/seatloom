# 合并防护的显式删除豁免清单

| Field | Value |
|---|---|
| id | NIMBUS-2026-10-22-containment-allowlist-v1 |
| dispatch | disp-5WQ7UeLjxhEd6j8PsuOxL |
| packet | AEGIS-2026-10-22-containment-allowlist-v1 |
| target_seat | seat-nimbus-001 |
| date | 2026-10-22 |
| branch | packet/nimbus/project-scoped-reads |

## 改了什么

一个文件:`crates/seatloom-mcp-emit/src/dispatch.rs`(+279 −3)。

三处改动:

1. **清单本体**(Tool 5 之前新增一节):`DeletionExemption` 结构 + `DELETION_EXEMPTIONS`
   常量,当前一条 —— `ui/src/app-v2/AppV2.tsx`,`ruled_by: "human:supervisor"`,
   `why` 带派单号 `disp-DFeH6Ut_aNfzIbuuR21V5` 第 3b 项和「零 import、无路由挂载、
   在主线上就已不可达」。
2. **判定处**(`request_acceptance` 的 containment (c)):合并预览算出的删除集合先
   `filter(|p| !is_deletion_exempt(p))`,**非空才拒绝**。空 = 所有删除都被裁定了,
   没什么可防的。
3. **错误信息**:被豁免的路径不再出现在 `MergeWouldDelete` 里 —— 它已经被减掉了,
   列出来只会让操作者以为还有东西要处理。

另外把 `MergeWouldDelete` 变体的 doc 注释改了,写明「被裁定的删除不在这里,它在决策前
就被减掉了,所以这个变体只装还在被防的东西」。

## 清单与 `MergeWouldDelete` 的关系

- **只对这一条防护开口**。`BranchNotSynced` 一行代码都没碰:一个没见过主线的分支,
  跟哪个操作者裁定过哪个文件无关。测试 3 专门证明这点。
- **不可由环境变量或运行时配置注入**。它是 `const`,唯一的入口是改代码、走审核、
  留 git 记录。`SEATLOOM_*` 环境变量在这个判定里完全不出现。
- **精确匹配,不是前缀/通配**。`is_deletion_exempt` 是 `==`。`ui/src/app-v2`、
   `ui/src`、`AppV2.tsx`、`ui/src/app-v2/AppV2.tsx.bak` 都不继承豁免。
- **减法语义**。清单让 N 个删除通过,第 N+1 个照样被拦,并且把剩下的全列出来。

## 测试怎么证明它没被削弱

三个测试,加在既有 `request_acceptance_refuses_unsynced_and_destructive_branches`
旁边(那条**原样未动**,仍然通过):

1. `exempt_deletion_passes_while_the_rest_are_still_refused` —— 一个 scratch 仓库
   三种形状:
   - `packet/seat/exempt`:已同步,只删被裁定的路径 → **不再拒绝**,请求穿过防护到
     达数据库(死 pool 证明它走到了那一步);
   - `packet/seat/mixed`:已同步,删被裁定的路径 **外加** `keep.txt` → **仍然拒绝**,
     且 `paths` 精确等于 `["keep.txt"]`(减法只减它自己那条),错误信息里
     **不出现** `AppV2.tsx`;
   - `packet/seat/unsynced`:只删被裁定的路径但从未同步 → `BranchNotSynced`,
     清单对它无效。
2. `deletion_exemptions_are_explicit_audited_and_exact` —— 审清单本身:每条都有
   path/ruled_by/why,不含 `*`、`?`、`[`;并且逐个断言 8 个「近似路径」都不继承豁免,
   包括 `*` 自己。**这条测试的意义就是防将来有人把清单改成通配**:谁把
   `"ui/src/app-v2/AppV2.tsx"` 换成 `"*"`,这条测试和上面的 glob 断言会同时红。
3. **编译期审计**(不是测试,是 `const _: () = { ... }`):条目缺 `ruled_by` 或缺 `why`
   **直接编译失败**。实测把 `ruled_by` 清空:

   ```
   error[E0080]: evaluation panicked: no `ruled_by`: nobody can tell whose ruling this is
   ```

   之所以放编译期:测试是可以被删的,而「半年后没人记得为什么那一行在,于是当成历史
   包袱绕过它」正是这个清单要防的事。审计字段不可删,比有人写个测试审它更稳。

## 验证

| 命令 | 结果 |
|---|---|
| `cargo build -p seatloom-mcp-emit` | 过,零 warning |
| `cargo build --workspace` | 过 |
| `cargo test --workspace` | 全绿(emit 83 passed / 16 passed;其余 suite 全 ok) |
| `cargo clippy -p seatloom-mcp-emit --all-targets` | 零告警 |
| `cargo fmt -p seatloom-mcp-emit -- --check` | clean |

编译期审计的反向验证:把 `ruled_by` 临时清空 → 编译失败并打出上面那句;恢复后重新
build 通过。这条是手动做的一次性验证,不是常驻测试。

## 没做的事(按 packet「明确不做」)

- 没加环境变量开关
- 没给 `BranchNotSynced` 开任何口子
- 没删 `ui/src/app-v2/AppV2.tsx`(那是合并时的事)
- 没改其它防护的判定
- 没合并、没开验收卡

## 一条给 Aegis 的观察(不在本派单范围内,未动代码)

`crates/seatloom-core/src/git/ops.rs` 头部注释写:

> `request_acceptance` runs all three at request time and
> `cmd_accept_delivery` re-runs them at decision time

实测**不成立**:全仓库对 `branch_contains` / `merge_would_delete` 的调用只有
dispatch.rs:1808 和 1822 两处,`cmd_accept_delivery`
(`src-tauri/src/commands/acceptance_cmds.rs`)只做「是 git 树 / 工作区干净 / 分支存在 /
base 存在 / 在 base 上 / merge,冲突则 abort」,**不重跑 containment**。

影响面:一张在防护存在之前录下的 pending 卡,点 Accept 时不会被 containment 复查
(注释承诺过会)。本次改动没有扩大也没有缩小这个缺口 —— 豁免清单只作用于
`request_acceptance`,点击时的合并路径照样会真的执行那个删除。

要不要给 `cmd_accept_delivery` 补上决策时重跑(以及豁免清单是否也该在那时生效),
请 Aegis 裁。我不自己动。
