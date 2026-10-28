# 开卡:两件已验绿的活

| Field | Value |
|---|---|
| id | AEGIS-2026-10-22-nimbus-open-cards-v1 |
| target_seat | seat-nimbus-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-22 |
| 前提 | 两件都已验绿、分支已同步,只差卡 |

## 先说状态:你的活已经站住了

| 活 | 实现单 | 验绿 |
|---|---|---|
| 删除豁免清单 | `disp-5WQ7UeLjxhEd6j8PsuOxL` | **pass** |
| 操作者待决对象 | `disp-dwSmYkULRU4HdSDJCDWH9` | **pass** |

Aegis 逐条核过的:
- 89/89 测试过(含既有防护测试,没被改断言)
- 分支已同步:`bfe5299`(合并 track)在,落后的 1 个是 Aegis 自己的文档提交,
  且那区间内**没有任何文件落在 `ui/src` 或 `src-tauri`** —— 按新判据是干净的

## 为什么必须你自己开

`request_acceptance` 只允许**完成派单的那个席位**调。Aegis 不是干这活的人,开不了。

这不是流程障碍,是设计:**卡由干活的人开,他得对卡上写的验收条件负责。**
Aegis 替你开,就等于 Aegis 替你说"这活验过了"。

## 开两张,分开开

### 卡一:删除豁免清单

- `dispatch_id`: `disp-5WQ7UeLjxhEd6j8PsuOxL`
- `flux_verdict_ref`: 那一单的验绿事件 id(pass 那条,**不是**重复记账那条;
  有两 pass,以挂在现役派单上且确认了正确分支那条为准)
- `how_to_verify`: 要能回答"这防护有没有被削弱" —— 建议写成
  "构造一个不在清单上的删除,断言仍被拒"(这正是你已有测试做的事)
- `acceptance_plan`: steps + expected 都要有,缺了工具会拒

### 卡二:操作者待决对象

- `dispatch_id`: `disp-dwSmYkULRU4HdSDJCDWH9`
- `flux_verdict_ref`: 那一单的验绿事件 id
- `how_to_verify`: 要能回答"只有他能定的事现在能不能被结构化的问出来"
- `acceptance_plan`: 同上

## 一个必须说清的限制(会打折扣,但必须说)

**待决对象那一件,张先生在界面上现在还看不到。**

因为:对象和 MCP 工具做出来了,但**界面那一层还没接**(派单里我写的是
"界面要有地方看,具体哪一屏你定"—— 那是你的事,但你交付时我核到的是后端)。

所以卡上要如实写:**后端与工具已就绪,界面未接**。
不要写"已经能用"—— 张先生点开应用找不到入口,那比不告诉他更糟。

(如果界面其实已经接上了,把接在哪一屏写进卡里,我核漏了我改。)

## 验收

- 两张卡各自 `dispatch_id` 与 `branch` 对得上
- 每张的 `flux_verdict_ref` 点名**它自己**的实现单,不许互借
- `how_to_verify` 是"做什么、看到什么",不许写空话
- 待决对象那张如实标注界面是否已接
- 不许为了好过把验收条件写含糊

## 明确不做

- 不合并(张先生点)
- 不改代码 —— 两件都已验绿
- 不把两件合成一张大卡
- 截图 token 修复单(`disp-CJsvktofZ7hkdsESlJQ3D`)在飞,**别把它并进这两张卡**
