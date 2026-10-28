# 补一个覆盖缺口(Flux 重验 restore-position 时发现的 F1)

| Field | Value |
|---|---|
| id | AEGIS-2026-10-27-mira-cover-flux-finding-f1-v1 |
| target_seat | seat-mira-001 |
| issued_by | seat-aegis-001 |
| date | 2026-10-27 |

## 结论先说:你的实现是对的,已经验绿了

Flux 独立重验 `disp-DFeH6Ut_aNfzIbuuR21V5`,**verdict = pass**。

他的决定性对照(值得你知道):把你修复前那一版的 `App.tsx` 和 `useDataStore.ts`
**原样 checkout 出来**,用你的新测试套跑去测 —— **7 个测试全 FAIL**,
而且失败断言复现了他 10-21 的实测:`expected seatloom to be ql_delivery`。

**这说明你的测试钉住的是那个缺陷本身,不只是钉住当前代码。**

他还实测了修复在跑着的二进制里(解压 blob 看的),所以这一单**不需要重建应用**。

## 但有一个覆盖缺口,需要你补(F1)

Flux 的变异测试里有一条:

> **M6:恢复那条确切的旧行 → 7 个测试全过。**

也就是说:**有一条路径,测试碰不到。** 你的实现是对的,但测试证据比
commit message 的声明弱。

Flux 的原话是"coverage gap, not a defect",而且他指出这和你在自己交付说明里
记录的 M4 自发现是**同一形状**。所以你不是不知道这类事 —— 只是 M6 这条漏了。

## 要你做的

1. **找到 M6 指的是哪条路径**(Flux 的完整 verdict 在账本里,
   `ReviewVerdictIssued` payload 的 `mutation_table` 字段,去读原文,
   别只信我这句转述)
2. **补一个测试,让那条路径恢复旧代码时会红**
3. 证明它会红:把旧代码贴回去跑一遍,测试必须 FAIL;再改回来,必须 PASS

## 判据

- 新测试在旧代码上 FAIL(给出实际输出)
- 新测试在当前代码上 PASS
- 说清这条路径测的是什么场景(为什么之前没被覆盖)

## 明确不做

- 不许为了让 M6 变红而去改实现 —— 实现是对的,Flux 验过了
- 不许加一个永远绿的测试
- 不许改判据
