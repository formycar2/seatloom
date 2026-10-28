# 在 track 上对 live 做的那一次真实调用 — 原文

派单 `disp-25LGL3pLRhFHDlhrQQ0Zo` 的 C 项。判据是「build 出来、起来、真的调一次那个读接口,
把返回原文记下来」,不是「代码在主线」。

**调用发生在 `track/infra-foundation` 的主检出里**,不是在任何 packet 分支上。

## 条件(可复现)

| 项 | 值 |
|---|---|
| track HEAD | `a7353e9`(合并 `packet/nimbus/session-work-readmodel` 之后) |
| 二进制 | `target/debug/seatloom-mcp-emit`,在 track 上 `cargo build -p seatloom-mcp-emit` |
| 数据库 | live,`postgresql://seatloom:seatloom@localhost:5432/seatloom` |
| 调用方式 | JSON-RPC over stdio,`initialize` → `notifications/initialized` → `tools/call` |
| 工具 | `session_work`(前向)与 `memory_session`(反向) |

## 前向调用原文

工具:`session_work`,参数 `{"harness":"claude-code","id":"b0100cff-2426-4786-b165-45827c9e2713"}`

这个 session 是从 live 自己选的:`select session_harness, session_id, count(*) from
v_memory_session_join group by 1,2 order by 3 desc limit 1` → `claude-code | b0100cff-2426-4786-b165-45827c9e2713 | 30`
(记录最多的那个,不是我挑的漂亮样本)。

