# `disp-_TS3QA0Mg563KTzW2bi2b` — delivery: the heredoc rule, measured on myself first

- **Dispatch**: `disp-_TS3QA0Mg563KTzW2bi2b`, issued by seat-aegis-001, packet
  `AEGIS-2026-10-22-use-edit-not-heredoc-v1`.
- **What the packet asks**: install the rule "edit code with `edit`, never a `bash`
  heredoc", count my own heredoc usage, and say in the delivery note which tool this
  dispatch's code changes went through.
- **What this dispatch changes**: **no product code.** It is a rule-installation
  dispatch. The only file produced is this note, and it is a **new file written with
  the `write` tool** — not a `python3 - <<'EOF'` block.

---

## 1. The count the packet asks for — measured, on my own sessions

Source: my four session logs under
`~/.saih/agent/sessions/--Users-jyxc-dz-0100609-Documents-GitHub-seatloom-worktrees-flux--/`
(2026-10-16 → 2026-10-22). Parsed from the JSON tool-call records, not grepped out of
the raw text — see §3 for why the grep number is wrong.

| Session | bash | real heredocs | `edit` | `write` |
|---|---|---|---|---|
| 2026-10-16 | 465 | 29 | 5 | 20 |
| 2026-10-21 | 364 | 62 | 25 | 22 |
| 2026-10-21 (2nd) | 44 | 0 | 0 | 0 |
| 2026-10-22 (today, pre-dispatch) | 45 | 9 | 0 | 1 |
| **total** | **919** | **100 (11%)** | **31** | **43** |

**11% of my `bash` is heredoc — against the team's 58%.** Now the part that matters:
what those 100 heredocs actually *did*.

| Heredoc did what | count | Rule says |
|---|---|---|
| `open(<repo source path>, 'w')` — wrote into `crates/` / `src-tauri/` / `ui/src/` | **3** | ❌ this is the banned pattern |
| `open(/tmp/..., 'w')` — scratch data, deleted after | 10 | ✅ allowed |
| read-only analysis (parse jsonl, count, assert on strings) | 46+ | ✅ allowed |
| `cat` a fixture | 2 | ✅ allowed |

**So: 3 violations out of 919 `bash` calls (0.3%), all three in one session, all
three the same file.** They were the mutation probes I ran against
`crates/seatloom-mcp-emit/src/decisions.rs` — deliberately breaking Nimbus's source
to see whether the tests go red. That is Flux's core verification method and I am not
going to stop doing it; what I will change is *how the mutation is written*. All three
used the exact banned shape:

```
cd /Users/.../worktrees/nimbus && python3 - <<'EOF'
p='crates/seatloom-mcp-emit/src/decisions.rs'
s=open(p).read()
old='...'
assert old in s
open(p,'w').write(s.replace(old,new))
EOF
```

The correct tool for a mutation is `edit`: it takes the fragment, it shows a diff, and
it can dry-run before it touches anything. A heredoc has none of that — the mutation is
only visible by reading the whole command back.

All three ran clean (no `AssertionError`, no `Traceback` — I checked each paired
`tool_result`), and the tree is clean now: `git status` in the nimbus worktree is
empty, `git log -S flux_probe --all -- crates/` returns nothing, so **no mutation probe
was ever committed.**

## 2. The second number — `cd <absolute> &&` — is worse than it looks, and better

The packet says don't write `cd <absolute-path> &&` because `bash` already starts in my
own worktree. Measured: **485 of my 919 `bash` calls start with `cd <absolute>` (53%).**
Split by destination:

| Destination | count | Is it waste? |
|---|---|---|
| My own worktree (`worktrees/flux`) | **63** | **yes — pure waste**, cwd is already there |
| Other seats' worktrees (mira 235, nimbus 80, seatloom main 8, vela, onyx…) | **422** | **no — load-bearing** |

This is the seat-type difference the team-level number hides. Aegis's figure was
"~75k tokens, 8% of `bash`" across the team. For an implementation seat that number is
waste. **For Flux, 87% of it is the verification itself** — you cannot mutation-test
Nimbus's `decisions.rs` from inside my own worktree; the file is not there. The
instruction "your bash already starts in its own worktree, cwd is correct" is true and
I will stop prefixing it (63 calls, gone). But it should not be read as "a `cd` into
another tree is waste", or the next verifier will start believing its own
cross-tree evidence is a token leak.

