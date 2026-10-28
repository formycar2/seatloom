# NIMBUS-2026-10-26-ui-shows-tilde-project-name-v1 — 交付说明

派单 `disp-NMcTvureVxwPh_3bFAGxx`(目标 `seat-nimbus-001`)。
任务包 `docs/coordination/tasks/aegis/AEGIS-2026-10-26-ui-shows-tilde-project-name-v1.md`。
分支 `packet/nimbus/project-scoped-reads`。

## 判据 1、2、3 一次答完:那一行在哪、值从哪来、`~/` 怎么产生的

**渲染那一行:`ui/src/components/ProjectSwitcher.tsx:107`**

```tsx
<div className="text-[10px] text-muted-foreground truncate font-mono opacity-80 mt-0.5">{project.path}</div>
```

(同一份 `project.path` 还有两处显示:`ProjectOverview.tsx:53`、`AllProjectsView.tsx:125`。)

**值的来源:`projectFromDto`,在 `ui/src/lib/dto-to-v1.ts:153`(改前)**

```ts
export function projectFromDto(d: ProjectDto): Project {
  return {
    id: d.id,
    name: d.name,
    path: '~/' + d.name,   // ← 就是这里
    ...
```

**`~/networkorchestrator` 就是 `'~/' + d.name` 当 `d.name === 'networkOrchestrator'` 时的结果。**
账本里 `projects.name = 'networkOrchestrator'`(实测),所以拼出来正是张先生看到的那个串。

### 为什么任务包排除了三条却仍然漏了

Aegis 排除的第 2 条(后端 `ProjectDto` 没有 `home_path`)和第 3 条(UI 源码没有 `~` 逻辑)
都是**对的**,但结论「前端拿不到路径,所以不可能显示路径」只对了一半:
前端拿不到真路径,于是它**自己造了一个**。`'~/' + d.name` 不在 `dto-to-v1.ts` 之外,
所以 `grep "'~'"` 抓不到它 —— 它是一个表达式,不是字面量。

第三条(`grep 'project_delivery' ui/src` 为空)也对,但同样不构成反证:
UI 显示的一直是 `name`,不是 `id`。`~/networkorchestrator` 和
`project_delivery/networkOrchestrator` 本来就不该在同一次渲染里匹配。

### 一个值得记的旁证:这个 `~/` 是故意的,然后泄漏了

`useDataStore.ts:88-99` 的注释写明了来历 —— 2026-10-16 张先生看到三条**写死的假项目卡**,
而真假的分辨依据是「假卡的路径是 `~/Documents/GitHub/…`,而 mapper 合成 `~/<名字>`」。
`'~/' + d.name` 是当一个**识别水印**用的,不是当数据显示的。

水印活干完了没拆。于是每一 个真项目都顶着一条假路径。
**这是我最想指出的那一类问题:一个临时判据被留在了产品路径上。**

## 修法:把真路径一路接过来,没有就说没有

不是把 `~` 去掉 —— 那会让那一行变成空白,而张先生下次仍然不知道为什么这里有个空。
也不是换个合成规则。**账本里有真的。**

链路四层,全接上:

| 层 | 文件 | 改动 |
|---|---|---|
| DB 查询 | `crates/seatloom-core/src/db/repositories.rs:81,93` | `SELECT` 加 `home_path`;`row_to_project` 加 `r.get(5)` |
| 行模型 | `crates/seatloom-core/src/db/models.rs:7` | `ProjectRow.home_path: Option<String>` |
| DTO | `src-tauri/src/dto.rs:25,33` | `ProjectDto.home_path: Option<String>`,`From` 透传 |
| UI 类型 | `ui/src/lib/types-dto.ts:8` | `ProjectDto.homePath: string \| null` |
| mapper | `ui/src/lib/dto-to-v1.ts:153` | `path: d.homePath ?? null`(不再合成) |
| UI 模型 | `ui/src/types/index.ts:274` | `Project.path: string \| null` |
| 三处显示 | `ProjectSwitcher.tsx:107`、`ProjectOverview.tsx:53`、`AllProjectsView.tsx:125` | `{project.path ?? '无 home_path'}` |

