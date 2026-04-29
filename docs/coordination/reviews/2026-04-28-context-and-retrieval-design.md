# 跨 Runtime 上下文管理与分层召回设计

| 项目 | 内容 |
|------|------|
| 文档 | 上下文管理与分层召回设计 |
| 状态 | 待 Mr. Zhang 审阅 |
| 作者 | Aegis |
| 日期 | 2026-04-28 |

---

## 1. 问题一：跨 Runtime 切换的上下文重建成本

### 1.1 问题本质

当 Seat 从 Runtime A（如 Claude Code）切换到 Runtime B（如 Codex），当前设计的处理方式是：结束旧 Session → ContextPack Compiler 组装 LaunchPack（8192 token 预算）→ 注入新 Session。

信息流的损失链：

```
Claude Code 中的完整上下文（~80k tokens）
  ↓ Checkpoint 摘要（~500 tokens，损失 95%+）
  ↓ LaunchPack 注入 Codex（~8k tokens）
  ↓ Codex 首轮"重新理解"（~5k tokens 额外消耗）
```

切换隐性成本 ≈ 13k tokens + 重复工作。一天 4 次切换 ≈ 50-80k tokens 浪费。

### 1.2 解决策略

#### 策略 1：分层上下文存储（Context Tiers）

不再将所有上下文压缩为单一 LaunchPack，而是分层存储、按需加载：

| Tier | 内容 | Token 量 | 注入时机 |
|------|------|---------|---------|
| Tier 0: Identity | "你是 nimbus，正在做 WI-012，在 feature/oauth-login 分支" | ~200 | 永远注入 |
| Tier 1: State | AC 完成情况、最近 commit、当前 blocker | ~1000 | 永远注入 |
| Tier 2: Decisions | 已做的技术决策和原因 | ~2000 | 默认注入 |
| Tier 3: Evidence | 错误 stack trace、完整 diff、测试结果 | ~4000 | 按需加载 |
| Tier 4: History | 完整 Session 的结构化摘要（按轮次索引） | 无上限 | 仅在 Seat 主动请求时 |

固定注入成本：~3200 tokens（Tier 0-2），按需获取 Tier 3-4。

#### 策略 2：结构化 Checkpoint

将 Checkpoint 从自由文本摘要改为结构化 YAML：

```yaml
# .seatloom/sessions/ses-005/checkpoint-003.yaml
timestamp: 2026-04-28T16:40:00
workitem: WI-012
branch: feature/oauth-login
last_commit: abc1234

progress:
  ac_completed: ["Google OAuth 登录", "GitHub OAuth 登录"]
  ac_remaining: ["登录后跳转到 /dashboard"]
  
decisions:
  - question: "NextAuth v4 还是 v5?"
    answer: "v5"
    reason: "v5 原生支持 App Router"
    
  - question: "callback 同步还是异步?"
    answer: "异步"
    reason: "避免阻塞 middleware chain"

blockers:
  - file: "src/auth/callback.ts"
    line: 42
    error: "TypeError: Cannot read properties of undefined"
    attempted_fixes: ["添加 null check", "改用 optional chaining"]
    status: unresolved

files_modified:
  - path: src/auth/callback.ts
    insertions: 28
    deletions: 5
  - path: src/auth/providers.ts
    insertions: 14
    deletions: 0

artifacts_produced:
  - AR-015: diff_summary
```

优势：
- 引擎可处理：Pack Engine 精确提取需要的字段，不需要 LLM "理解"自由文本
- 跨 Runtime 无损：结构化数据语义不变
- 可组合：不同 Tier 的 Checkpoint 字段可以灵活组装

#### 策略 3：增量上下文（Delta Context）

Seat 从 A 切换到 B 再切回 A 时，A 只需要"自从暂停以来发生了什么"：

