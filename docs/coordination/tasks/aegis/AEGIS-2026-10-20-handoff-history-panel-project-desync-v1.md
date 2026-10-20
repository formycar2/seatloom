# 交接:历史会话面板与全局项目选择脱节

| Field | Value |
|---|---|
| id | AEGIS-2026-10-20-handoff-history-panel-project-desync-v1 |
| date | 2026-10-20 |
| source | 张先生观察到:「会话标签里每一个条目都是一个项目,没有按 SeatLoom UI 的项目选项过滤」 |
| verified_by | Aegis,逐行读代码确认 |

## 根因(已定位到行)

`ui/src/app-v2/panel/HistorySessionsPanel.tsx:765`

```ts
const [projectId, setProjectId] = useState<string | null>(null);
```

面板自己维护一份 `projectId` 本地状态,`:892` 把这份本地值当"当前选中项目"
传给项目列表做高亮。**它从不读取全局的项目选择。**

因此左上角项目选择器切到别的项目时,这个面板完全不动,两个选择互不相干。

## 后端没有问题

`crates/seatloom-core/src/db/repositories.rs:986` 的 `list_history_sessions(&self, project_id: &str)`
接受任意 `project_id`;`src-tauri/src/commands` 里对应命令也只是透传。
**查任何项目的历史会话,后端本来就支持,数据都在。断的只有界面这一处。**

## 这修的是什么

同一处缺陷同时是两件事的根因:

1. 张先生 2026-10-20 问的「如何在 UI 里唤起其它项目的过往 session」
   —— 功能存在,但入口跟全局选择脱节。
2. 项目选择器与历史面板不一致,造成"明明选了 A,这里显示 B"。

## 修的判据(验收用)

- 打开历史会话面板时,**默认选中全局当前项目**,不是自己那份初始值
- 切换左上角项目选择器,历史面板跟着重载
- 面板里的项目列表仍可点(可以看别的项目),但**必须显示它与全局选择不同**,
  不能让人以为全局也切了 —— 这正是 Lyra 说的"一个入口一个含义"
- 后端不动;不动实时镜像那条路径

## 状态

**待派。** 张先生尚未确认派单;按治理流程由 Aegis 派给界面席位(Mira)。
注意:Mira 手上已有 V5 编造数字修复一单在飞,这一单需等交付或并行。
