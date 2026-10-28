# Delivery: reconnecting every seat on every menu switch was a defect, not a design

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-16-live-seats-reconnect-delivery-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-16 |
| version | v1 |
| dispatch | disp-cEyS0pTJRZaC3b64vFUdR |
| tags | live-seats, reconnect, defect, store, delivery, mira |

---

## 0. The one-paragraph version

The operator asked, in person, why switching menus and coming back to Live Seats
made him Connect every seat again. It was not by design. `LiveSeatsView` is
conditionally rendered by `App.tsx`, so leaving the tab unmounted it — and the
attachments lived in the component's own `useState`, so they evaporated while the
backend sessions kept running. The durable state now lives in
`useLiveSeatsMirrorStore`, a remount restores what the backend still holds, and the
mount-time reconcile drops anything that died while the view was away. The
deliberate first-seat auto-attach is preserved and no longer fires a second attach
on top of a live mirror.

## 1. Root cause, re-verified rather than taken on trust

The dispatch said "I traced it to file lines, verify it yourself, don't trust me."
I did. All three parts hold, and one part of the dispatch's premise does not:

| Claim | Verified |
|---|---|
| `App.tsx:395` conditionally renders `<LiveSeatsView />` | ✅ `if (activeTab === 'live-seats') return <LiveSeatsView />;` |
| `LiveSeatsView.tsx:63` held attachments in component state | ✅ `const [attachments, setAttachments] = useState<Attachment[]>([])` |
| Remount auto-attached only `seats[0]`, once per mount | ✅ `autoAttachedRef` is a `useRef(false)` — it resets on every remount |
| Unmount does not kill the backend sessions | ✅ the cleanup at :112/:144/:145 clears an interval and two listeners; nothing touches the backend |
| **`api.listLiveSessions()` is ready to be called** | ⚠️ **it exists, but it cannot do the job the dispatch's preferred option asks of it** — see §2 |

## 2. Why the preferred option was not taken (measured, not preferred)

The dispatch's first choice was: on mount, call `api.listLiveSessions()` and
re-attach what is still alive. It cannot work, for two reasons found by reading the
backend:

1. **`cmd_list_live_sessions` returns `Vec<String>`** — session ids only
   (`src-tauri/src/commands/session_cmds.rs:366`). Rebuilding the view needs the
   seat↔session mapping *and* the `LiveSessionDto` (pane geometry, snapshot path,
   tmux session name) that the terminal renders from. An id alone reconstructs
   none of it.
2. **Re-attaching would be worse than doing nothing.**
   `cmd_attach_tmux_session` inserts unconditionally
   (`session_cmds.rs:247`, `state.sessions.lock().await.insert(id.clone(), session)`).
   A second attach for the same tmux pane starts a second `tmux pipe-pane` and
   overwrites the registry entry — a leaked pipe-pane and a changed backend session
   lifecycle, which this dispatch explicitly forbids ("do not touch the backend session
   lifecycle").

So the attachments are hoisted instead, and `listLiveSessions()` is used for what it
*can* honestly do: a mount-time **filter**, dropping sessions that exited while the
view was away (the `session:exit` listener only runs while mounted, so that moment is
the only chance to catch up).

This is also the pattern the codebase already proves works: `useLiveSessionsStore`
holds `sessionBySeat` across unmounts today, which is how the Supervisor IM keeps
routing chat into the right pane.

## 3. What was built

| File | Change |
|---|---|
| `ui/src/stores/useLiveSeatsMirrorStore.ts` | **new**. The durable mirror state: `attachments`, `activity`, `activeName`, and the `autoAttached` flag — plus `addAttachment` / `removeAttachment` / `addBytes` / `pruneDead` / `markAutoAttached` |
| `ui/src/views/LiveSeatsView.tsx` | reads and writes the store instead of local `useState`; three new effects (mount reconcile, routing re-assert, corrected auto-attach). No visual or behavioural change other than the fix |
| `ui/src/views/LiveSeatsView.test.tsx` | **new**. 7 assertions, mounting the real view with a mocked Tauri layer |

Three behaviours the fix has to hold, all pinned:

1. **Connections survive.** Unmount, remount → same attachment count, zero new
   attach calls, and the only Connect button left is for a seat that was never
   attached.
2. **Counters survive.** Bytes fed through the real `session:output` listener are
   still there after a remount, byte-for-byte.
3. **The deliberate design survives.** On a genuinely first open the first
   discovered seat is still auto-attached (`{tmuxSessionName: 'Lyra-lyra-seatloom',
   seatId: 'lyra'}`), and a remount that finds live mirrors performs **no** extra
   attach — the flag is durable now, so the old second `pipe-pane` cannot happen.

Two honesty guards, because a restore that guesses is worse than the defect:

4. **A session that exited while away is dropped.** The reconcile compares against
   the live id set; a dead mirror is not shown as connected.
5. **A failed list does not cost the operator his mirrors.** `listLiveSessions()`
   rejecting is swallowed deliberately — losing live panes on a transient error
   would be a worse defect than the one being fixed.

Plus: the seat→session routing the Supervisor IM depends on is re-asserted on mount
(idempotent, repairs a partial write).

## 4. Boundaries

- No interface skeleton was re-laid-out. `LiveSeatsView`'s markup is unchanged.
- No backend session lifecycle was touched — no command was added, changed or
  called differently. The only backend interaction is a read that already existed
  and was never called from this view.
- No other view was changed. The new store is imported by `LiveSeatsView` only;
  `useLiveSessionsStore` is byte-for-byte untouched.

## 5. Gates

| Gate | Result |
|---|---|
| `tsc --noEmit` (whole tree) | clean |
| `vitest` | **100/100** (7 new) |
| `scripts/lint-changed.sh track/infra-foundation` | eslint 0 errors, prettier clean |
| `vite build` | green |
| `cargo test --workspace` | 251 passed / 0 failed (no Rust changed) |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `scripts/verify-docs.sh` | this note clean; the 2 reds are the two pre-existing untracked scratch files, not this dispatch's work |

## 6. What is not proven

Nobody has clicked through the running app. The evidence is that the real view,
mounted with a mocked Tauri layer, keeps its attachments across an unmount/remount
and does not re-attach — which is the mechanism the defect lived in. What the test
cannot show is the operator's actual experience: that switching menus and coming
back leaves six connected seats with their counters intact. His click is the first
look, and it is what the acceptance card asks for.

## 7. Artifact index

- Branch: `packet/mira/knowledge-panel`.
- Ledger: see `emit_event(ArtifactCreated)`.
- Acceptance card: `acc-disp-cEyS0pTJRZaC3b64vFUdR`.