```
Session A (paused) → Session B (active) → Session A (resumed)
                                           │
                                           └── 只注入 Delta:
                                               "在你暂停期间：
                                                - 修复了 callback.ts:42 的 TypeError
                                                - 新增了 test case
                                                - commit: def5678"
```

需要 Ledger 支持"自某个 Checkpoint 以来的事件查询"，Pack Engine 组装 delta。

#### 策略 4：Session 暂停而非终止

```
当前模型：running → completed → 新 Session（全量 rehydrate）
优化模型：running → suspended → resumed（delta inject only）
```

suspended 状态保留完整 Checkpoint，随时恢复。这比"completed + 从头 rehydrate"成本低得多。

### 1.3 量化对比

| 场景 | 当前设计 | 优化后 |
|------|---------|--------|
| 切换一次的注入成本 | ~8192 tokens | ~3200 tokens (Tier 0-2) |
| 切回原 Runtime 的注入成本 | ~8192 tokens（再次全量） | ~800 tokens（delta only） |
| 一天 4 次切换总成本 | ~32k tokens | ~6k tokens |
| 信息保真度 | ~60%（自由文本摘要有损） | ~95%（结构化 Checkpoint） |
| **Token 节省率** | baseline | **~81%** |

---

## 2. 问题二：沉淀内容的分层召回

### 2.1 SeatLoom 沉淀的内容种类

| 内容类型 | 数据量级 | 查询模式 |
|---------|---------|---------|
| Seat Profile / Card | 几十条 | 精确查找（by name/role） |
| WorkItem 状态和 AC | 几十到几百条 | 精确查找 + 过滤 |
| Handoff 记录 | 几十到几百条 | 精确查找 + 关联查询 |
| Ledger 事件 | 几千到几万条 | 时间范围 + 类型过滤 |
| Checkpoint（结构化） | 几百条 | 精确查找（by session + 时间） |
| Artifact 元数据 | 几百条 | 精确查找 + 分类过滤 |
| Artifact 内容 | 几百个文件，MB 级 | 全文搜索 + 语义搜索 |
| Session Transcript | 几十个文件，每个 10k-100k tokens | 语义搜索 |
| Playbook | 几十到几百条 | 条件匹配 + 语义相似度 |

### 2.2 三层召回引擎

```
┌─────────────────────────────────────────────────┐
│  Query Router（查询路由器）                        │
│  接收查询请求，判断走哪一层                         │
├─────────────────────────────────────────────────┤
│                                                 │
│  Layer 1: Structured Index（结构化索引）          │
│  ├── 存储：SQLite（单文件，本地优先）              │
│  ├── 内容：对象元数据                             │
│  ├── 查询：精确匹配、过滤、排序、关联              │
│  ├── 成本：零 LLM Token                         │
│  └── 示例："WI-012 的所有 Handoff"               │
│                                                 │
│  Layer 2: Full-Text Search（全文检索）            │
│  ├── 存储：SQLite FTS5 或 tantivy               │
│  ├── 内容：Artifact 全文、Checkpoint 文本         │
│  ├── 查询：关键词匹配、短语搜索                    │
│  ├── 成本：零 LLM Token                         │
│  └── 示例："callback TypeError"                  │
│                                                 │
│  Layer 3: Semantic Index（语义索引）              │
│  ├── 存储：本地向量库（usearch）                  │
│  ├── 内容：段落级嵌入                             │
│  ├── 查询：自然语言语义相似度                      │
│  ├── 成本：嵌入计算（远低于 LLM 推理）             │
│  └── 示例："为什么放弃了 passport.js？"           │
│                                                 │
└─────────────────────────────────────────────────┘
```

### 2.3 统一存储引擎：PostgreSQL

三层召回不再使用三个独立引擎（SQLite + tantivy + usearch），而是统一到 **PostgreSQL** 一个引擎中：

