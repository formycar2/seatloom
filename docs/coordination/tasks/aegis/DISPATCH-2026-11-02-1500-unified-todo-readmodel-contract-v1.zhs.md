# 派单:把 dsh 架构评审的第一项落地成可执行契约

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | AEGIS-2026-11-02-1500-unified-todo-readmodel-contract |
| status | dispatched |
| author | aegis-dispatch |
| recipient | aegis |
| date | 2026-11-02 |
| version | v1 |
| depends_on | AEGIS-2026-11-02-dsh-architecture-review-delivery-v1.zhs |
| tags | architecture, contract, readmodel, dispatch |

## 背景

你昨天交付了 `docs/coordination/tasks/aegis/AEGIS-2026-11-02-dsh-architecture-review-delivery-v1.zhs.md`。
那份交付我完整读过了,质量可以。特别是这几点我接受你的判断,不接受我原来的草稿:

- **不加可写 stage 列** —— 它制造第二事实源,默认 working 把未知伪装成运行
- **不另造总评估器** —— `intent/plan.rs` 已经是确定性规划器,应补授权判定而非覆盖
- **先做项目级待办联合列表** —— 不等同于已有显式关联,禁止按标题相似猜关联

你在 §7 写的下一步是:「先由 Lyra/Nimbus/Mira/Flux 按第一项形成最小实现与验收契约」。

**本轮派给你:把第一项从「架构建议」变成「能直接开工的派单契约」。**

## 本轮要交付什么

**一份可以直接派给席位的派单规格,而不是你自己实现。**

具体三件:

### 1. 第一项的验收契约(最重要)

你在 §5 列了 8 条「必须能失败的验收情形」。把它们变成**可执行的测试用例描述**:
每条写清「给什么输入 / 期望什么 / 什么情况下必须失败」。

你列的第 1 条现在是这样:
> 同一份过滤条件下,待办 count 与完整结果一致;跨项目同名任务不串单;超过一页不会少算。

我要的是:这句话拆成几个独立断言,每个断言有明确的构造数据和失败条件。
**一个测试如果无论实现对错都通过,等于没有测试。**

### 2. 共享读模型的接口契约

你要 Nimbus 做「统一查询与 DTO 组装」。给他之前,先定:

- 这个查询的输入是什么(操作者?项目?两者都要?)
- 返回的每个 obligation 必须带哪些字段(你说了 kind/id/project/actor)
- **计数和列表是否来自同一次快照** —— 你强调过不能拿截断后的列表长度充当全量数字,那契约里怎么强制?
- 分页时 total 从哪来

### 3. 派单拆分

第一项要 Lyra/Nimbus/Mira/Flux 四方协作。**拆成几张派单,每张的依赖关系和验收人是谁。**

你在 §5 估了 3-5 日。如果拆完发现某张可以独立完成,标出来 —— 那样能并行。

## 两条硬约束

**账本现在是毁坏状态。** `dispatches` 0 行,`canonical_events` 只剩 1 行(那条事故报告)。
Lyra 10-31 对错库跑了 `drop schema public cascade`。

所以:

1. **本轮不许你补造账本数据。** 昨天的派单记录没了就没了,不要为了闭环去重建。
2. **本轮产出只写文件,不进账本。** 等你把契约交出来,我们再谈怎么派。

## 关于昨天的完结

你说过「报告交付与账本完结分开,后者未完成」—— 因为当时 MCP 没连上。
现在 MCP 是通的(28 工具)。但那个派单的行已经不存在了,**所以 complete_dispatch 无从调用**。

**这件事我不追。** 你的交付文档本身是完整的,文件在仓库里,那才是事实。

## 输出位置

`docs/coordination/tasks/aegis/AEGIS-2026-11-02-1500-unified-todo-readmodel-contract-v1.zhs.md`

按你惯用的 T3 格式。

## 我不需要你做的

- 不改代码
- 不动 live 库
- 不建表、不写迁移
- 不替其它席位下实现判断

## 完成后

写文件、提交、在回复里告诉我三件事:
1. 拆成了几张派单
2. 哪张能最先开工
3. 你认为这个契约里最可能被实现方做错的一点是什么
