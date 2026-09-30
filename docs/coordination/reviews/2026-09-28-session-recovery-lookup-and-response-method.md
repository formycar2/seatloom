# Session Recovery Lookup and Response Method

| Field | Value |
|---|---|
| template | T4 |
| subtype | gap_review |
| id | seatloom-session-recovery-method-2026-09-28 |
| status | issued |
| author | human-directed-codex |
| date | 2026-09-28 |
| version | v1 |
| depends_on | `/Users/jyxc-dz-0100609/workspace-restore/README.md`, `/Users/jyxc-dz-0100609/workspace-restore/recovery-retrospective.md`, `/Users/jyxc-dz-0100609/iterm-tmux-snapshots/20260928-170427/manifest.json`, `/Users/jyxc-dz-0100609/iterm-tmux-snapshots/20260928-170427/restore-reports/restore-report-latest.json` |
| tags | session-recovery, iterm2, tmux, codex, stepcode, continuity, handoff, evidence |

## 0. Purpose

This document records the exact investigation and recovery method used when the `stepatlas` Codex work surface did not return after an iTerm2/tmux workspace restore.

The important operational distinction is:

```text
layout restored != agent context restored
```

The restore report can prove window, tab, pane, and tmux counts while an individual Codex or Claude session is still sitting at a shell, an update prompt, a missing-session screen, or a model-capacity screen.

This is a reusable SeatLoom continuity case. It is not a request to run another full restore now.

## 1. Snapshot Baseline

Snapshot:

```text
/Users/jyxc-dz-0100609/iterm-tmux-snapshots/20260928-170427
```

The snapshot manifest reported:

| Surface | Expected |
|---|---:|
| iTerm2 windows | 5 |
| iTerm2 tabs | 16 |
| iTerm2 sessions | 16 |
| tmux sessions | 12 |
| tmux panes | 13 |

The restore report was structurally passing:

```text
status: pass
iterm_window_count: pass
iterm_tab_count: pass
iterm_session_count: pass
tmux_session_count: pass
tmux_pane_count: pass
tmux_session_names: pass
window bounds and tab counts: pass
```

The manifest safety flags were:

```text
secrets_excluded: true
captured_text_redacted: true
restore_requires_explicit_execute: true
```

These safety flags must remain true for future snapshots.

## 2. Full Runtime Inventory

### 2.1 iTerm2 inventory

The relevant snapshot inventory was:

| Window | Tab | Visible name | Saved command or classification |
|---:|---:|---|---|
| 1 | 1 | `step - jyxc-dz-0100609 (-zsh)` | shell |
| 2 | 1 | `feishu-assistant (tail)` | shell / bridge |
| 2 | 2 | `jyxc-dz-0100609 (codex)` | `codex resume 019f69b0-830e-7ea3-afcc-6846c896fc65` |
| 3 | 1 | `Claude Code (claude)` | `stepcode claude --resume 104f1244-3547-438c-bd6c-4c5cc8bffcb4` |
| 3 | 2 | `Claude Code (claude)` | `stepcode claude --continue` |
| 3 | 3 | `stepatlas (codex)` | `codex resume 019ed021-bcd6-7710-8ced-5063169cf652` |
| 3 | 4 | `stepatlas (codex)` | original snapshot command: `codex resume --all` |
| 3 | 5 | `Claude Code (claude)` | `stepcode claude --resume cf6b7611-8edc-4c9f-960c-462ebc4d534a` |
| 3 | 6 | `Claude Code (claude)` | `stepcode claude --continue` |
| 3 | 7 | `quota-topo (claude)` | `stepcode claude --resume 9deb64b0-78c8-47be-b14e-2de78a093884` |
| 4 | 1 | `storage (codex)` | `codex resume 019f6ac0-fe53-7982-83fb-d8c54f86029d` |
| 4 | 2 | `fde_field_evaluation (claude)` | `stepcode claude --resume 510e3b75-11aa-4915-8350-6cb397bafc6a` |
| 4 | 3 | `implementation codebase sync / openapi (codex)` | `codex resume --last` |
| 4 | 4 | `StepFun step-5-preview coding agent` | `stepcode claude --resume ff4d92a9-5aa9-43cc-9eb8-910010f439ed` |
| 4 | 5 | `cn-gpu-infer-fabric (codex)` | `codex resume 019ff53e-8436-7971-82f7-45f7b84ae428` |
| 5 | 1 | `reply OK (codex)` | `codex resume --profile step 01a0d354-4bcc-7960-8101-15611975df12` |

