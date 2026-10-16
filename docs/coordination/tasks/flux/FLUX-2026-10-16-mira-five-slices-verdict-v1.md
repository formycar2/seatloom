# Flux verdict — Mira's five 2026-10-16 slices, re-verified against each dispatch's own criteria

- **Dispatch**: `disp-Tn-83tjvS_B6YgOV4dCCF` (issued by seat-lyra-001). A re-verification: five
  slices were green-lit on an invalid basis and two of them are already in `track`.
- **Verdict event**: `ev-0idwakl17abEU75LNED24` — **pre-declared id, emitted exactly as required**
  (`ReviewVerdictIssued`).
- **Verdict**: **PASS — all five, each against its OWN dispatch's acceptance criteria, measured.**
  Two findings are raised that are not defects in any of the five slices: the acceptance gate that
  let them through, and one ordering rule the backend does not honour.
- **Verified commits** (repeated in every finding below):
  - item 1 history UI: `51bccc5` on `packet/mira/knowledge-panel`, **merged into track @ `4cfd230`**
  - item 2 app icon: `d04f373` on the same branch, **merged into track @ `4cfd230`**
  - item 3 live-seats reconnect: `18aa027` on the same branch, **not in track**
  - item 4 supervisor feed: `62e535a` on the same branch, **not in track**
  - item 5 acceptance count: `f34111d` on the same branch, **not in track**
  - track @ `8a2d733`; the merge's first parent (pre-merge track) @ `09d7abb`; branch tip `93226cb`
- **Where I worked**: my branch is `packet/flux/intent-verdict` @ `6b5d67d` and has none of this
  code, so I verified in the two checkouts that hold it — the main checkout (on `track`, tracked
  tree clean) for the two merged slices, and the mira worktree (on `packet/mira/knowledge-panel`,
  tracked tree clean) for the three branch slices. **Read-only in both**: I committed nothing and
  changed no tracked file. The live database was touched only by read-only `SELECT`s and by
  read-only MCP tool calls whose no-write property I measured.

---

## §14 captures (COORDINATION_RULES §14)

```
$ git -C <flux worktree> rev-parse --abbrev-ref HEAD  -> packet/flux/intent-verdict
$ git -C <flux worktree> rev-parse HEAD               -> 6b5d67d
$ git -C <flux worktree> status --short               -> ?? .scratch/*, .seatloom/rollout-ack.txt (pre-existing untracked; tracked tree clean)
$ git -C <main checkout> rev-parse --abbrev-ref HEAD  -> track/infra-foundation
$ git -C <main checkout> rev-parse HEAD               -> 8a2d733
$ git -C <main checkout> status --short               -> (clean)
$ git -C <mira worktree> rev-parse --abbrev-ref HEAD  -> packet/mira/knowledge-panel
$ git -C <mira worktree> rev-parse HEAD               -> 93226cb
$ git -C <mira worktree> status --short               -> ?? two pre-existing docs (pre-existing untracked; tracked tree clean)
```

---

## 0. The background claim, verified independently before judging any slice

The dispatch's premise — I checked it in the ledger rather than taking it on trust:

| Claim | Measured |
|---|---|
| five cards, all citing the same verdict ref | **true.** `acc-disp-history-sessions-ui`, `acc-disp-ol420Y586tNIsbbeh5nlU`, `acc-disp-cEyS0pTJRZaC3b64vFUdR`, `acc-disp-QBVu3LeruaTavYLIN5BCI`, `acc-disp-CEa11raWfi8W_UHiXGKBw` — all five carry `flux_verdict_ref = ev-verdict-history-sessions` |
| that verdict is unrelated to four of the five | **true.** `ev-verdict-history-sessions` is a `ReviewVerdictIssued` with `verdict = fail` about the 2026-10-14 historical-session corpus. It says nothing about an app icon, a live-seats reconnect, a supervisor-feed translation, or an acceptance count. |
| two of them were accepted on that basis and merged | **true.** the two `accepted` cards are `acc-disp-history-sessions-ui` and `acc-disp-ol420Y586tNIsbbeh5nlU`; their code is in track @ `4cfd230` |
| the tool only checked existence, not relevance or value | **true**, and the ledger proves it twice over: the same failing verdict satisfied the gate for five unrelated cards, and the three later cards were voided with `feedback` naming exactly this ("flux_verdict_ref 引用错误,卡不该存在") |
| the merge brought only those two slices | **true.** `git diff --name-only 09d7abb 4cfd230` = 70 files; the union of `51bccc5`'s and `d04f373`'s file lists = the same 70, `comm` empty both ways. Nothing else rode in. |
| track's copy is the merged copy | **true.** for all 11 source files the two slices touch, `git rev-parse track/infra-foundation:<f>` == `git rev-parse packet/mira/knowledge-panel:<f>` — byte-identical. So verifying track verifies exactly what the operator is running. |