- **Layer 1 结构化索引**：PostgreSQL 原生关系查询 + JSONB + 数组操作
- **Layer 2 全文检索**：PostgreSQL 内置 tsvector/tsquery，支持权重、排名、中文分词（zhparser 扩展）
- **Layer 3 语义索引**：pgvector 扩展，支持 HNSW / IVFFlat 近似最近邻搜索

单引擎的优势：
- 三层查询可以在一条 SQL 中组合（结构化过滤 + 全文搜索 + 语义排序）
- 事务一致性：写入和索引更新在同一个事务中
- 运维简单：一个服务，一套备份
- 并发安全：多进程同时读写不会锁冲突

### 2.4 Layer 1: Structured Index

```sql
CREATE TABLE objects (
  id TEXT PRIMARY KEY,
  type TEXT NOT NULL,
  status TEXT,
  owner_seat TEXT,
  created_at TIMESTAMPTZ,
  updated_at TIMESTAMPTZ,
  metadata JSONB DEFAULT '{}'
);

CREATE TABLE relations (
  from_id TEXT REFERENCES objects(id),
  to_id TEXT REFERENCES objects(id),
  relation_type TEXT,
  PRIMARY KEY (from_id, to_id, relation_type)
);

CREATE TABLE events (
  id TEXT PRIMARY KEY,
  event_type TEXT NOT NULL,
  actor TEXT,
  object_id TEXT,
  timestamp TIMESTAMPTZ NOT NULL,
  summary TEXT,
  payload JSONB DEFAULT '{}'
);

CREATE INDEX idx_objects_type ON objects(type);
CREATE INDEX idx_objects_status ON objects(status);
CREATE INDEX idx_objects_owner ON objects(owner_seat);
CREATE INDEX idx_objects_metadata ON objects USING GIN(metadata);
CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_actor ON events(actor);
CREATE INDEX idx_events_time ON events(timestamp);
```

**同步机制**：Reconcile Engine 从 `.seatloom/` 的 YAML/JSONL 文件同步到 PostgreSQL。PG 是索引层，YAML/JSONL 文件是主数据（保证 Git 可追踪）。

**查询成本**：零 Token，毫秒级返回。

### 2.5 Layer 2: Full-Text Search (tsvector)

```sql
-- Artifact 内容分块存储 + 全文索引
CREATE TABLE artifact_chunks (
  artifact_id TEXT,
  chunk_index INT,
  section_heading TEXT,
  chunk_text TEXT NOT NULL,
  search_vector tsvector GENERATED ALWAYS AS (
    to_tsvector('english', chunk_text)
  ) STORED,
  PRIMARY KEY (artifact_id, chunk_index)
);

CREATE INDEX idx_chunks_fts ON artifact_chunks USING GIN(search_vector);
```

**索引内容**：所有 Artifact 全文（按段落切片）+ Checkpoint 文本字段 + Playbook 内容

**查询成本**：零 Token。

### 2.6 Layer 3: Semantic Index (pgvector)

**为什么需要**：前两层处理精确查询，但很多查询是语义性的：
- "上次为什么放弃了某方案？"
- "跟认证相关的所有决策"（"认证" = "auth" / "OAuth" / "登录" / "credential"）
- Playbook 匹配：当前场景与哪个历史场景最相似？

```sql
CREATE EXTENSION IF NOT EXISTS vector;

CREATE TABLE artifact_embeddings (
  artifact_id TEXT,
  chunk_index INT,
  embedding vector(768),
  PRIMARY KEY (artifact_id, chunk_index),
  FOREIGN KEY (artifact_id, chunk_index) REFERENCES artifact_chunks(artifact_id, chunk_index)
);

CREATE INDEX idx_embed_hnsw ON artifact_embeddings
  USING hnsw (embedding vector_cosine_ops);
```

**嵌入模型**：本地小模型（nomic-embed-text, ~100M 参数）或 API（OpenAI embedding, ~$0.0001/1k tokens）

**索引粒度**：按段落/章节切片（与 artifact_chunks 表对齐），不是整个文件一个向量