The saved terminal transcripts are under:

```text
/Users/jyxc-dz-0100609/iterm-tmux-snapshots/20260928-170427/iterm/sessions/
```

The two `stepatlas` transcripts used in this investigation were:

```text
/Users/jyxc-dz-0100609/iterm-tmux-snapshots/20260928-170427/iterm/sessions/window-3/tab-3-session-1-stepatlas_codex.txt
/Users/jyxc-dz-0100609/iterm-tmux-snapshots/20260928-170427/iterm/sessions/window-3/tab-4-session-1-stepatlas_codex.txt
```

### 2.2 tmux inventory

The 12 tmux sessions in the snapshot were:

```text
Aegis-Supervisor
Flux-Quality&Ops-seatloom
Lyra-po-seatloom
Mira-UX/UED-seatloom
Nimbus-TechArchi-seatloom
Onyx-data-seatloom
aegis-feishu
cp-alpha-ws
ebook_guardian_web
hwfs-test
quota-topology-ui
ss-ws-test
```

The session roles visible from the snapshot were:

| Session group | Sessions |
|---|---|
| SeatLoom agent seats | `Aegis-Supervisor`, `Flux-Quality&Ops-seatloom`, `Lyra-po-seatloom`, `Mira-UX/UED-seatloom`, `Nimbus-TechArchi-seatloom`, `Onyx-data-seatloom` |
| Feishu bridge | `aegis-feishu` |
| Workspace / infrastructure | `cp-alpha-ws`, `hwfs-test`, `ss-ws-test` |
| Other local work | `ebook_guardian_web`, `quota-topology-ui` |

The tmux pane captures are under:

```text
/Users/jyxc-dz-0100609/iterm-tmux-snapshots/20260928-170427/tmux/panes/
```

## 3. The Actual Failure

There were two visually identical `stepatlas` tabs, but they had different recovery states.

### 3.1 Window 3 / Tab 3: already recovered

Saved command:

```bash
codex resume 019ed021-bcd6-7710-8ced-5063169cf652
```

Historical session file:

```text
/Users/jyxc-dz-0100609/.codex/sessions/2026/06/16/rollout-2026-06-16T19-12-12-019ed021-bcd6-7710-8ced-5063169cf652.jsonl
```

This session was already running. Its visible state was:

```text
Planning documentation inspection and parsing
Selected model is at capacity. Please try a different model.
Conversation recap ...
```

The model-capacity message was a runtime availability problem, not a failed session restore. The session context itself was present and must not be opened a second time.

### 3.2 Window 3 / Tab 4: not recovered

Snapshot command:

```bash
codex resume --all
```

Snapshot terminal:

```text
jyxc-dz-0100609@JYXC-DZ-0100609deMacBook-Pro stepatlas %
```

At the time of inspection:

- the tab was a shell;
- there was no Codex child process for this tab;
- the tab had no stable session ID in the saved command;
- the current iTerm2 session ID was `C724C976-0ADA-4AEF-AAA8-C989BE8A6ACB`;
- the snapshot tty was `ttys013`.

This was the only `stepatlas` tab that required a recovery action.

## 4. Lookup Procedure

The following procedure resolved the missing session without closing the desktop or rebuilding the whole arrangement.

### Step 1: Inspect process truth

Use the process tree to distinguish a real Agent from a shell or wrapper:

```bash
ps -axo pid,ppid,command | rg -i 'codex|stepcode|stepgo' | rg -v 'rg -i'
```

Interpretation:

| Observation | Meaning |
|---|---|
| `codex resume <id>` plus the native Codex child | session is running |
| `stepcode claude ...` plus a Claude child | StepCode Claude is running |
| only shell / login process | no Agent has been restored in that terminal |
| update screen or missing-session screen | command launched, but context is not usable yet |

### Step 2: Inspect the saved terminal transcript

Read the specific tab transcript, not the entire snapshot:

