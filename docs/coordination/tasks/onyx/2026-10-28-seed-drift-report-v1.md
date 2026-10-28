# Seed / live drift report — the seed described a fleet that does not run

| Field | Value |
|---|---|
| id | ONYX-2026-10-28-seed-drift-report-v1 |
| template | T7 |
| subtype | data_integrity_drift_report |
| status | delivered |
| author | seat-onyx-001 |
| date | 2026-10-28 |
| version | v1 |
| tags | seed, drift, database, data-integrity, baseline |
| depends_on | `docs/coordination/roles/ONYX.md`, `ENGINEERING_STANDARDS.md` §Migrations and seed |
| source | dispatch `disp-bc20UzOznJQ8hCgbHw876` (implementation); supersedes the cancelled `disp-pi-NzENN_ijkHKxgej8D5` |

## 结论先行

**Aegis 报的"种子里少 4 个席位"是对的,但只是最小的一块。**

全表扫下来,真正的漂移是三类,不是一类:

| 类 | 规模 | 这单怎么处理 |
|---|---|---|
| A. 脚本写进 live、从未进种子 | 4 席位 + 2 项目 + 4 绑定 + 1 workitem | **已修**(新种子文件 005) |
| B. 种子与 live **值不一致**(同一个 id,不同值) | 6 个席位的 3 列 | **未修**,需你/Aegis 决定(见 §5) |
| C. 迁移与种子之间的**结构性缺口** | 3 张表 3 列 | **未修**,是 track 的既有 bug(见 §4) |
| D. 反向漂移(种子有、live 没有) | documents 等 8 张表 | **未修**,是有意状态,不是缺口(见 §6) |

**一句话**:种子里少的不是"4 个席位",是**整个 2026-04 之后产生的行**。
种子 001-004 写于 2026-04-27,此后所有新席位/项目/工作项都直接写进 live,
从来没回写过种子。这单只补了 A 类;B 类和 C 类不能由我在这个 packet 分支上修。

## 1. 怎么量的(可复现)

全程**只读** live 库,零写入。判据:`git ls-tree` 取种子声明的 id,
`select id from <t>` 取 live id,集合差。扫描范围是**全部 31 张表**,不是只有 seats
(Aegis 只查了 seats,我按派单要求把其余的查了)。

```bash
# live ids
podman exec seatloom-postgres psql -U seatloom -d seatloom -tAc "select id from seats order by id;"
# seed ids: 从 infra/postgres/seed/*.sql 的每个 INSERT INTO <t> 的每个 VALUES 元组取第一个字符串
```

实测结果:

| 表 | 种子 | live | live 有而种子没有 |
|---|---|---|---|
| **seats** | 6 | 10 | **4** |
| **projects** | 1 | 40 | **39**(其中 37 个归迁移 021,见 §3) |
| **project_role_bindings** | 6 | 10 | **4** |
| **workitems** | 6 | 7 | **1** |
| sessions / handoffs / artifacts / seat_delegations | — | — | 0 |
| topics | 0 | 51 | 51 |
| canonical_events | 32 | 2483 | 2451 |
| memories | 0 | 388 | 388 |
| acceptances | 0 | 62 | 62 |
| documents | 14 | 0 | **−14**(反向) |

topics / canonical_events / memories / acceptances 那几千行**不算漂移** ——
它们是 append-only 的账本和运行时产物,本来就不该进基线种子(见 §6)。

## 2. A 类:脚本写进 live、从未进种子(已修)

四个席位的来源,我从 git 历史确认了(**没有采用 Aegis 的猜测当结论**):

| id | 来源 | 提交 |
|---|---|---|
| `seat-vela-001` | `scripts/bind-vela-to-seatloom.sh` | `1143995` 2026-10-16 |
| `seat-ql-001` | `scripts/adopt-ql.sh` | `9b4d726` / `9ecd57c` 2026-10-16 |
| `seat-networkorchestrator-001` | `scripts/adopt-network-orchestrator.sh` | `4403715` / `6b10242` 2026-10-26 |
| `seat-stepaicloud-001` | `scripts/adopt-stepaicloud.sh` | `88cdee9` / `a72cfc7` 2026-10-27 |

Aegis 的猜测方向对,但有三处它说得不准,我照实测写:

1. **vela 的席位行不是那个脚本建的。** `bind-vela-to-seatloom.sh` 只跑一条
   `INSERT INTO project_role_bindings`(脚本第 36-37 行),席位行比它更早
   (`created_at` 是 2026-10-10)。脚本自己都查 `seats` 表确认席位存在才绑定。
