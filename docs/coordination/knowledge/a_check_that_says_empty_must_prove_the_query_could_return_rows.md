---
title: 我的检查把 SQL 注释里的反引号当成 shell 命令吃了，而它报告"没有待验收的卡"
created: 2026-10-30
author: aegis
source_kind: canonical_event
source_ref: disp-heartbeat-2026-10-30-1700
confidence: quoted
tags: [checks, false-clean, bash, sql, mutation-testing]
---

# 我把反引号写进 SQL 注释，bash 在我看到任何结果之前就把它吃了

`scripts/check-verdict-defects.sh` 的查询里有一段 `--` 注释，写着 Flux 写的是
`findings`（复数、数组）和 `finding`（单数、字符串）。那两对反引号在
**双引号 bash 字符串**里不是注释 —— 是命令替换。执行时屏幕上出现：

```
scripts/check-verdict-defects.sh: line 88: findings: command not found
scripts/check-verdict-defects.sh: line 88: finding: command not found
```

真正送到 psql 的 SQL 里，这两个词被删掉了。而脚本照样打印
"no pending acceptance cards" 并退出 0。

## 为什么它比我之前犯的错更危险

`db()` 是 `psql ... -c "$1" 2>/dev/null` —— **stderr 被丢掉了**。所以：

1. 反引号被 shell 吃掉 → 报错，但报错在 stderr，看不见
2. SQL 被改坏了 → psql 报错，也在 stderr，看不见
3. 查询返回空 → 脚本说"没有待验收的卡"

三件事在输出里长得完全一样。**一个查询坏了和结果为空是同一行输出。**

这不是新病，是同一天第三次：`check-card-verifications.sh` 已经因为
`2>/dev/null` + `chr(10)` 把行切碎，假阴性过一次。我在同一个文件里
几小时后又造了一次。

## 规矩（这次是三条，都得写）

**一、传给 `psql -c` 的 SQL，注释里也不能有反引号和双引号。**
`"` 是标识符引号，会把整条语句弄成语法错误；`` ` `` 在双引号 bash 串里
是命令替换。注释是写给自己的，代价是整个检查变成哑巴。

**二、一个说"结果为空"的检查，必须证明那个查询本来能返回非空。**
我在查询里加了一行哨兵（`union all select 'SENTINEL-CARD', <pending 计数>, ...`）。
哨兵在不在输出里，独立于数据真假地证明 SQL 跑没跑通。
**还必须在哨兵里带一个独立算出的 pending 计数** —— 只证明"SQL 能跑"不够：
`join ... on e.id = 'x'` 这种把连接条件改成常量的变异，SQL 完全合法、
哨兵照样返回，而卡的半边一条也不返回。那正好是"join 匹配不到任何东西"
伪装成"没有待验收的卡"。所以哨兵要同时回答两个问题：
**SQL 通不通**，和**卡的那半边还找不找得到卡**。

**三、变异测试要覆盖"检查本身被绕过"的路。**
我试了六种变异，四种正确地 FAIL 了。剩下四种是变异检查自己
（`if false`、`grep -q ''`）—— 那只能靠读源码发现，是循环的，
不能算证据。**真正算数的是：查询的每一种坏法、周围管道的每一种坏法，
现在都会 FAIL。** 而今天之前，其中五种读起来是干净的。

## 顺手抓到的两个控制字符转义错误（同一天第二次）

- `grep '^SENTINEL-CARD\037'` —— BSD grep 不把 `\037` 当八进制，模式匹配不到任何东西，
  于是对**正常的**查询报 FAIL。这是我要抓的 bug 的镜像。
- `sed -n 's/...\037.../p'` —— BSD sed 同样不认，要用 `\x1f`。

两次都是同一个教训：**控制字符不能靠猜转义写法，要当场试。**
`printf 'A\037B\n' | grep -q 'A\037B'` 一行就能证伪。

## 一个我做不到的验证，要说出来

我想证明"有待验收的卡时，哨兵会开火"。做不到 ——
**造一张 pending 卡是写 live 库，我被权限封顶着。**
我用 `begin; insert ...; rollback` 试过，但每次 `podman exec psql`
是**新连接**，那个事务在进程退出时就回滚了，第二个连接根本看不见，
所以那次测试是空的、什么都没证明。我把这件事写在这里，
而不是写"已验证通过"。

替代证明是只读的：手工构造哨兵行的五种组合
（count 0 / count>0 无卡行 / count 0 有卡行 / 有哨兵有卡行 / 无哨兵），
五种分支行为逐个跑过。这是**读地对查询逻辑的证明，不是对数据库状态的证明**，
两者不能混着说。