```bash
sed -n '1,220p' \
  /Users/jyxc-dz-0100609/iterm-tmux-snapshots/20260928-170427/iterm/sessions/window-3/tab-4-session-1-stepatlas_codex.txt
tail -n 140 \
  /Users/jyxc-dz-0100609/iterm-tmux-snapshots/20260928-170427/iterm/sessions/window-3/tab-4-session-1-stepatlas_codex.txt
```

The transcript contained a unique work result:

```text
ABC真实审批通过-20260914T1530Z
```

The exact text is more useful than the tab title because it identifies the conversation content rather than only the terminal layout.

### Step 3: Search local Codex history and session files

Search the lightweight indexes first:

```bash
rg -n -F 'ABC真实审批通过-20260914T1530Z' \
  /Users/jyxc-dz-0100609/.codex/history.jsonl \
  /Users/jyxc-dz-0100609/.codex/session_index.jsonl
```

Then search the saved JSONL transcripts:

```bash
find /Users/jyxc-dz-0100609/.codex/sessions \
  -type f -name '*.jsonl' -print0 |
  xargs -0 rg -l -F 'ABC真实审批通过-20260914T1530Z'
```

The search identified:

```text
019f668d-0b84-78d1-a34f-cddd777cebd5
```

The corresponding session file was:

```text
/Users/jyxc-dz-0100609/.codex/sessions/2026/07/16/rollout-2026-07-16T00-12-27-019f668d-0b84-78d1-a34f-cddd777cebd5.jsonl
```

The first `session_meta` record confirmed the cwd:

```text
/Users/jyxc-dz-0100609/Documents/GitHub/sf-code/stepatlas
```

The final conversation records confirmed that this was the Atlas/ABC workflow session, not the other `stepatlas` Codex session.

### Step 4: Check for duplicate ownership

Before launching the candidate, compare it with all currently running Codex IDs:

```bash
ps -axo pid,ppid,command |
  rg -i 'codex resume|codex-darwin|019ed021|019f668d' |
  rg -v 'rg -i'
```

The result showed:

```text
019ed021-bcd6-7710-8ced-5063169cf652  -> already running in Window 3 / Tab 3
019f668d-0b84-78d1-a34f-cddd777cebd5  -> not yet running in Window 3 / Tab 4
```

This duplicate-ownership check is mandatory. The same Codex session must not be opened in two terminals.

### Step 5: Send the command only to the failed iTerm session

The targeted AppleScript was:

```applescript
tell application "iTerm2"
  tell window 3
    tell tab 4
      tell session 1
        write text "cd /Users/jyxc-dz-0100609/Documents/GitHub/sf-code/stepatlas && codex resume 019f668d-0b84-78d1-a34f-cddd777cebd5"
      end tell
    end tell
  end tell
end tell
```

No iTerm window was closed. No tmux session was killed. No global restore command was run.

### Step 6: Verify the result

Process verification showed the new Codex process:

```text
node /opt/homebrew/bin/codex resume 019f668d-0b84-78d1-a34f-cddd777cebd5
native Codex child for the same session
```

The tab screen returned to the saved ABC workflow context, including:

```text
下一步 prompt 已生成并写入执行计划
instance_key = 2251799816789802
```

The recovery was therefore context-correct, not merely layout-correct.

## 5. Response Method Used With the User

The response pattern that worked was:

1. State the safety boundary first: only inspect and repair the one failed tab.
2. Inspect evidence before sending commands.
3. Explain the distinction between:
   - layout restoration;
   - process restoration;
   - conversation/session restoration.
4. Identify the exact failed tab and the exact candidate session ID.
5. Explicitly state which existing session must not be reopened.
6. Send one targeted command.
7. Verify the child process and visible context.
8. Report the exact tab, cwd, session ID, and non-actions.

A concise operational update should contain:

```text
target tab
candidate session ID
evidence used
command sent
verification result
what was deliberately not touched
```

Do not claim success from a count-only restore report. Do not claim a model-capacity screen is a missing session. Do not use `codex resume --all` when the intended historical session can be identified deterministically.

## 6. Known Failure Modes and Guardrails

### 6.1 iTerm2 AppleScript property failures

Broad collection expressions such as these were unreliable:

```applescript
get name of every tab of every window
get id of every tab of window 3
```

They can fail with AppleScript error `-1728` even when iTerm2 is running. Prefer direct indexed traversal:

