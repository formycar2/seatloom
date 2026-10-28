# UI 里出现 `~/networkorchestrator` —— 定位它,并判断是不是 bug

| Field | Value |
|---|---|
| id | AEGIS-2026-10-26-ui-shows-tilde-project-name-v1 |
| target_seat | seat-nimbus-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-26 |

## 现象(张先生 2026-10-26 19:51 报告)

SeatLoom 应用窗口里,一个**项目下拉框**中,除了正确的
`project_delivery/networkOrchestrator`,还有一项显示为 `~/networkorchestrator`。

同一份截图中,另一处(派单相关的字段)显示的项目目录是**正确的完整路径**
`/Users/jyxc-dz-0100609/Documents/GitHub/project_delivery/networkOrchestrator`,
和账本一致。

## 我要你做的

1. **定位渲染 `~/networkorchestrator` 的那一行代码** —— 哪个组件、哪个表达式
2. **判断它是不是 bug**,并说明依据
3. 如果是 bug,修;修完给判据

## Aegis 已排除的(别重复查,这些都是实测不是推测)

1. **账本里没有这个值**:`projects` 表只有
   `project_delivery/networkOrchestrator`(id)、`networkOrchestrator`(name)、
   `/Users/.../project_delivery/networkOrchestrator`(home_path)。没有 `~` 开头的行。
2. **后端不给前端 home_path**:`src-tauri/src/dto.rs:25` 的 `ProjectDto` 只有
   id / name / created_at / worker_budget_tokens / supervisor_budget_tokens。
   **前端拿不到路径,所以不可能显示路径。**
3. **UI 源码里没有 `~` 逻辑**:`grep -rn "'~'" ui/src/` 为空;
   `grep -rn 'project_delivery' ui/src/`(排除 fixtures)为空。
4. **项目表只有两个**:`projects` 和 `project_role_bindings`。
5. **跑着的应用里 `networkOrchestrator` 出现 0 次** ——
   这条只说明项目名是运行时从库读的(不可能硬编码在 JS 里),
   **不能**用它证明 UI 显示什么。

## 你可能要用的方向(你自己判断,别照抄)

- 那个下拉框是哪个组件?搜 `<select` 和项目相关的 option
- 它调的是哪个接口?`cmd_list_projects`?还是别的?
- 🔴 **如果它显示的是 `name` 字段而不是 `id`**,那 `~/networkorchestrator`
  就解释不通(账本 name 是 `networkOrchestrator`),得另找来源
- 会不会有第二份 SeatLoom 在跑?Aegis 实测 19:47 有过第二个实例
  (pid 10683,来自 `target/release/bundle/macos/`),它显示的可能不是同一份代码
- **跑的是哪份二进制**这件事要先确认:张先生看到现象的那个窗口,来自哪个路径

## 判据(可重跑)

给出实际输出,证明:
1. 那一行代码在哪(文件:行号 + 表达式)
2. 它取的值来自哪里(哪个接口/表/字段)
3. `~/networkorchestrator` 这个字符串是怎么产生的(或者:证明它不可能由当前代码产生)

## 明确不做

- 不许为了让现象消失而改判据或改显示(比如把 `~` 去掉但不解释它从哪来)
- 不许顺手改业务逻辑
- 如果结论是"当前代码不可能产生这个字符串",**要说出它可能从哪来**,
  不要只说"我复现不了"—— 张先生看到了,那就是有个来源
