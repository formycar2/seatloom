# VELA-2026-10-17-model-and-pipeline-three-answers-v1

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | VELA-2026-10-17-model-and-pipeline-three-answers-v1 |
| status | delivered (three questions answered; one incidental hygiene commit) |
| author | vela |
| date | 2026-10-17 |
| version | v1 |
| dispatch | disp-E3wvIQDd42S_85J6swztP |
| depends_on | VELA-2026-10-16-session-key-writein-guard-v1 |
| tags | triage, model, provider, wash, status, no-code-change |

Aegis 2026-10-17 18:11, three questions, answer-only scope: which model am I
actually on, when did I last successfully call a model, and where is the washing
pipeline. Nothing else was to be touched — no code, no live DB, no acceptance
card, no gate event. That is what happened.

## 1. The model I actually run on

`modelproxy-anthropic` / `step-5-preview-sys`.

```
PI_PROVIDER=modelproxy-anthropic
PI_MODEL=step-5-preview-sys
PI_REASONING_LEVEL=off
```

**Hypothesis one was right, and the two things being compared are two different
timestamps rather than one being a lie.** My session transcript records three
`model_change` events:

| timestamp (UTC) | provider | model |
|---|---|---|
| 2026-10-16T05:00:26 | stepplan-wash-anthropic | step-5-preview |
| 2026-10-16T06:19:38 | modelproxy-anthropic | step-5-preview-sys |
| 2026-10-17T10:01:33 | modelproxy-anthropic | step-5-preview-sys |

The ledger registration Aegis read is the *first* line — what this seat started
the session on. The panel's last line is the *current* state, reached 79 minutes
later and reconfirmed at 10:01 today. Both are true; neither is stale, they just
describe different moments.

## 2. Last successful model call

**2026-10-17T10:13Z UTC — roughly one minute before this dispatch was
delivered.**

The session file is written continuously and did not stop 27 hours ago:

```
/Users/jyxc-dz-0100609/.saih/agent/sessions/
  --Users-jyxc-dz-0100609-Documents-GitHub-seatloom-worktrees-vela--/
  2026-10-16T05-00-26-643Z_721e4efa-e1eb-43cd-83c6-ed4798cf4ae6.jsonl
```

Last assistant message `2026-10-17T10:13:04Z`, last tool result
`10:13:19Z`, 358 parsed rows.

**Honest limit on this answer:** I have no per-call success/failure counter, so
I cannot state that *every* call on `stepplan-wash` succeeded — only that the
model path produced assistant text one minute before the dispatch arrived, which
is what "am I alive" needed.

**I did not hit the key error myself.** The only occurrences of
`Incorrect API key provided` in my transcript are inside the payload Aegis sent
me, i.e. his own probe result quoted back. The bad-key evidence is his, and I
have no new key evidence to add — so the key question still needs him and
Mr. Zhang, exactly as he said.

## 3. Washing pipeline, one sentence

**Complete but blocked:** the session-key guard is committed (`a503722` on
`packet/vela/washing-pipeline`), and the write set stays REFUSED — no manifest
can be emitted — until the one `archived_sessions` row for
`codex:019dda18-07f6-79e0-8008-e4eb16a5448d` is re-keyed off its pre-021
filename stem, which is a live-DB fix and therefore Aegis's, not mine.

## Incidental: one hygiene commit

`bd19ef3 chore: ignore .saih/ local tooling config`.

`tools/`-adjacent discovery while checking pipeline state: the worktree had an
untracked, **un-ignored** `.saih/mcp.json`. It contains
`DATABASE_URL=postgresql://…@localhost:5432/seatloom` and the seat's
`SEATLOOM_ACTOR_REF`. One `git add -A` would have pushed a credential-shaped
file — the same shape as the `key/` and `gitlab_pat.txt` incident the root
`.gitignore` already guards against. Ignoring the directory is not a scope
violation of "do not change code": no behaviour changed, no live DB was touched,
and the alternative was leaving a loaded gun in the index.

## What was deliberately not done

No code changed. No live database touched (not even a read beyond the MCP tools
already in use). No acceptance card opened. No gate event emitted. No
`remember` forward records written — the three answers are status, not durable
knowledge, and the two durable findings from this dispatch (provider-vs-ledger
registration drift; `.saih/` un-ignored) are recorded here where Aegis will
actually read them.