The tool that *is* right for cross-tree work is `git -C <path>`, which I already use
14 times in today's session — it names the repo without a subshell `cd`.

## 3. A methodology error worth more than the rule (also found independently by Onyx)

The packet's acceptance says "grep your session for heredoc occurrences". I did that
first, and it is wrong in an instructive way:

```
$ grep -o "<<'EOF'\|<<EOF\|<<'PY'\|<<PY\|heredoc" <today's session> | sort | uniq -c
  29 <<'EOF'      4 <<'PY'      3 <<EOF      3 <<PY     183 heredoc
```

**183 hits.** Real heredoc redirection operators in that same session: **46**, of which
only **9** were tool calls I actually issued. The gap is because **my own analysis
command contains the pattern I am searching for** — the `grep` is a substring match
against text that includes its own pattern, plus every mention of the word "heredoc" in
the transcript doubles the count. Counting a tool by grepping for it from inside that
same tool is self-contaminating.

The fix is to parse the session JSON and look at the `arguments.command` of real
`toolCall` blocks, matching the redirection *operator* `<<WORD` — and then classify
what the body does, which is the part that actually decides whether a heredoc was a
violation. Onyx found the same trap on the same dispatch and recorded it as a
`failure`. I am recording it too because I found it separately and the fix differs
(mine: pair each `toolCall` with its `tool_result`, which is also how I checked that
no mutation ever raised).

## 4. Two things I found while measuring that are not mine to fix

**4.1 — the rule is not on my branch.** `docs/coordination/ENGINEERING_STANDARDS.md` on
`packet/flux/intent-verdict` has no "Tool use and token cost" section at all. The
section exists on `track/infra-foundation` (landed in `c7f4e06`, the same commit that
dispatched this packet) — my branch is 252 commits behind track. **The standard that
this dispatch points at as the shared source is not in the tree I work in.** I am
following the packet text verbatim, which is identical, but the next seat that reads
the standard from its own branch may find nothing there.

**4.2 — the docs gate is red on my branch, and it is not grandfathered.**
`scripts/verify-docs.sh <merge-base>` reports **38 of my changed docs failing the
universal-header check** (§2 of `DOCUMENT_TEMPLATES.md`), and `.gitlab/docs-grandfather.txt`
lists **zero** of them — the grandfather list's newest `tasks/flux` entries are from
May. Every delivery note I have written since October fails the gate. That is a real
blocker for merging this branch, but the fix (add them to the grandfather list, or
retro-fit 38 headers) is a governance decision about authority docs, not a verifier's
unilateral edit. Reported, not acted on.

## 5. Compliance for this dispatch

- Code changes: **none** (rule-installation dispatch, no code to change).
- This note is a **new file, written with `write`**.
- The measurement in §1–§3 is **read-only `bash`** (parse jsonl, count) — the
  "analysis-only" row above, the category the rule explicitly allows.
- For the *next* code-changing task, mutations and source edits go through `edit`,
  not a heredoc; `cd` into my own worktree is dropped; cross-tree work uses `git -C`.

## 6. Files changed

| File | Change |
|---|---|
| `docs/coordination/tasks/flux/disp-_TS3QA0Mg563KTzW2bi2b-delivery.md` | this note — **new file**, written with the `write` tool |

## 7. Forward records

Recorded via `remember` (two rows; `docs/coordination/FORWARD_RECORDING_PROTOCOL.md`,
the file the protocol reference names, **does not exist in this repo on track or
main** — I did not invent a copy of it, and recorded directly with the typed fields
the protocol specifies):

- `mem-nhpmZ5TQWYkLyloMn-ZaM` — `failure` — counting a tool's usage by grepping for
  it from inside that same tool inflates the count with its own search pattern (§3).
- `mem-PtXEwEcNfkciQd5OJpJbq` — `fact` — Flux's `cd <absolute>` is 87% cross-tree
  verification work, not waste; the team-level 8% figure must not be applied to
  verifier seats (§2).
- `unresolved` — 0. The two §4 items are reported in this note, not recorded as
  unresolved memories: they are Aegis's to decide, and "another seat must rule" is
  not the same as an open question I own.
- Two probe rows (`mem-msrx_BSTcusObswyJsbqI`, `mem-FFZzd8Y9i9tqZVzP7lwM8`) were
  written to find a topic that accepts writes and then soft-forgotten with the reason
  recorded; the invented topic `seatloom/operating-knowledge` from the knowledge docs
  is **not** a real topic in the live DB.
