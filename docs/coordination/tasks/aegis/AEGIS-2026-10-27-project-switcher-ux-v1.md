# 项目选择器的交互体验 —— 五个实测出来的问题

| Field | Value |
|---|---|
| id | AEGIS-2026-10-27-project-switcher-ux-v1 |
| target_seat | seat-mira-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-27 |

## 背景

张先生 2026-10-27:"SeatLoom UI 项目选择器,产品交互需要提升体验。"

Aegis 先读了代码(`ui/src/components/ProjectSwitcher.tsx`,159 行),
**查出五个问题,全部有文件行号,不是印象**。怎么改由你定,这条派单不下设计。

## 实测数据

账本里 **39 个项目**(`select count(*) from projects`)。

## 问题 1:搜索框的占位符承诺了一个不存在的快捷键(最该先修)

`ProjectSwitcher.tsx:59` 的 placeholder 写着:

```
搜索项目或快速切换（Ctrl+K）
```

**Ctrl+K 打不开这个选择器。** 实测:

- `ui/src/App.tsx:366` —— Ctrl+K 是**开关 Supervisor IM**(L1 面板)
- `ui/src/hooks/useGlobalShortcuts.ts:70` —— Ctrl+K 是 **Supervisor Command Bar**

两条都注册了,`preventDefault()` 都调了。**没有一个打开 ProjectSwitcher。**

所以那句话是在邀请用户按一个做不到的组合键。按了之后出现的是别的东西 ——
比没有提示更糟。

## 问题 2:"最近访问"那一列显示的是创建时间

`ui/src/lib/dto-to-v1.ts:160`:

```ts
last_accessed: d.createdAt,
```

后端 `ProjectDto` **没有** last_accessed 字段(`src-tauri/src/dto.rs:25`
只有 id/name/created_at/两个预算)。所以前端拿创建时间冒充访问时间。

`ProjectSwitcher.tsx:113` 把它显示成 `{formatShortDateTimeZh(project.last_accessed)}`,
用户看到的是"这个项目什么时候建的",不是"我什么时候来过"。

`useDataStore.ts:1624` 倒是有在切换项目时更新 `last_accessed`(更新的是
**内存态**),但:

- 它不落库(后端没这个字段)
- 重启就丢
- **而 mapper 每次都用 createdAt 覆盖它** —— 所以连内存里那次更新,
  下次 hydrate 也会被抹掉

## 问题 3:分组标题和排序不符

`ProjectSwitcher.tsx:72` 的标题是「**最近访问与置顶项目**」,
但 `ProjectSwitcher.tsx:31` 的排序只有:

```ts
.sort((left, right) => (left.isPinned === right.isPinned ? 0 : left.isPinned ? -1 : 1))
```

**只按置顶排,没按 last_accessed 排。** 标题承诺了两件事,只做到一件。
(而且第 2 条说明另一件现在也做不到。)

## 问题 4:没有键盘导航

整个文件 `grep onKeyDown|ArrowDown|ArrowUp` **零命中**。

39 个项目、列表 `max-h-[420px]`,只能鼠标点或滚。
**搜索框是 autoFocus 的,所以用户的手已经在键盘上**,却只能用鼠标选。

## 问题 5:搜索只匹配项目名

`ProjectSwitcher.tsx:30`:

```ts
.filter((project) => project.name.toLowerCase().includes(search.toLowerCase()))
```

39 个项目里有很多层级关系(`project_delivery/QL`、`sf-code/stepcloud`、
`ai-assistant/skills`...)。搜 `ql` 能命中,搜 `sf-code` 里的某个子项目
就得打全名。**路径/父级不参与匹配。**

## 怎么改由你定,但有几条边界

1. **不许把假数据改成另一种假数据。** 问题 2 的正解要么接真数据
   (后端加字段 + 落库),要么**不显示那一列**。
   不许换个说法继续显示 createdAt —— 那正是你今天修掉的那个 `~/` 假路径的形状。
2. **placeholder 要么做到,要么删掉。** 如果你实现 Ctrl+K 打开它,
   要先解决和 App.tsx:366 的冲突(那个已经是既定的 L1 交互)。
   删掉那句话也是一个可接受的答案。
3. **不要新增交互范式。** 键盘导航、按路径搜索,这些都是既有组件的常规做法,
   不是新设计。张先生说过"基于现有的 SeatLoom 设计来完善功能"。
4. **39 个项目这个数字会变。** 别按 39 写死任何分组或分页。

## 判据

- 五个问题逐条说:改了/不改,以及为什么
- 如果你判断某个不是问题,给出反证
- 测试:键盘导航要有测试(按下键移动、回车选中、Escape 关闭);
  搜索要有测试(匹配 name、匹配路径、空结果)
- 不许加永远绿的测试

## 明确不做

- 不许顺手改 Supervisor IM 或 Command Bar 的行为
- 不许改账本、不许改后端 schema(那要另派单)
- 不许改视觉风格(颜色/圆角/字号)—— 这条是交互,不是视觉