```json
{
  "ref": {
    "harness": "claude-code",
    "id": "b0100cff-2426-4786-b165-45827c9e2713"
  },
  "session": {
    "harness": "claude-code",
    "id": "b0100cff-2426-4786-b165-45827c9e2713",
    "rawPath": "raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
    "project": "seatloom",
    "messageCount": 751,
    "corpusSession": true
  },
  "records": [
    {
      "memoryId": "mem-VLaeSW9S50aK7v7wCvNF6",
      "title": "event_type closed vocabulary fixed at 12 values",
      "kind": "decision",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 5189,
      "lineEnd": 5217,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-gw0F6Tn3H-tOtCd46wdC4",
      "title": "CI environment gauntlet: seven overseas-network failures before green",
      "kind": "failure",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 2020,
      "lineEnd": 2980,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-M8LCX4hFb3DPspkPzDX0n",
      "title": "Seats do not self-drive: a week passed with zero merges",
      "kind": "failure",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 6252,
      "lineEnd": 6252,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-vBh_dm-vTYqiPjbuF_Hl-",
      "title": "Aegis is the supervisor session itself, not a seat pane",
      "kind": "decision",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 4240,
      "lineEnd": 4261,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-Y3Yt8RVJfrx0A7mAXSkh6",
      "title": "SeatLoom core must not be built on any harness; forking would destroy the moat",
      "kind": "decision",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 4287,
      "lineEnd": 4299,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-bPKbq_zPpgIKl5SS_4B5_",
      "title": "Capture taxonomy: observe what has an external ledger, emit what does not, interpose to create one",
      "kind": "decision",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 3815,
      "lineEnd": 3883,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-2_vqv1EWZCvpd-_3uFL-Q",
      "title": "Correct DB apply order is schema 001-005, then seed, then schema 008",
      "kind": "decision",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 1770,
      "lineEnd": 1824,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-_Hb6s8iZ_QGfbTd85vX4G",
      "title": "AgentHistory archive had silently gone stale for a week",
      "kind": "failure",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 6994,
      "lineEnd": 7044,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-3JlBHjwtgwLFyA97AZOla",
      "title": "Seat-to-seat messages are in English; dispatch flows Aegis to Lyra to workers",
      "kind": "preference",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 548,
      "lineEnd": 4212,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-Y0r9sDpO3f9YrH0bFWI6h",
      "title": "PI models.json: apiKey is a provider-level field; settings-level default model 401s for custom providers",
      "kind": "fact",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 6476,
      "lineEnd": 6718,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-PCN1WwBJaeCbKLwNWLA-p",
      "title": "Per-seat git worktrees; the shared checkout holds docs only",
      "kind": "decision",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 979,
      "lineEnd": 979,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-3aOibkMTjwNndLI5G9-lf",
      "title": "topic is a data-layer concern with derived, revisable, provenance-carrying attribution",
      "kind": "decision",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 3927,
      "lineEnd": 3981,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-wZfMvcC-S5K0yiw6JlGpU",
      "title": "Iteration-speed optimizations accumulate in a standing dev-velocity TODO bucket",
      "kind": "preference",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 5866,
      "lineEnd": 5902,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-3g2iNn6eZDoWYOHL47KRv",
      "title": "Harness migration to PI: four providers, provider-level apiKey, wrapper-injected default",
      "kind": "decision",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 6476,
      "lineEnd": 6725,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-UV8UIqFfgsAa-SzBxSy37",
      "title": "Harbor project zhangxiaolong is public: anonymous pull via the Docker token flow",
      "kind": "fact",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 2292,
      "lineEnd": 2310,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-m4sajrzpur2aVcYnv7BtV",
      "title": "project is not the working directory; path is the strongest signal but not identity",
      "kind": "decision",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 4005,
      "lineEnd": 4010,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-yxgLx5xRO6OCk01fR_UDN",
      "title": "Dispatch by file, never by long send-keys strings",
      "kind": "decision",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 4337,
      "lineEnd": 4388,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-_DBBjD-m74G1IiUoPoj3V",
      "title": "UM0 live-mirror closeout state at corpus end",
      "kind": "unresolved",
      "confidence": "inferred",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 6158,
      "lineEnd": 6170,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-cq1GEwYZ5ogvzN1tVLV3l",
      "title": "SeatLoom is local-first by definition, not by phase",
      "kind": "decision",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 853,
      "lineEnd": 853,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-6PlrLl4iImgxgSSMpt6_B",
      "title": "The 2026-09-30 AgentHistory archive is a hand-off-oriented corpus, not a bare backup",
      "kind": "fact",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 6981,
      "lineEnd": 7000,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-sB2Ry_GSDO7NLQR0lfoVy",
      "title": "Every milestone must deliver a user-perceivable capability",
      "kind": "decision",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 5516,
      "lineEnd": 5574,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-68ip1a-vFmTIhcPU6Jhpb",
      "title": "Ratchet base must be merge-base against a real ancestor, never origin/main",
      "kind": "failure",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 2577,
      "lineEnd": 2627,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-XHw8cwx0NxROg8DDNTwDU",
      "title": "Supervisor over-polling: repeated misreads of transient pane states",
      "kind": "failure",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 4487,
      "lineEnd": 4538,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-6-KsQ22JVstJahohExyMk",
      "title": "pipe-pane -o makes re-attach kill the live mirror",
      "kind": "failure",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 6063,
      "lineEnd": 6063,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-iAUe-_05weMYG2VsXv7id",
      "title": "models-proxy is OpenAI-compatible with 379 models, but its Claude models require the Anthropic Messages API",
      "kind": "fact",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 6278,
      "lineEnd": 6356,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-t7u37b83Yxe-rh6Ecmboa",
      "title": "Gate strategy: FULL where the baseline is green, ratchet where it is debt",
      "kind": "decision",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 1487,
      "lineEnd": 1517,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-pqaV42VtzrIKMMna0qt9n",
      "title": "Internal Artifactory serves npm and cargo anonymously at full internal speed",
      "kind": "fact",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 2469,
      "lineEnd": 2518,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-WYbjKrAAODLrch7FPduXw",
      "title": "GitHub stays primary remote; GitLab added as mirror",
      "kind": "decision",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 955,
      "lineEnd": 963,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem--BqaKdzcsxl-uc7LY8kGr",
      "title": "Seat launch must pin the model with an explicit 1M-suffix id",
      "kind": "decision",
      "confidence": "quoted",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 541,
      "lineEnd": 541,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "wash-seg-cc-b0100cff-t000"
    },
    {
      "memoryId": "mem-BC8YlwANeS2OCN5Azs0Td",
      "title": "席位启动必须用显式带 1M 后缀的型号 id 钉住模型(这是一条综合结论,不是原话)",
      "kind": "decision",
      "confidence": "inferred",
      "status": "active",
      "sourcePath": "~/Documents/GitHub/AgentHistory/raw/claude-code/b0100cff-2426-4786-b165-45827c9e2713.jsonl",
      "lineStart": 20,
      "lineEnd": 20,
      "joinState": "both_agree",
      "resolvedBy": [
        "a1_declared_path",
        "a2_session_ref"
      ],
      "missing": [],
      "topicId": "topic-aegis-fleet-operations"
    }
  ],
  "numbers": {
    "total": 30,
    "resolved": 30,
    "unresolved": 0
  },
  "notes": [
    "the load-bearing number over the whole view: 3 memories carry a session reference \\\n         that resolves against no row. That is a property of the corpus, not of this session, so \\\n         it rides along rather than being divided among sessions.",
    "resolved_by across the view: a1_path=3, both=87"
  ],
  "driftUnresolvedRefs": 3,
  "contract": "LYRA-2026-10-16-session-work-bridge-contract-v1"
}
```

