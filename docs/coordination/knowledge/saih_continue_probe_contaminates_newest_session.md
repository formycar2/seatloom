---
memory_id: saih_continue_probe_contaminates_newest_session
project_id: seatloom
topic_id: seatloom/operating-knowledge
kind: failure
confidence: inferred
status: active
tier: warm
source_kind: file
source_ref: saih/CONTINUATION-NOTE.md
l2_ref_kind: file
l2_file_path: saih/CONTINUATION-NOTE.md
l2_line_start: 264
l2_line_end: 282
curation: authored
l0_summary: `--continue` 不是只读的:它恢复该 cwd 下最新的会话并往里写。用驱动会话探测 `--continue` 语义时,探测的问答会被追加进"当时正好最新"的别人会话文件。已实际发生:两个外部回合写进了 operator 会话,一个写进了转移快照。
---

## 发生了什么

2026-10-17 下午,ttys008 上的旧会话(69855)用 `./bin/saih --continue -p "..."`
探测转移会话的恢复语义。`--continue` 恢复的是该 cwd 下**最新**的会话 ——
14:58 那会儿正好是 operator 正在用的会话(06-53)。于是两次探测的用户消息
("Reply with exactly: CONTEXT_OK" 14:58:06、"Use your ls tool …" 14:58:16)
被追加进了 operator 会话的文件;14:59:41 又一次追加进了 `~/.saih` 下的转移快照。
operator 会话的上下文没被污染(它只认自己回合),但文件里多了不是 operator 发的 user 消息。

## 为什么会改变未来动作

- 探测 `--continue` / 会话语义之前,先 `ls -lat` 该 cwd 的会话目录,确认哪个文件最新;
  不要在还有别人活跃会话的 cwd 里探测。
- 被污染的会话文件**不能改**:harness 是追加写,在它下面重写文件会让后续回合写进旧 inode。
  记录在案,留给未来 `--continue` 的人知道上下文里为什么有陌生问答。
- 任何"读一下会话库"的操作都要先当"会写"来对待。

## 证据

- operator 会话文件(`~/.saih/agent/sessions/--…-seatloom-saih--/2026-10-17T06-53-08-185Z_….jsonl`)
  第 5 行是 operator 的真实 prompt,第 35、37 行是两次外部探测追加的 user 消息。
- 转移快照(`~/.saih/…/2026-10-17T04-21-21-634Z_….jsonl`)尾部 14:59:41 多了一轮
  "what is the oldest topic…" + 回答。
- 69855 自己的转录显示它 14:58–14:59 连续跑 `--continue` 探测,15:00 得出正确命令是
  `--session <显式文件>`。