**没有 `home_path` 的行就显示「无 home_path」**,不显示空、不显示猜测。
实测:39 个项目**全部**有 `home_path`(`count(home_path)` = 39,`count(*) - count(home_path)` = 0),
所以现在界面上不会出现「无 home_path」。那句话是给将来用的 —— 021 之外的加载路径
可能写入空路径,那时候它说「没有」而不是编一个。

## 端到端实测(不是读代码)

写了个一次性 probe 跑真库(用完已删):

```
$ DATABASE_URL=postgresql://… cargo run --example project_path_probe
id=project_delivery/networkOrchestrator name=networkOrchestrator
home_path=Some("/Users/jyxc-dz-0100609/Documents/GitHub/project_delivery/networkOrchestrator")
```

也就是**修完之后,那一行会显示账本里那条完整路径**,和同一份截图里另一处已经正确的
路径一致。

## 测试:两个方向都钉住,而且证明过它们会红

`ui/src/views/AllProjectsView.hydration.test.tsx` 加了 1 个、改严了 1 个:

1. **`the ledger home_path is rendered verbatim, and a missing one says so`**(新)——
   真路径必须原样出现;`~/networkOrchestrator` 必须不出现;没有 home_path 的必须说「无 home_path」。
2. **`a hydrated project renders the ledger path, never a synthesised one`**(原测试改严)——
   原来只断言种子路径不出现;加了一条 `expect(screen.queryByText('~/ql_delivery')).toBeNull()`。

**两个测试都验过会红**:我把 `path: d.homePath ?? null` 临时改回 `'~/' + d.name` 跑一遍,
两个都 FAIL;改回来两个都 PASS。不会出现「测试永远绿」那种事。

原测试的**断言一条没动**,只加了新的;文件头注释里那段「mapper 合成 `~/<名字>`」的
说明改成了「2026-10-26 已移除」并保留了为什么(水印的来历),免得下一个人以为是漏写。

## 验证

| 检查 | 结果 |
|---|---|
| `cargo build --workspace` | 0 warning |
| `cargo test --workspace` | 全绿 |
| `cargo clippy --workspace --all-targets` | 0 warning |
| `cd ui && pnpm tsc --noEmit` | 干净 |
| `cd ui && pnpm vitest run` | **207 passed**(改前 206,新增 1) |
| 新测试在旧代码上 | **2 failed**(证明它们真的在守) |

## 明确不做的,确认没做

- **没有只把 `~` 去掉不解释来源** —— 来源找到了(`'~/' + d.name`),并且接上了真数据。
- 没改压缩阈值、没改任何业务逻辑,没动那三张 blocked 单。
- 没把显示改成空字符串 —— 空字符串和假路径一样是「看起来像数据」。
- 没合并、没开验收卡。

## 需要上报

1. **这个界面改动要张先生看到才算数。** 我证明了 `home_path` 从库里正确一路到
   `ProjectRow`(probe 实测),但**没有重新构建 app、没有安装** —— 上一次派单
   (`AEGIS-2026-10-26-tauri-bundle-missing-frontend-v1`)里他说过装之前要先问,
   我沿用那条:要不要重新构建 + 安装,等他一句话。
2. **`home_path` 为 NULL 的项目数,我原来准备上报"没数",数完了:0。**
   实测 `select count(*), count(home_path) from projects` → `39 | 39`,一条都不缺。
   所以这次修完之后界面上不会出现「无 home_path」,张先生看到的每条都会是完整路径。
   (我在写交付说明时先按"没数"起好了草,数完发现是 0,就把草改了 —— 记这个是因为
   一条"我没测"的上报和一条测错了的上报一样会让人做错决定。)
3. **`isPinned: true` 是硬编码的**(`dto-to-v1.ts:155`,这次之前就是这样)。
   每个项目都显示成已置顶。这不是我改出来的,也不在这一单的范围里,但它是同一类
   「mapper 里写死的值冒充数据」的问题,记在这里免得下次又被当成新发现。

## 提交

`fix(ui): the project path comes from the ledger, not from '~/' + name`
(绝对哈希见 `git log --oneline -1`)。