**So the green-light basis was invalid. The work is a separate question, and that is what I measured.**

---

## 1. Item 1 — `disp-history-sessions-ui` (history sessions UI, `51bccc5`, **merged**) — **PASS**

Criteria: the contract's §8.2 slice 3 + §4 screen definition + §3.2 empty states + the operator's
arithmetic + no hardcoded wiring flag + a measured zero-new-issues result.

### Gates, run by me on track (not taken from the delivery)

| Gate | Result |
|---|---|
| `pnpm typecheck` (whole tree) | **clean** |
| `pnpm test` (vitest) | **93 / 93 passed** (21 of them the new `HistorySessionsPanel` tests) |
| `scripts/lint-changed.sh 09d7abb` | **clean** — 6 changed files, eslint 0 errors, prettier clean |
| `pnpm build` (vite) | **green** |
| `cargo test --workspace` | **251 passed / 0 failed** |
| `cargo clippy --workspace --all-targets -- -D warnings` | **clean** |
| `cargo fmt --all -- --check` | **one red, pre-existing** — `src-tauri/src/main.rs:88`, the HOME-fallback block's indentation |

That fmt red is real and I verified it is not hers: `git show 09d7abb:src-tauri/src/main.rs`'s
`if let Some(home) …` block is **byte-identical** to HEAD's, and `51bccc5`'s `main.rs` diff is
**14 insertions, 0 deletions**. Her delivery states this; it holds.

### The arithmetic, measured twice — once against live, once against the source corpus

Live, read-only, driving the same binary the app spawns:

```
list_history_projects -> projectCount 38 / sessionTotal 183 / messageTotal 36680 / corpusLoaded true
per-project sessionCount summed over all 38 projects = 183
per-project messageCount summed over all 38 projects = 36680
OfficeBench 49 · seatloom 14 · (root) 8 · cursor 14, ALL 14 undated
harness split: codex 150 / claude-code 18 / cursor 14 / stepcode 1
```

Independently of the database, from `resume-contexts/`: **183** `by-session/*.md` files,
**183** data rows in `SESSIONS-FLAT.csv`, the per-file `消息数` headers summing to **36680**, and
**38** distinct home paths. Both sides agree with the operator's card.

### Ordering, measured across all 38 projects

```
projects whose dated block is NOT newest-first:            none
projects where a dated row appears after an undated one:   none
```

`seatloom`'s 14 rows are the concrete case: 13 dated, newest first, then one `cursor` row at the
bottom flagged undated with 0 messages.

### The line numbers are real, not decorative

For session `621ae382-d6c1-4f0c-b3fa-7c1155a5ecb8` (claude-code, 75 turns, `countsAgree: true`,
declared 75 == stored 75):

```
turn 0 says original line 7
  original file line 7  -> '{"parentUuid":"dc20cecb-…","isSidechain":false,"prompt…'   (the first prompt)
  context file line 13  -> '## [原文件第 7 行] 你'
```

### Everything else on the checklist

- **No new route / window / top-level nav.** `SessionsWorkspace` is mounted at the existing
  `sessions` tab (`ui/src/app-v2/AppV2.tsx:97`, ⌘2) — the same place as before. The three history
  screens are an in-tab two-mode shell.
- **No mock.** `history-measured.json` is imported by the **test file only**; the app's
  `liveLoader` calls `api.listHistoryProjects / listHistorySessions / getHistorySessionTurns /
  readHistoryContextSlice / openLocalPath`, which invoke the Tauri commands, which shell out to
  the MCP tools.
- **No hardcoded wiring switch.** A test reads the panel's source and forbids the
  `…_REGISTERED = true|false` shape, and asserts the panel goes through `api.*`.