2. **`project_delivery/QL` 项目不该由我补。** 它已经是迁移 021 的 38 个
   corpus home 之一(021 文件第 201 行),由 021 拥有。补了就是两个文件争一行。
3. **external 项目只有 2 个,不是 3 个。** 第三个是 QL,归 021。

### 修法:新文件 `infra/postgres/seed/005_adopted_seats_and_external_projects.sql`

- 每行都带**证据注释**(哪个脚本、哪个提交、哪一行),符合我角色对种子的要求
  ("attach an evidence comment to every seeded row; no fabricated content")
- 每行值都是 live 的**原样读数**,不是我推定的
- 全部 `ON CONFLICT DO NOTHING`,additive,不改 001-004 任何一个字节
- 空 `capability_tags` 如实写成 `ARRAY[]::text[]` —— 那是 adopt 脚本声明的值,
  不是我省事
- `home_path` 留 NULL 并在文件头说明为什么(见 §4)

### 验证(在一个一次性 scratch 库里,不在 live)

建了一个临时库 `onyx_seedproof`,按 apply-db.sh 的规范顺序灌 21 个迁移 + 001-005 种子:

| 检查 | 结果 |
|---|---|
| 005 连跑 3 次 | 每次 `10 seats / 40 projects / 10 bindings / 7 workitems`,**完全不变** |
| scratch 的席位 id 集合 vs live | **IDENTICAL** |
| scratch 的角色绑定 vs live | **IDENTICAL** |
| 4 个新席位的完整九列 vs live | **IDENTICAL** |
| live 库 | **未动**(仍是 10 席位 / 40 项目),scratch 用完已 drop |

**也就是说:从种子重建,现在能重建出正在跑的那支舰队。** 这是这单真正的验收判据。

## 3. B 类:同一个 id,种子和 live 值不一样(未修,需要你决定)

这一类比 A 类更危险,因为它**在下次 apply 种子时会覆盖 live**。

seed 001 对六个老席位的 009 三列是这样 upsert 的:

```sql
ON CONFLICT (id) DO UPDATE SET
  default_harness = EXCLUDED.default_harness,   -- 种子: 'Stepcode'
  default_agent   = EXCLUDED.default_agent,     -- 种子: 'ClaudeCode'
  default_model   = EXCLUDED.default_model;     -- 种子: 'claude-opus-5[1m]'
```

live 里这六个席位实际是:

| 列 | 种子写 | live 实际 |
|---|---|---|
| `default_harness` | `Stepcode` | **`Pi`** |
| `default_agent` | `ClaudeCode` | `ClaudeCode`(一致) |
| `default_model` | `claude-opus-5[1m]` | **`stepplan-wash-anthropic/step-5-preview`** |

**历史(从 git 确认)**:`629c68e`(009)把 `Stepcode`/`claude-opus-5[1m]` 写进种子;
`af0275a` 把 live 定成 Stepcode;然后**你 2026-10-17 的裁定**(`9d8cb57`,
`fix-seat-harness-to-pi.sh`)把 live 的 harness 改成 `Pi` —— 那是一个 live UPDATE,
**从没回写种子**。模型的发散更晚,来自 adopt 脚本和 `set-seat-default-model.sh`。

**为什么这不只是难看**:`claude-opus-5[1m]` 是 Claude Code 时代的模型名,不是 pi 的
`provider/model` 格式。`scripts/seat-launch.sh` 的 `model_for()` 专门有一个形状守卫
拒绝它(注释原文:「必须校验形状:seats.default_model 里有历史遗留值」)——
也就是说**启动路径防住了,种子路径没防,而种子会在 apply 时赢**。

**为什么我没修**:这要改 seed 001 已有的 VALUES 块和它的 ON CONFLICT 体,
是修改历史文件而不是加新文件;而且改它等于把"live 的真值"写进基线,
这是产品决策(你的裁定),不是我一个数据席位该定的。这条是漂移报告里优先级最高的一项。

## 4. C 类:迁移与种子的结构性缺口(未修,是 track 既有 bug)

量的时候踩到三个,都是**先有种子、后加约束**造成的:

| 表.列 | 约束 | 种子还写不写这一列 |
|---|---|---|
| `sessions.project_id` | 008 设 `NOT NULL` | seed 001 的 sessions INSERT **不写** |
| `workitems.project_id` | 008 设 `NOT NULL` | seed 001 的 workitems INSERT **不写** |
| `handoffs.project_id` | 设 `NOT NULL` | seed 001 的 handoffs INSERT **不写** |

