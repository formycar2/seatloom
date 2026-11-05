# 🔴 数据丢失事件报告 — live 数据库 seatloom 被清空


| Field | Value |
|---|---|
| template | T3 |
| subtype | task_delivery |
| id | AEGIS-2026-10-31-DATA-LOSS-INCIDENT |
| status | delivered |
| author | aegis |
| date | 2026-10-31 |
| version | v1 |
| tags | seatloom |
**时间**:2026-10-31 09:00 心跳发现
**最后成功写入**:2026-10-31 00:11:52 UTC
**发现时状态**:`seatloom` 库里 `public` schema **0 张表**

## 丢了什么

账本上到 00:11 为止的全部记录:

- `canonical_events`(事件账本)—— 约 2900 条
- `memories` —— 452 条
- `dispatches`(派单)—— 约 400 件
- `acceptances`(验收卡)—— **68 张已接受 / 9 张取消 / 5 张待验收**
- `topics` —— 51 条
- `seats` / `projects` / `project_role_bindings` / `workitems` / `sessions` / `handoffs` / `artifacts` / 等 32+ 张表

**5 张待验收卡全部消失**,包括你已经可以点的那几张。

## 我查到的(都是实测,不是推测)

1. **00:11:52 还有成功写入**(`INSERT INTO dispatches`)。
2. **00:34:10 第一次报 `relation "seats" does not exist`**。
3. 所以清空发生在这 **22 分钟**内。
4. **这 22 分钟的 postgres 日志是空的** —— 连每 4 秒一次的席位心跳都没有。
   `log_min_messages=warning`,DDL 会被记录,而窗口内**没有任何 DROP/DELETE/TRUNCATE**。
5. 00:34:40 有人跑 `ALTER TABLE projects ADD COLUMN IF NOT EXISTS home_path` ——
   那是**对着一个已经空了的库**做重建,不是清空动作本身。
6. 00:34:12 有人跑 `drop database if exists lyra_probe; create database lyra_probe`
   并报错 `DROP DATABASE cannot run inside a transaction block`。**不是它。**
7. 我查了全部脚本:只有 `build-sandbox-db.sh` 和 `refresh-sandbox-db.sh` 含 DROP,
   而它们只动 `seatloom_sandbox`,且要显式 `--drop`。**不是它们。**
8. 容器 10-27 11:38 之后没重启过,`podman ps` 显示 Up 3 days。
9. 6 个 WAL 段里,第 6、7 段的 mtime 是 **00:34** —— 那时有一次大写入,
   但日志里对应的只有那些 SELECT 和 ALTER。

## 恢复来源:没有好的

| 库 | canonical_events | 时间 | 可用? |
|---|---|---|---|
| `seatloom`(live) | **0 张表** | — | 没了 |
| `seatloom_sandbox` | 1535 | **10-21**,18 张验收卡 | 差 10 天,不可用 |
| `seatloom_tclose2` | 36 | 10-30 | 只有测试数据 |
| 其他 40 个 scratch | <100 | 各自用途 | 不可用 |

**没有任何一个库含有 10-21 之后的账本快照。** 也没有 `pg_dump` 备份
(我找过 `~/seatloom-db-backups/` 和仓库里的 `*.dump` / `*.sql.gz`,都没有)。

## 我现在能做的和不能做的

**不能**:重建 schema、灌数据、改任何账本行 —— 那是被权限封顶的动作,
和往常一样,我不绕。

**能,而且已经写好脚本等你跑**:仓库里有 `scripts/apply-schema-to-live.sh`
(它不删任何东西,只建缺的表)。跑它能把 32 张表建回来,
但**表是空的** —— 那 452 条记忆、68 张已接受的卡、约 400 件派单,
**没有任何地方还有它们。**

## 需要你定三件事

**一、这 22 分钟里,你或别的进程对这台机器上的 PostgreSQL 做过什么?**
日志是空的,所以我只能靠你回忆。特别是有没有跑过什么涉及
`seatloom` 这个库名的命令(哪怕是别处的脚本、别的项目的工具、
或者一次 `podman` 操作)。

**二、你有没有这台机器之外的备份?**
比如 gitlab/github 上的导出、另一台机器的副本、或者某个时点的手工 dump。
如果没有,那 10 天的账本就此消失,我们需要决定怎么往下走。

**三、schema 要不要我先建回来?**
```bash
bash scripts/apply-schema-to-live.sh
```
它默认空跑,会先告诉你它要建什么。建回来之后表是空的,
席位会全部从零开始(第一次派单、第一次记忆、第一次验收)。
**已接受的 68 张卡不会回来,但代码都在 git 上,一份没丢。**
丢的是「谁在什么时候接受了什么」这个记录,不是代码。

## 一条我要说的

`fleet-status.sh` 这一轮**没有把这件事显示成"没有需要你动手的事"** ——
它显示成「⚠️ 读不到裁定清单(operator_decisions 表还没建)」和
「❌ 未上线(memories 表还没建到数据库里)」。

它说的是真话,这是它对的地方。但它说的是「还没建」,
而事实是「建过,然后被清空了」。**这两件事对他人的含义完全不同** ——
前者是"还没开始",后者是"丢了东西"。

我在心跳协议里没有一条规则覆盖"live 库整个消失"这种情况。
这是我这周学到的最大的一个空洞:我所有的检查都在问
「迁移登记了吗」「卡点得动吗」「verdict 指对了吗」,
**没有一个在问「账本还在吗」。** 一个不存在的表和一个空的表,
在我的检查里长得一样。