- **Every §3.2 empty/missing state is present with why + what is next** — not-in-Tauri, read
  failure (the backend's own text, verbatim), corpus not loaded ("历史会话还没加载" + apply 021, and
  explicitly no empty project list), no project selected, project with no sessions, no session
  selected, empty turns (`emptyReason`, no empty frame), source file gone (line numbers survive,
  content does not), `countsAgree: false` (a defect, not rounding), `countsAgree: null` (one side
  absent), undated sessions, the root group, zero-message sessions, `messageCount == null`.
- **The root group** is labelled `未归类(仓库根目录)`, carries the reason ("stripped 之后是空串…
  合进邻居或者直接藏掉都是一种猜测,这里两种都不做"), and is **moved to the very end** by the panel.
- **The acceptance card's `acceptance_plan` is the contract's §6 verbatim** — the scenario, all
  eight steps with their 看/算对/打回 text, and the two preconditions.
- **The tmux mirror really did move verbatim.** Her delivery's `diff` command is now stale (it says
  `track/infra-foundation`, which post-merge is the *new* shell), so I re-ran it against the correct
  base `09d7abb`: a character-level diff shows the only edits are prettier's redundant parentheses
  and trailing commas. Semantically identical.

### One observation, not a failure (F-OBS-1)

The contract §4 says the project list is "ordered by session count descending … **the
repository-root group appears last**". The **backend** `list_history_projects` does not do the
second half — it returns `(root)` at **position 6 of 38**, because it orders purely by session count
and the root has 8. The **UI** compensates (`[...rest, ...root]`), so the operator-facing criterion
is met and this is not a defect in the slice. But the contract's ordering rule now lives in exactly
one of the two surfaces, so any other consumer of that tool gets an order the contract forbids.
Worth a line to Lyra; not a reason to fail the slice.

---

## 2. Item 2 — `disp-ol420Y586tNIsbbeh5nlU` (app icon, `d04f373`, **merged**) — **PASS**

Criteria: a 1024 master, the full set from `tauri icon`, `bundle.icon` filled with more than one
file, a release build proving the `.icns` is in the bundle, recognisable at 16×16, no
photo/gradient/text, and one sentence on what it is and why.

| Criterion | Measured |
|---|---|
| 1024×1024 master with alpha | `src-tauri/icons/icon-1024.png` — `sips` reports 1024×1024, `hasAlpha: yes` |
| full set from the official tool, not hand-sized | `scripts/gen-app-icon.sh` runs `./node_modules/.bin/tauri icon … -o "$ICONS_DIR"` |
| `bundle.icon` is a real set | **5 entries**: `32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.icns`, `icon.ico` — not one file |
| the `.icns` really holds the tiers | parsed the icns directory: `is32`(16) `ic11`(32) `ic12`(64) `ic13`(128) `ic14`(256) `ic07`/`ic08`/`ic09`/`ic10`(128–1024) — all seven size tiers |
| the `.ico` too | 6 entries: 16 / 24 / 32 / 48 / 64 / 256 |
| **the release build proves it** | `target/release/bundle/macos/SeatLoom.app/Contents/Resources/icon.icns` exists and is **md5-identical** to `src-tauri/icons/icon.icns`; `Info.plist`'s `CFBundleIconFile` is `icon.icns`; and the bundle's binary contains `cmd_list_history_projects` etc., so it **is** the post-merge code, not a stale build |
| **recognisable at 16×16** | I decoded the actual 16×16 pixels out of the `.ico` and rendered them. See below. |
| no white border | **0** white pixels on the outer 1-px ring at 32×32, 128×128 and 1024×1024; corners are rounded (95.8–97.7 % opaque) |
| no photo / gradient / text | exactly **two** colours in the whole image — `(47,111,235)` the brand blue and `(255,255,255)` white — plus antialiased edge pixels |
| one sentence on what it is and why | delivery §1, verbatim: *"the same brand blue the product already uses (#2F6FEB), with the letter swapped for a geometric mark that still reads at 16 pixels: three thick white bars sweeping in from the left and merging into one thicker vertical bar — the three bars are the working seats, the bar they merge into is the one place their records come together."* |

### The 16×16 judgement, by looking

The 16×16 pixels, decoded from the `.ico` and rendered (`#` = the white mark, `.` = the blue
ground):

```
 ..###...###.....
 ...####.###.....
 .....######.....
 .......####.....
 ...########.....
 ...########.....
 .......####.....
 .....######.....
 ...####.###.....
 ..###...###.....
```