**三层联合查询示例**：

```sql
-- 在 accepted reviews 中，语义搜索最相关的段落
SELECT ac.artifact_id, ac.section_heading, ac.chunk_text,
       ae.embedding <=> $1::vector AS distance
FROM artifact_embeddings ae
JOIN artifact_chunks ac USING (artifact_id, chunk_index)
JOIN objects o ON o.id = ae.artifact_id
WHERE o.type = 'review' AND o.status = 'accepted'
ORDER BY ae.embedding <=> $1::vector
LIMIT 5;
```

**查询成本对比**：

| 方式 | 找到"为什么放弃 passport.js"的成本 |
|------|--------------------------------|
| LLM 读完整文档（~5k tokens input） | ~$0.015 |
| 语义搜索 → 精确段落 → LLM 只读相关段（~500 tokens） | ~$0.0025 |
| **节省** | **~83%** |

### 2.6 三层协作流程

Pack Engine 组装 LaunchPack 时的召回流程：

```
1. Layer 1 (SQLite): 精确查询
   "WI-012 的 AC、owner、关联 Session"
   → 结构化数据，零 Token

2. Layer 2 (FTS): 关键词搜索
   "WI-012 相关 Artifact 中含 'TypeError' 的内容"
   → 精确文件和片段，零 Token

3. Layer 3 (Vector): 语义召回（仅在需要时）
   "与当前 blocker 最相似的历史 Playbook"
   → 匹配结果，极低成本

4. Pack Engine 汇总
   → 按 Context Tier 分层组装
   → 总 Token 消耗最小化
```

### 2.7 渐进实现计划

| 阶段 | 实现 | 额外依赖 |
|------|------|---------|
| MVP | PostgreSQL Layer 1 (relational) + Layer 2 (tsvector FTS) | PostgreSQL（embedded 或 Docker sidecar） |
| P1 | Layer 3 (pgvector + 本地嵌入模型) | pgvector 扩展 + ~100MB 嵌入模型 |
| P2 | Layer 3 可选 API 嵌入（更高质量） | 网络（可选） |

PostgreSQL 部署策略：Tauri 应用启动时自动管理一个本地 PG 实例（通过 embedded-postgres crate 或 Docker），用户无需手动安装。

### 2.8 三层召回与用户功能的关系

| 用户功能 | 依赖层 | LLM 成本 |
|---------|--------|---------|
| Inbox 条目生成 | Layer 1 | 零 |
| Timeline 过滤和搜索 | Layer 1 + Layer 2 | 零 |
| LaunchPack 自动组装 | Layer 1 + Layer 2 + Layer 3 | 极低（仅嵌入） |
| "为什么做了这个决策？" | Layer 3 + LLM 解释 | 低（只读精确段落） |
| Playbook 自动匹配 | Layer 3 | 极低 |
| Supervisor 查找相关上下文 | Layer 1 + Layer 2 + Layer 3 | 低 |
| Artifact 审阅中搜索 | Layer 2 + Layer 3 | 零到极低 |

---

## 3. 两个问题的关系

分层上下文管理和分层召回是同一个核心理念的两面：

```
分层上下文 = 写入侧优化（用最小成本存储最大信息）
分层召回   = 读取侧优化（用最小成本获取最精准信息）
```

它们在 Pack Engine 中汇合：

```
Pack Engine
  ├── 写入：Session Checkpoint → 结构化分层存储 → 索引更新
  └── 读取：查询请求 → 三层召回 → 按 Context Tier 组装 → 注入 LaunchPack
```

这直接支撑了 SeatLoom 的核心价值主张：**Token ROI 最大化**。每一个存储和检索操作都在工程层面确保"只在刀刃上花 Token"。

---

*本文档待 Mr. Zhang 审阅，结论应反馈给 Lyra 纳入 PRD v0.5 和架构设计。*
