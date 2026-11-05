# Aegis 18:00 心跳交付：席位绑定仍未修；你提交的预算文档让文档关卡变红

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | AEGIS-2026-11-02-1800-heartbeat-binding-still-open-and-doc-gate-red |
| status | delivered |
| author | aegis |
| date | 2026-11-02 |
| version | v1 |
| tags | heartbeat, defect, docs-gate, seats |

## 变了什么

本轮初始 `3e3c8bc349511d4b0652bd5dd62b5c151be66291`（你 17:50 提交了预算池文档），
分支 `track/infra-foundation`；已全文读取 Aegis 心跳协议并逐项执行。

**先说一件我差点报错的事。** 我跑 `verify-docs.sh 3e3c8bc`（把当前 tip 当基准），
它输出 `all conform`。我据此准备说「关卡是绿的」。**然后我核了一遍**：
那份文档一个模板头字段都没有，`head` 一眼可见。改成跑默认路径（CI 实际走的那条），
同一份文档立刻被抓出来：

```
✗ docs/coordination/knowledge/BUDGET_POOL_MODEL_MAP.md:
  missing universal-header field(s): template subtype id status author date version tags (§2)
```

原因：把 tip 当基准时，那份文档在 `tip..HEAD` 里是「已存在」而不是「新增」，所以不在检查集内。
**这是我用错基准造成的假绿，不是关卡缺陷。** 我把它记下来，因为「用一个不会失败的基准去跑检查」
和「检查本身是坏的」看起来一样，而前者是我自己造的。

## 检查结果（18:00，数据库自己的时间）

- `check-ledger-exists.sh` 退出 0：35 表、1 事件。
- 自醒 18:00 自然 POKED；`--check` 退出 0，原本/副本/协议路径一致。
- `check-salvage-fresh.sh` 退出 0：两份有效补丁匹配，未提交工作有着落。
- `check-cards-clickable.sh` / `check-verdict-refs.sh` / `check-verdict-defects.sh` /
  `check-card-verifications.sh` 退出 0 —— 但 **pending 卡是 0，样本为空**。
- 席位存活：9 个席位会话的子进程都是 saih（按父子关系核）。
- 队列、48 小时内 done 无卡、pass verdict 无卡、pending 裁定：全部 0 行。
- **没有陈旧 `in_flight`，没有派给我的活。**

## 两件未解决的事（都不是我造成的，但我没修）

### 1. 席位绑定仍未修 —— 你在 UI 里还是只看到 vela

活库实测（`now() = 2026-11-02 10:00:28+00`）：

```
project_role_bindings WHERE project_id='seatloom'  →  只有 seat-vela-001
seats WHERE id='seat-aegis-001'  →  0 行
```

上轮我交付的 `scripts/bind-seatloom-seats.sh` 已验过（沙箱库双向测试、幂等、负向测试），
但它**没有被执行**，所以缺陷原样存在。它是改 live 账本的写操作，我被权限封顶。

```
bash scripts/bind-seatloom-seats.sh --apply
```

跑完 SeatLoom 项目下应该出现 7 个席位，含 aegis。

### 2. 你 17:50 提交的预算文档让文档关卡变红

`3e3c8bc` 把 `docs/coordination/knowledge/BUDGET_POOL_MODEL_MAP.md` 提交上了 track，
它没有模板头（`DOCUMENT_TEMPLATES` §2 要求 8 个字段）。默认路径下关卡报 1 条违规。
这不是内容问题——内容我读过了，你已把金额口径从「余额」修正为「已花费」——
纯粹是缺 8 行表头。

**我没有改你的文档。** 它是你的文件、你的提交。两个选项，都要你点头：

- 我补上那 8 个模板头字段（只加表头，不动你一个字内容），或
- 把它加进 `.gitlab/docs-grandfather.txt`（承认是债，不清偿）。

## 你会看到什么 / 需要你做什么

- 自醒、账本、席位在线、唤醒服务都正常；应用验收门可点但 0 张卡。
- **两件等你**：跑绑定脚本；决定预算文档的模板头怎么处理。
- 本会话无 SeatLoom MCP 工具，未写 remember / 未发事件 / 未完结账本派单。

## 自报

上轮我说「绑定脚本已交付、待执行」——现在它仍未执行，缺陷仍在。
我上轮还说了「文档关卡唯一违规是你那份预算文档」——**那句在我用错基准时不成立**，
我是复核后才敢写的。这次没有把「我跑出来是绿」直接当成绿。
