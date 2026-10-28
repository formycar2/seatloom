---
id: a_test_that_reads_its_own_assertion_strings_can_never_fail
title: 一个用 include_str! 读自己文件的测试,断言匹配到的是它自己 —— 永远无法失败
curation: authored
confidence: inferred
status: active
created_at: 2026-10-29
source_kind: canonical_event
source_ref: AEGIS-2026-10-29-hourly-heartbeat
tags: [mutation-testing, source-text-test, remember, dedup, false-proof]
---

# 一个用 `include_str!` 读自己文件的测试,断言匹配到的是它自己

2026-10-29 04:00。Nimbus 修了 `remember` 无去重的缺陷(我上一轮发现的),
proof 脚本 7 点全过,211 个 Rust 测试全过。按惯例我做变异。

我把被测 SQL 里的 `l2_line_start IS NOT DISTINCT FROM $2`
改成了裸的 `l2_line_start = $2`(NULL 安全比较被破坏,而 live 上 3/4 的重复组
正是没有行号的),然后:

```
bash crates/seatloom-mcp-emit/proof/memory_dedup_proof.sh   → ALL POINTS PASS
cargo test -p seatloom-core --lib the_dedup_key              → ok
```

**两个都过了。** 变异体活下来了。

## 为什么 proof 杀不掉它

因为防护是双层的:SQL 预查给出友好错误,**唯一索引在数据库层兜底**,
而索引的 key 是用 `coalesce(...)` 写的,NULL 安全。所以改掉 SQL 那一层,
NULL 行的重复照样被索引挡住。

这不是缺陷,是设计。真正的问题在 Rust 测试。

## 那个测试在检查它自己

`the_dedup_key_is_the_pointer_plus_the_substance_and_not_the_title`
用 `include_str!("repositories.rs")` 读**自己所在的这个文件**,然后:

```rust
for predicate in [
    "coalesce(l2_file_path, '') = coalesce($1, '')",
    "l2_line_start IS NOT DISTINCT FROM $2",
    ...
] {
    assert!(src.contains(predicate), "the dedup key must contain: {predicate}");
}
```

而那四个字符串**就写在测试自己里面**。`src.contains(...)` 匹配到的是
测试代码里那份字面量,不是被测的 SQL。

实测确认:

```
1293:  AND l2_line_start = $2                        ← 被测 SQL(已被我变异)
4520:  "l2_line_start IS NOT DISTINCT FROM $2",      ← 测试自己的字符串
```

`src.contains("l2_line_start IS NOT DISTINCT FROM $2")` 为真,
因为第 4520 行就是它自己。

## 但它不是完全无效

我把 key 的第二个列换成一个无意义的谓词(`AND title = 'TOTALY-DIFFERENT-KEY'`),
这次测试**失败了** —— 死在第 4545 行,那个"title 不许出现在 WHERE 里"的断言。

所以它的实际能力是:**能抓住"title 进了 key",抓不住"NULL 安全比较被削弱"。**
而它给自己的定位是"钉住四个 key 谓词",四条里至少两条是空转的。

## 它自己承认了弱点,但承认得不够

测试注释写得很好:

> A source-text test is weaker than a behaviour test and is labelled as such.
> It is here because the alternative is nothing, and "nothing" is what let
> the write path ship with no dedup at all for twelve days.

这个判断对,而且它确实比什么都没有强。但它**没有意识到自己匹配的是自己的文本**,
所以它以为自己在守四条,实际守得住一条半。

**一个知道自己弱的检查,仍然可能比自己以为的更弱。**

## 判据

**`include_str!` + `contains` 的测试,匹配范围必须排除测试自身。**

可行的做法:把要守的谓词从被测代码里抽成具名常量,测试引用常量本身
(那样类型系统会拦住改名),或者让 proof 脚本对每个谓词各做一次反向验证
(把谓词改坏,proof 必须失败)。

我用的后者已经证明了它有效:proof 的 POINT 2 在我改坏 SQL 时没失败,
但改坏**索引**时 —— 我也试了,也没失败,因为 SQL 那层还在挡。
**双层防护意味着任何单层变异都可能被另一层挡住**,所以验一个防护,
要把另一层也变异掉,或者直接在数据库层确认是哪一层挡的。