```applescript
tell application "iTerm2"
  tell window 3
    tell tab 4
      tell session 1
        return contents
      end tell
    end tell
  end tell
end tell
```

### 6.2 `codex resume --all` is not a deterministic recovery key

`--all` opens a picker and requires a human choice. It is appropriate only when no stable ID can be recovered. When a unique transcript string identifies a historical JSONL file, use the explicit ID instead.

### 6.3 Update prompts are separate from session lookup

Codex update prompts and StepCode update prompts are different mechanisms. A restore agent must:

- select `Skip`, `Later`, or the equivalent non-upgrade choice when a version prompt blocks recovery;
- not assume that a successful upgrade resumes the original session;
- re-check the visible prompt and process tree after the update screen exits.

The StepCode config changes made earlier were:

```json
"updateCheckEnabled": false
```

in:

```text
/Users/jyxc-dz-0100609/.stepcode/config.json
/Users/jyxc-dz-0100609/.stepgo/config/config.json
```

These settings do not disable Codex's own update prompt.

### 6.4 Do not run full restore during a local repair

The command below closes and rebuilds the current iTerm layout. It was intentionally not run during this repair:

```bash
restore-from-snapshot.py --execute
```

Use it only as an explicit, separately approved operation.

## 7. Recovery Acceptance Criteria

A future recovery run is accepted only when all applicable criteria pass:

| Criterion | Required evidence |
|---|---|
| iTerm layout exists | arrangement, window count, tab count |
| tmux inventory exists | session names and pane count |
| target shell is identified | tty or iTerm session ID |
| candidate historical session is found | session file path plus `session_meta.cwd` |
| duplicate ownership is excluded | process tree |
| exact command is sent to the target | targeted AppleScript or tmux command |
| agent process is alive | process tree after launch |
| correct conversation is visible | screen contents or transcript marker |
| non-target sessions remain untouched | before/after process and tab evidence |
| deferred or failed items are recorded | durable report with reason |

## 8. Product Implications for SeatLoom

This incident maps directly to SeatLoom continuity requirements:

1. A `Session` needs a stable identity independent of a terminal tab title.
2. A recovery record needs both a runtime locator and a conversation locator:
   - runtime: iTerm window/tab/session or tmux target;
   - conversation: Codex/Claude/OpenCode session ID and source file.
3. Recovery status must be multi-dimensional:
   - `layout_restored`;
   - `process_started`;
   - `conversation_resolved`;
   - `context_verified`;
   - `blocked`;
   - `deferred`.
4. The UI must not show one green "restore pass" badge when only layout counts passed.
5. Session lookup should search transcript content and cwd when the launch command has no stable ID.
6. The system needs duplicate ownership protection before resuming a historical session.
7. Update prompts need a policy state and a retry/deferred queue.
8. A recovery action must be scoped to one target session whenever a full rebuild is unnecessary.
9. The final report must retain evidence paths, not only a prose success message.

## 9. Source Evidence

- Snapshot manifest: `/Users/jyxc-dz-0100609/iterm-tmux-snapshots/20260928-170427/manifest.json`
- Latest restore report: `/Users/jyxc-dz-0100609/iterm-tmux-snapshots/20260928-170427/restore-reports/restore-report-latest.json`
- Workspace restore README: `/Users/jyxc-dz-0100609/workspace-restore/README.md`
- Workspace restore retrospective: `/Users/jyxc-dz-0100609/workspace-restore/recovery-retrospective.md`
- Restore implementation: `/Users/jyxc-dz-0100609/workspace-restore/bin/restore-from-snapshot.py`
- Codex history index: `/Users/jyxc-dz-0100609/.codex/history.jsonl`
- Codex session index: `/Users/jyxc-dz-0100609/.codex/session_index.jsonl`
- Recovered session JSONL: `/Users/jyxc-dz-0100609/.codex/sessions/2026/07/16/rollout-2026-07-16T00-12-27-019f668d-0b84-78d1-a34f-cddd777cebd5.jsonl`
- Already-running `stepatlas` session JSONL: `/Users/jyxc-dz-0100609/.codex/sessions/2026/06/16/rollout-2026-06-16T19-12-12-019ed021-bcd6-7710-8ced-5063169cf652.jsonl`

