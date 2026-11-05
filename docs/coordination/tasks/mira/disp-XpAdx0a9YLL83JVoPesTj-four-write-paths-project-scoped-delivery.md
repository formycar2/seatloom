# disp-XpAdx0a9YLL83JVoPesTj 交付说明 —— 四个写路径不再落进硬编码项目

**派单**:`disp-XpAdx0a9YLL83JVoPesTj`(Lyra 发)
**提交**:`ad78e68`
**门禁**:`cargo test` 23 passed / `vitest` 385 / 31 files / tsc 净 /
eslint 145 problems(135 errors + 10 warnings)= 主检出 baseline,零新增 / build 1.99s

---

## 1. 缺陷,以及为什么它比"读路径修了、写路径没修"更值得说

读路径逐个改用了 `require_project_id` —— **拒绝缺失的 project_id,不静默退回**。
它的 doc comment 自己记了理由:

> A fallback that silently substitutes another project's rows is worse than an error:
> the error says something is missing, while the fallback says nothing at all.

四个写/spawn 路径从来没得到这个待遇。它们把 `state.default_project_id` 传给子进程,
而那个常量在 `main.rs:120` 硬编码成 `"seatloom"`。

**Lyra 实测的爆炸半径**(我核了四处调用点,一条条对上):

| 事实 | 数字 |
|---|---|
| live 项目数 | 40 |
| 有席位绑定的项目 | 4 |
| 其中**不是** seatloom 的 | **3** |
| 这 3 个各自绑定的席位数 | 各 1 |
| 这 3 个席位是否也在 seatloom 下绑定 | **否** |

**所以这不是边缘情况,是那 3 个项目的常规路径。**

操作者看到的:他在项目 X,seat 下拉列出 X 的那一个席位(`DispatchInboxView.tsx:665`
`.listSeats(activeProjectId)`),他填表单点提交。表单关闭、列表刷新、**页面空白** ——
因为列表读 `cmd_list_dispatches({projectId: X})`,而新行的 `project_id` 是 seatloom。

**没有任何东西告诉他去哪了。**(它确实在 seatloom 下,按原始 seat id 分组 ——
`DispatchInboxView.tsx:751-753` 回落到 `seatId`。所以是"归档错且不披露",不是"消失"。)

## 2. 改了什么

### Rust 四处

| 文件 | 命令 | 改动 |
|---|---|---|
| `dispatch_cmds.rs` | `cmd_enqueue_dispatch` | 加 `project_id: Option<String>`,`require_project_id` |
| `dispatch_cmds.rs` | `cmd_plan_intent` | 同上 |
| `history_cmds.rs` | `call_tool`(共享) | 加 `project_id: &str` 参数,三个调用方传入 |
| `session_work_cmds.rs` | `call_tool`(共享) | 同上,`cmd_session_work` / `cmd_memory_session` 加 `project_id` |

**`cmd_list_history_projects` 原本没有项目参数** —— 它返回的是全局列表,
看起来不需要。但子进程仍需一个项目在作用域内,而它原来退回启动常量。
**所以它也改成 `Option + require`** :这条路径拒绝,而不是伸手拿常量。

### 前端

| 文件 | 改动 |
|---|---|
| `api.ts` | `enqueueDispatch` / `planIntent` 加 `projectId`;三个 history 读加 `projectId` |
| `DispatchInboxView.tsx` | 派单表单传 `activeProjectId` |
| `SupervisorIntent.tsx` | 两处 `planIntent` + `enqueueDispatch` 传 store 当前项目 |
| `HistorySessionsPanel.tsx` | `HistoryLoader` seam 加 `projectId`,`SessionWorkSection` / `HistorySessionDetailView` 逐层传 |

### 三处 effect 依赖

`HistorySessionsPanel` 里三个 `useEffect` / `useCallback` 的依赖数组加了 `projectId`。

**这不是为了让 eslint 闭嘴** —— 项目变了必须重读,否则切项目后会显示上一个项目的语料。
每一处都写了注释说明为什么属于那里。

## 3. 测试:两个方向都失败了,如派单所要求

**Rust 侧没法单测这两个命令** —— 它们 spawn 子进程讲 JSON-RPC。
所以我加的是**源码契约测试**(`mod spawn_path_project_scoping_tests`):

```rust
// 六处 require_project_id 调用
for (file, command) in [
    ("dispatch_cmds.rs", "cmd_enqueue_dispatch"),
    ("dispatch_cmds.rs", "cmd_plan_intent"),
    ("history_cmds.rs", "cmd_list_history_projects"),
    ("history_cmds.rs", "cmd_get_history_session_turns"),
    ("session_work_cmds.rs", "cmd_session_work"),
    ("session_work_cmds.rs", "cmd_memory_session"),
] { ... }

// 三处 spawn 不再传启动常量
for file in ["dispatch_cmds.rs", "history_cmds.rs", "session_work_cmds.rs"] { ... }
```

**为什么是源码检查而不是 mock**:缺陷恰恰是"子进程拿着一个常量启动",
而可观测后果(派单归错项目)只在真库 + 两个项目时才出现。
源码检查在常量回来那一刻就失败 —— 而那正是回归会来的方向。

**注意第二个测试数的是 `.env("SEATLOOM_PROJECT_ID"...)` 调用,不是任何出现。**
我的注释块里**合法地提到了那个常量名**(记录删掉了什么),
所以按调用计数,不按文本计数。

### 派单要的两个方向

| # | 变异 | 结果 |
|---|---|---|
| 1 | 把子进程的 `SEATLOOM_PROJECT_ID` 换回 `state.default_project_id` | **1 红** —— `no_spawn_path_hands_the_child_the_startup_constant` |
| 2 | `cmd_enqueue_dispatch` 的 `require` 换成 `unwrap_or_else`(静默退回) | **1 红** —— `each_of_the_four_calls_require_project_id` |

恢复后 `cargo test` 23 passed。

## 4. 门禁

```
cargo test          → 23 passed; 0 failed
pnpm test           → 385 tests / 31 files
pnpm typecheck      → clean
pnpm exec eslint    → 145 problems (135 errors + 10 warnings)
                       = 主检出 baseline,零新增
prettier            → clean
pnpm build          → 1.99s
```

**eslint 我一度新增了 4 个 warning**(三处 `projectId` 缺依赖 + 一处),全部按"依赖属于那里"补上,
回到 baseline。**没有用注释压掉。**

## 5. 未做

- **没有删 `state.default_project_id` 或 `main.rs:120`** ——
  派单明确说它有其他读者,是单独的决定。**我只是让这四个不再用它。**
- **没有 merge into track、没有重启应用、没有写 live DB**(派单 `do_not`)。
- **没有碰 Nimbus 的 124 行未提交 Rust**(`repositories.rs` / `memory.rs`)。
- **gitlab 仍未推** —— 已知不可达。origin 已推到 `ad78e68`。