It is **not a blob**. There is a solid vertical trunk in the middle, three arms sweeping in from the
left at three heights, and the blue ground showing through the gaps between them — the area-averaged
supersampling did its job, where point sampling would have closed those gaps. The mark is
mirror-symmetric about the horizontal axis, so it will not look crooked when the Dock shrinks it.

**Honest caveat, stated rather than papered over:** at 16 px you read *"a connected three-armed
geometric mark"*, not *"seats merging into one record"*. That is the correct trade-off for 16 px —
a pictogram would have smeared — and it is exactly the trade-off the dispatch asked to be explained.
Whether it is **good-looking** is Mr. Zhang's call, as the dispatch said; what I can measure is that
it is still a legible connected shape at the Dock's smallest size, and that it is his product's own
blue rather than the system default.

---

## 3. Item 3 — `disp-cEyS0pTJRZaC3b64vFUdR` (live-seats reconnect, `18aa027`, **branch**) — **PASS**

Criteria: restore existing connections on remount instead of from scratch; the connection count,
byte counters and activity state continue rather than reset; the deliberate first-seat auto-attach
must not regress.

### What was built, and why it is not the dispatch's preferred option

The durable state — the seat↔session mapping, the byte/chunk counters, which pane is on screen, and
the auto-attach "once" flag — moved out of the component's `useState` into a new
`useLiveSeatsMirrorStore` that outlives the view. `api.listLiveSessions()` is used for the one thing
it can honestly do: a **mount-time filter** that drops attachments whose session exited while the
view was away (the `session:exit` listener only runs while mounted, so that is the only moment to
catch up). A failure to list is swallowed on purpose.

Her claim that the dispatch's *preferred* fix could not work is **verified in the code**, not a
matter of taste:

- `cmd_list_live_sessions` (`session_cmds.rs:366-369`) returns `Vec<String>` — **session ids only**.
  It cannot rebuild the seat↔session mapping or the `LiveSessionDto` the terminal renders from.
- `cmd_attach_tmux_session` (`session_cmds.rs:187`) spawns a **new `tmux pipe-pane`** and
  (`session_cmds.rs:247`) **inserts unconditionally**. Re-attaching would start a second pipe-pane on
  the same pane and leak the first — and would change backend session lifecycle, which the dispatch
  explicitly forbade.

So the alternative the dispatch offered ("hoist `attachments` into a global store") is what was
taken, and the preferred one is measurably impossible.

### The tests are real, and they assert the acceptance card's three things

Seven new tests mount the **actual `LiveSeatsView`** with a mocked Tauri layer:

1. connections survive an unmount/remount with **zero new attach calls** (`attachCalls.length` stays
   2, and only one Connect button is left to press);
2. the byte counters survive — `toEqual` on the whole `Activity` object, so `bytes`, `chunks` **and**
   `lastAt` all persist;
3. a genuinely first open still auto-attaches the first seat
   (`attachCalls == [{tmuxSessionName: 'Lyra-lyra-seatloom', seatId: 'lyra'}]`);
4. **no second attach** on top of a mirror that is still live (this is the duplicate-pipe-pane
   regression the old `useRef` flag allowed);
5. a session that exited while away is **dropped**, not shown as connected;
6. a live-session list that cannot be read **keeps** the mirrors;
7. the seat→session routing the Supervisor IM depends on is re-asserted.

### Gates, run by me on the branch

`tsc` clean · vitest **159 / 159** · `lint-changed.sh 4cfd230` clean (10 files), and clean again
per-slice against each commit's own parent · `vite build` green · `cargo test --workspace`
**251 / 251** · clippy clean · the one pre-existing `main.rs:88` fmt red, inherited from track.

### One observation, not a failure (F-OBS-2)

While the view is unmounted its `session:output` listener is removed (`LiveSeatsView.tsx:151`), so
bytes that arrive while the operator is on another tab are **not** added to the counters. The
dispatch's criterion is "之前看到的字节数和活动状态还在,不是归零重来" — and they do persist, so this
passes. But "moving" is derived as `now - lastAt < 2500ms`, so after a short absence a seat correctly
stops reading as active (no bytes arrived), and the terminal shows the attach-time snapshot plus
whatever arrives after the return, not a replay of what was missed. Replaying it would need backend
scrollback, which is outside this dispatch's boundary ("不动后端会话生命周期"). Recorded so nobody
later mistakes it for a regression.

---

## 4. Item 4 — `disp-QBVu3LeruaTavYLIN5BCI` (supervisor feed human text, `62e535a`, **branch**) — **PASS**