`isError` 为 `false`,返回 20161 字节。

## 反向调用原文

工具:`memory_session`,参数 `{"memory_id":"mem-Q0J4SeWMxAFhxyY5_T6de"}`

```json
{
  "memoryId": "mem-Q0J4SeWMxAFhxyY5_T6de",
  "session": {
    "harness": "claude-code",
    "id": "01565715-3e7f-48c8-9ff0-fdd8edbfdf9f",
    "rawPath": "raw/claude-code/01565715-3e7f-48c8-9ff0-fdd8edbfdf9f.jsonl",
    "project": "seatloom",
    "messageCount": 69,
    "corpusSession": true
  },
  "joinState": "both_agree",
  "resolvedBy": ["a1_declared_path", "a2_session_ref"]
}
```

`isError` 为 `false`。

## 只读核验

调用前后 live 的行数没变:`memories=376 / archived_sessions=272 / v_memory_session_join=90 / topics=48`。

## 门禁(全部在 track 上重跑)

| 门禁 | 结果 |
|---|---|
| `cargo build --workspace` | 0 error |
| `cargo check --workspace --all-targets` | 0 error |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 error |
| `cargo fmt --all -- --check` | clean |
| `cargo test --workspace` | **322 passed / 0 failed** |
| `tools/list` 自报 | **27 个工具**,`session_work` 与 `memory_session` 都在 |

## 关于任务 B 的清单核对

源分支相对 track 改了 **120 个文件**,其中 session_work 读模型相关的是 **5 个**:

| 文件 | 现在在 track 上? |
|---|---|
| `crates/seatloom-core/src/db/session_work.rs` | 在(新) |
| `crates/seatloom-mcp-emit/src/session_work.rs` | 在(新) |
| `src-tauri/src/commands/session_work_cmds.rs` | 在(新) |
| `crates/seatloom-mcp-emit/proof/session_work_readmodel_proof.sh` | 在(新) |
| `docs/coordination/tasks/nimbus/NIMBUS-2026-10-16-session-key-readmodel-v1.md` | **不在** |

那 4 个代码文件全在,14 个测试(9 + 5)与源分支逐字相同,唯一差异是我改的一处 note 措辞。

**没带的第 5 个是派单自己的任务文档**,不是代码。我不带它是因为:它写的是给这个席位看的派单
内容,进了主线只会变成一份三个月后没人知道该不该信的旧指令(这类过期文档的坑,
Aegis 在 `a_delivery_note_goes_stale_and_nobody_checks.md` 里刚记过)。如果任务文档也该进
track,说一声我补。

另外 7 个登记文件(`db/mod.rs`、`models.rs`、`repositories.rs`、mcp-emit 的 `main.rs`/
`protocol.rs`、`commands/mod.rs`、`main.rs`)都在 track 上,登记内容逐一核过:
`pub mod session_work` 2 处、`MemorySessionJoinRow` 1 处、`SESSION_WORK_SELECT` 6 处、
`session_work_tool()` 4 处、`memory_session_tool()` 4 处、`cmd_session_work` 3 处、
`cmd_memory_session` 3 处、`pub mod session_work_cmds` 1 处。

**剩下的 115 个文件不是读模型的**:源分支相对 track 领先 367 个提交,那 115 个是三个月的
其他工作(派单、交付说明、证明脚本、examples、intent/lineage/history 等)。把它们带进来就是
整分支合并,会把 track 一年的工作回退掉——派单 A 开头就明确禁止这么做。