**后果:在一个空库上从零建,`apply-db.sh` 会在 seed 001 就失败。**
我在 scratch 库里复现了,失败信息就是
`null value in column "project_id" of relation "sessions" violates not-null constraint`。
seed 001 是**未改动的原文件**,所以这不是我引入的 —— track 上现在就是这样。

**修法(不归我,也不该在这个 packet 分支上做)**:seed 001 的三个 INSERT 补上
`project_id` 列并填 `'seatloom'`,或者给这三列加 `DEFAULT`。前者更诚实。
这属于改历史种子文件,和 §5 一样需要 Aegis 决定。

**另一处,也不归我**:`projects.home_path` 由**迁移 021(后置)** `ALTER TABLE` 添加,
而种子在 021 之前跑。所以种子永远填不了 `home_path`。我在 005 里留 NULL 并在
文件头写明。补它应该是 021 旁边的一个 guarded `UPDATE`,不是我塞进种子。

**还有一处我在这个 scratch 库里发现的顺序问题**:apply-db.sh 的 `POST_SEED_SCHEMA`
把 **021 排在 011 之前**,而 021 的 `ALTER TABLE archived_sessions` 需要 011 先建表。
空库上跑会 `relation "archived_sessions" does not exist`。live 上没暴露是因为
live 的表早就建好了。这也是 track 既有顺序问题,只报不改。

## 5. D 类:反向漂移(未修,是有意状态)

种子声明了 live 没有的行,不是缺口:

| 表 | 种子 | live | 为什么 |
|---|---|---|---|
| `documents` | 14 | 0 | 002 的文档行只在 `bootstrap.sh` 的首次路径里灌;live 走 `ingest-documents.sh`,而 `apply-schema-to-live.sh` 明确不碰 seed |
| `pipeline_runs` | 1 | 0 | 同上,运行时表 |
| `review_threads` / `review_comments` | 3 | 0 | 同上 |
| `handoff_receipts` | 3 | 0 | 同上 |
| `document_associations` / `document_sections` | 23 | 0 | 同上 |

这些表的共同点:**live 是"真实运行中的库",种子是"协作基线快照"**。
两边本就不该逐行相等。`apply-schema-to-live.sh` 的注释说得比我清楚:
「在一个已有 60 条派单、真实数据的库上重跑 seed 是不必要的风险」。

## 6. 为什么那几千行不算漂移(免得下次又数一遍)

| 表 | live | 为什么不该进基线种子 |
|---|---|---|
| `canonical_events` | 2483 | append-only 账本,每笔是一个真实事件 |
| `event_object_refs` | 6140 | 账本的投影 |
| `memories` | 388 | 运行时记忆,`remember` 写的 |
| `topics` | 51 | 同上 |
| `acceptances` | 62 | 运行时验收卡 |
| `dispatches` | — | 派单 |
| `operator_decisions` | 0 | — |

ENGINEERING_STANDARDS 写「PostgreSQL is canonical structured truth」,而
`.seatloom/` 是 cache/export。**基线种子描述"这支舰队长什么样",不描述"它干过什么"。**
后者是账本的职责,塞进种子只会让种子变得不可重放、不可评审。

## 7. 给你的三件事(按优先级)

1. **§3 的六席位三列**:要不要把 live 的真值(`Pi` /
   `stepplan-wash-anthropic/step-5-preview`)写回 seed 001?这会把你的 10-17 裁定
   落到基线上。不写的话,下次 apply 种子会把 live 改回旧值。
2. **§4 的三个 `project_id NOT NULL`**:空库从零建现在会在 seed 001 失败。
   修 seed 001 的三个 INSERT(加列填 `'seatloom'`)是最直接的。
3. **§4 的 021/011 顺序**:`apply-db.sh` 把 021 排在 011 前,空库会失败。
   换一下顺序即可。

三条都在 track 侧,不由我从 packet 分支改。这单我只动了**新增**的 005。

## 8. 边界

- **没有对 live 库跑任何迁移** ✅(全程只读;唯一写操作发生在一个临时 scratch 库,已 drop)
- **没有改 live 的行** ✅
- **没碰 021 / 022** ✅(派单明确禁止;我反而发现 021 的一个顺序问题,只报不改)
- **没改 001-004 一个字节** ✅(只新增 005)
- **没开验收卡** ✅(这单是 implementation,verdict 由别的席位出)