Criteria: the 16 missing types translated to Chinese; carry the meaningful fields rather than ids;
**do not invent** anything the payload does not carry; keep the emoji prefix style; do not change the
sorting; do not touch the contact list; decide the 9 dead branches and say why.

### Both vocabulary claims, verified through the tool rather than believed

- **All 16 missing types are in the closed vocabulary** — `query_ledger` accepts each of
  `SeatDispatchIssued/Delivered/Completed/Failed/Retried/Cancelled`, `AcceptanceRequested/Granted/
  Rejected/Cancelled`, `MemoryCreated/Recalled/Promoted/Forgotten/Resolved`, `TopicOpened`.
- **The 9 switch-only types are not** — `query_ledger` **rejects each by name**:
  `SessionCompleted`, `SessionFailed`, `SessionInterrupted`, `HandoffDrafted`, `HandoffReceived`,
  `HandoffWorking`, `HandoffReturned`, `CheckpointCreated`, `PromptDetected`. Those branches could
  never match, so removing them is correct rather than lossy.

### My own measurement of the defect and the fix

`query_ledger`'s hard ceiling is **200** rows, so I measured 200 where she measured 400 — stated so
the number is not overclaimed:

```
composition of the 200 newest rows: MemoryCreated 46, SeatDispatchIssued 33, SupervisorMessage 23,
  ArtifactCreated 22, SeatDispatchDelivered 21, SeatDispatchCompleted 19, TopicOpened 10,
  SeatDispatchCancelled 9, AcceptanceRequested 5, AcceptanceGranted 5, AcceptanceCancelled 3,
  ReviewVerdictIssued 2, SeatDispatchFailed 1, MemoryForgotten 1

rows the OLD switch would have rendered as a bare `• Type`:  153 / 200  (76%)
rows the NEW translator still leaves to its default branch:  0
```

76 % corroborates her 80 % and the dispatch's 79 %; the ledger simply moved between measurements.
Every type present now has a dedicated line.

### The rest of the checklist

- **Meaningful fields, not ids.** A dispatch line says which work (`《title》`), to whom (the seat's
  display name), and what status (待认领 / 被卡住 / 进行中 / 已完成). An acceptance line carries the
  card's own `scenario`. A memory line carries the memory's **title**, resolved through a bounded
  second pass that fetches only the topics the loaded events actually mention.
- **Nothing invented.** Every field comes from the event's own payload or from a read the caller
  actually performed and handed in via `EventTextContext`. Every miss is *stated*:
  `《{id}》(这个派单的标题没取到)`, `(这条没记是哪个活)`, `(没记原因)`. A `listDispatches` failure
  leaves titles unresolved rather than blanking the feed. `AcceptanceGranted`'s payload carries no
  dispatch id and no refs, so the line says what it knows instead of stripping `acc-` off an id to
  derive one — the one place a guess would have been easy, and it was refused.
- **A real person's words are untouched.** `SupervisorMessage` / `SeatResponse` return
  `payload.content` verbatim, no emoji prefix, pinned by two byte-for-byte tests.
- **The emoji prefix style is preserved**, and a test asserts at least one emoji per category.
- **Sorting unchanged.** The rows come from `api.listSupervisorMessages` in the backend's order and
  are mapped in that order; there is no `sort()` on the message list anywhere in the diff.
- **The contact list is untouched.** No hunk in the diff covers `useContacts`; the only edits to the
  file are the two `any` tightenings the ratchet requires and one formatting wrap.
- **Scope.** One hook, one new lib file, two test files. No other view, no backend.

---

## 5. Item 5 — `disp-CEa11raWfi8W_UHiXGKBw` (pending-acceptance count, `f34111d`, **branch**) — **PASS**

Criteria: `isDecided` must know `cancelled`; the count must stop counting voided cards; voided cards
must not appear where the operator is asked to decide; accepted/rejected behaviour must not
regress; look for the same bug's second instance.

### The fix

`isDecided` is now `accepted || rejected || cancelled`; a new `isCancelled`; and `cancelled` finally
gets a status **label** (`已作废`) instead of falling through to the raw string. The awaiting list
(`!isDecided(status) && !dismissed`) therefore excludes voided cards, and they are rendered in their
own **collapsed** `已作废 N 项` area with the reason from `feedback` — visible, never counted. A card
with no reason points at the ledger's `AcceptanceCancelled` event rather than showing nothing.

