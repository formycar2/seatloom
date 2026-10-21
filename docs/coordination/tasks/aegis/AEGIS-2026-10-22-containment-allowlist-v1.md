# 给合并防护加显式豁免清单

| Field | Value |
|---|---|
| id | AEGIS-2026-10-22-containment-allowlist-v1 |
| target_seat | seat-nimbus-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-22 |
| 张先生裁定 | 2026-10-21 选 A:「给防护加一个显式豁免清单」 |

## 背景:为什么需要它

`crates/seatloom-mcp-emit/src/dispatch.rs` 的 containment 防护拒绝
「合并会删掉被主干跟踪的文件」的分支。它是用真实事故换来的:

```rust
/// `request_acceptance` (containment defence): the merge would DELETE files
/// tracked on the base — the failure mode that let an Accept destroy
/// already-merged work.
MergeWouldDelete { branch: String, base: String, paths: Vec<String> },
```

注释里那句是关键。**这道防护不能削弱,只能加一个可审计的口子。**

### 眼前这一例

`ui/src/app-v2/AppV2.tsx` 被 Mira 在派单 `disp-DFeH6Ut_aNfzIbuuR21V5` 第 3b 项
明确删掉(那项就是我写的:「AppV2 的 MOCK_CONTACTS 是死代码,删掉」)。

Aegis 独立核实的五条:
1. 全代码库零 import(grep 只有两处注释在讨论它不存在)
2. Rust 入口无任何路由挂载
3. **在主线上就已不可达** —— 不是删了才失效
4. 合并预览只此一个 `D`,无连带删除
5. 派单第 3b 项明确要求删

张先生 2026-10-21 裁定:**「文件删掉」**,并选择用豁免清单这条路。

## 要做的事

在 dispatch.rs 里加一个**静态清单**,三条硬规矩:

1. **每条必须带:路径、谁同意删的、依据**。只写路径不够 —— 半年后没人记得
   为什么那一行在,就会当成历史包袱绕过它。
2. **清单不可由环境变量或运行时配置注入**。它只能改代码、走审核、留 git 记录。
   一个能由环境变量打开的删除豁免,是把防护变成了可选项。
3. **清单只对 `MergeWouldDelete` 生效**,不影响另一条防护
   (`BranchNotSynced`)。分支未包含主干这件事,没有豁免。

## 建议的形状(你来定最终实现)

```rust
/// Deletions the OPERATOR has explicitly ruled on. Each entry carries the
/// reason and the ruling, because a bare path in a year reads as legacy
/// cruft and someone will route around it.
///
/// This list does NOT make the containment defence optional: it is consumed
/// only by `MergeWouldDelete`. `BranchNotSynced` has no exemption at all.
/// A deletion not on this list is refused, exactly as before.
struct DeletionExemption { path: &'static str, ruled_by: &'static str, why: &'static str }

const DELETION_EXEMPTIONS: &[DeletionExemption] = &[
    DeletionExemption {
        path: "ui/src/app-v2/AppV2.tsx",
        ruled_by: "human:supervisor",
        why: "dead code, deleted by dispatch disp-DFeH6Ut_aNfzIbuuR21V5 item 3b; \
               zero importers, no route, already unreachable on the base",
    },
];
```

行为要求:
- 合并预览算出的删除集合,**减去豁免集合**后若为空 → 不再拒绝
- 减去后**仍非空** → 照旧拒绝,并把剩余的路径全列出来
  (一个清单让三道删通过、第四道仍然被拦,这是对的)

## 验收

1. 豁免路径命中时不再拒绝,且错误信息里**不提**被豁免的路径
2. 未在清单上的删除**仍然被拒**,错误信息列出全部剩余路径
3. `BranchNotSynced` 完全不受影响(未同步的分支照样拒绝)
4. **清单本身要有测试**:故意把一个不在清单上的路径加进删除集,断言仍被拒
   —— 这条测试的意义是防将来有人把清单改成 `*`
5. 现有 `request_acceptance_refuses_unsynced_and_destructive_branches` 测试
   **必须仍然通过**(它是这道防护的既有证明,不许为让它过而改断言)
6. 不许为了让新测试过而把清单开成通配
7. cargo build / test / clippy / fmt 全过

## 明确不做

- 不加环境变量开关
- 不给 `BranchNotSynced` 开任何口子
- 不删 `ui/src/app-v2/AppV2.tsx`(那是合并时的事,不是你的事)
- 不改其它防护的判定
- 不合并、不自己开验收卡

## 交付

`docs/coordination/tasks/nimbus/NIMBUS-2026-10-22-containment-allowlist-v1.md`
写清:清单长相、与 `MergeWouldDelete` 的关系、测试怎么证明它没被削弱。