The judgment call (show, collapsed, with the reason — not hide) is explained in the delivery: he
counts, so hiding them changes the total again with nothing to reconcile against, and he has asked
"where did that card go" before.

### The second instance, found and fixed

`DecisionResult` derived `accepted`/`rejected` from status **or** `mergeCommit`/`reworkDispatchId`,
so a cancelled card matched neither and would have rendered an empty result strip. It now has an
explicit third branch — unreachable after the main fix, but impossible rather than merely unused. I
checked the rest of the tree myself: the only other `'accepted'` gates are
`SupervisorIntent.tsx:177` and `HandoffDetail.tsx:28`, both **handoff** statuses — a different
domain, exactly as she said.

### The number, re-measured on live — and it has moved again

```
live acceptance cards right now:  accepted = 6   cancelled = 5   pending = 0
```

The dispatch's criterion was "算对:是 5,不是 7", measured earlier today. **The live set has moved
since** (the three remaining cards from this very batch were voided while the work was in flight,
which is why `cancelled` is now 5). So the correct answer **today** is **0 awaiting decision, 5
voided, 6 settled** — and that is what the fixed code produces. With the bug still in place the
section would read **5** phantom items. The arithmetic of the fix is right in both worlds; only the
inputs changed.

### One thing honestly not worked around

The voided area shows the card's **creation** time, not the void instant, because
`AcceptanceCardDto` deliberately drops `decided_by`/`decided_at` (`src-tauri/src/dto.rs:417-418`
destructures them as `_`) and `ui/src/lib/types-dto.ts` is Nimbus's fence. Verified in the code. The
label says what the time is, and the void instant is in the ledger's `AcceptanceCancelled` event,
which the supervisor feed now renders. This is a real limitation, stated rather than hidden.

### Gates, run by me on the branch

Same set as item 3 — all green; the two touched test files pass **35 / 35**.

---

## 6. Raised, not failed on

### F-GATE-1 — the acceptance gate let five unrelated cards through on one failing verdict (Aegis / Lyra)

This is the defect that made the green light invalid, and it is **not** a defect in any of the five
slices. Measured: five cards on five different dispatches all carry
`flux_verdict_ref = ev-verdict-history-sessions`, a `ReviewVerdictIssued` whose `verdict` is
**`fail`** and whose subject is the 2026-10-14 historical-session corpus. The gate checked that the
referent **exists** and stopped there — not that it is about this dispatch, and not what it
concluded. Two of the five were accepted on that basis and merged; the other three were later voided
with `feedback` naming exactly this root cause. **Nimbus is fixing the two defects; the dispatch says
so.** I am not rolling anything back — that is Aegis's and Mr. Zhang's call, per the dispatch's own
boundary. What I am recording is that the *code* in track is verified and the *process* that put it
there was not.

### F-OBS-1 — the contract's "root group last" ordering lives only in the UI (Lyra)

`list_history_projects` returns `(root)` at position 6 of 38; the panel moves it to the end. The
operator-facing criterion is met, so item 1 passes — but the contract §4 rule is enforced in one of
two surfaces, and the next consumer of that tool will get an order the contract forbids.

### F-OBS-2 — bytes arriving while the live-seats view is unmounted are not counted (Mira / Aegis)

Noted under item 3. Persistence — the actual criterion — holds.

---

## 7. What I could not do, stated plainly

- **I did not click.** Every claim above is compile-level, unit-test-level, measured-read-level or
  file-level. I work from a shell and cannot see the interface. The three screens, the Dock icon at
  its smallest, the reconnect after a menu switch, the supervisor column and the 待验收 number have
  never been rendered for me. **Mr. Zhang's click is the first one.**
- **I did not build a fresh release bundle.** For item 2 I measured the bundle that exists on disk
  (`target/release/bundle/macos/…`, built 15:05, after the 14:54 merge) and proved it contains the
  post-merge commands and the byte-identical `.icns`. I did not run a new `tauri build`, which is
  Aegis's script anyway.
- **The supervisor-feed measurement used 200 rows**, `query_ledger`'s hard ceiling, not the 400 the
  column loads. The percentage is corroborating, not a re-measurement of her exact figure.
- **I did not verify the Tauri app runs.** `cmd_lineage`, the history commands and the icon wiring
  were compile-verified and string-verified inside the bundle binary; the app was not launched.
