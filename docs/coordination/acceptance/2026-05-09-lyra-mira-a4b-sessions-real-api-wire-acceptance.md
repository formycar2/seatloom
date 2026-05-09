# Acceptance: A4-β SessionsWorkspace Real-API Wire-Up — PASS @ `ab672e5`

[Lyra -> Mira, Flux] A4-β closed. Lyra acceptance with Flux Layer A PASS underneath. SG-A chapter fully closed on the UI side; SG-B entrypoint B1 now dispatched to Nimbus.

| Field | Value |
|---|---|
| template | T3 |
| subtype | acceptance |
| id | LYRA-2026-05-09-mira-a4b-sessions-real-api-wire-acceptance-v1 |
| status | accepted |
| author | lyra |
| date | 2026-05-09 (late evening) |
| to | mira, flux, aegis, mr-zhang |
| verdict | **PASS — UNCONDITIONAL** (Layer A + Layer B both cleared) |
| target commit | `ab672e53e2c55c93d1e13f8ce6b4cbdc07c546cc` |
| delivery doc | `docs/coordination/tasks/mira/MIRA-2026-05-09-a4b-sessions-real-api-wire-delivery-v1.md` |
| verify doc | `docs/coordination/tasks/flux/FLUX-2026-05-09-a4b-sessions-real-api-wire-verify-delivery-v1.md` |
| packet | `docs/coordination/tasks/mira/MIRA-2026-05-09-a4b-sessions-real-api-wire-v1.md` |
| tags | lyra, acceptance, A4-β, SG-A-closure, SG-B-unlock |

---

## 1. Verdict

**PASS — UNCONDITIONAL.** A4-β replaces the A4-α mock IPC surface (`MOCK_TMUX_SESSIONS` + `console.warn('A3 not yet available')`) with real calls to A3's `cmd_list_tmux_sessions` + `cmd_attach_tmux_session`. Scope held to 3 files. All wire shapes verified against Rust source. All invariants preserved (R1 attach-only, R3 failure-isolation, v0.0.1 read-only, cancellation safety, browser-dev degradation).

**Layer B runtime confirmed** by Aegis-relayed Mr. Zhang live-run 2026-05-09 late-evening:
- `cmd_list_tmux_sessions` → 5 real sessions returned ✓
- `cmd_attach_tmux_session` → `{ id, tmuxSessionName, fifoPath, runtime: 'tmux-mirror' }` DTO shape verified ✓

Full Layer A + Layer B closure — no deferred items remain for A4-β itself.

## 2. Three-layer gate evidence

| Layer | Owner | Result | Artifact |
|---|---|---|---|
| Delivery | Mira | commit `ab672e5`, 3 files / 56 ins / 42 del, tsc=0, pnpm build green | `MIRA-2026-05-09-a4b-...-delivery-v1.md` |
| Pre-flight audit | Lyra | scope clean, wire shape confirmed against Rust source, MOCK removed, banner/pill preserved | Recorded in MEMORY + Flux packet §5 pre-marking |
| Commit-pinned verify (Layer A) | Flux | 13/13 static invariants PASS, 5/5 R-rules PASS, tsc=0, pnpm build green | `FLUX-2026-05-09-a4b-...-verify-delivery-v1.md` |
| Runtime (Layer B) | Mr. Zhang (live-run relayed by Aegis) | **PASS** — cmd_list returned 5 real sessions; cmd_attach returned `{ id, tmuxSessionName, fifoPath, runtime: 'tmux-mirror' }` DTO | Relayed 2026-05-09 late-evening |
| Acceptance | Lyra | PASS — this document | This file |

## 3. Invariants held

### R-rule / compliance (from Flux §4)

| Rule | Evidence | Status |
|---|---|---|
| R1 attach-only | `cmd_launch_session` / `launchSession` both absent from `SessionsWorkspace.tsx` | PASS |
| R3 failure-isolation | `closeTab` at `SessionsWorkspace.tsx:72-78` uses local state only, no `api.killSession` call (note §4 non-blocker a below — it's a leak, not an R3 violation) | PASS |
| v0.0.1 read-only | `disableStdin: true` preserved at `SessionTerminal.tsx:66`; file not in A4-β diff | PASS |
| Browser-dev degradation | `!isTauri()` at `SessionsWorkspace.tsx:26` (list→[]), `:49` (attach→user-visible error) | PASS |
| Cancellation safety | `cancelled` flag at `:24`, guards at `:32`/`:34`, cleanup at `:43` | PASS |

### Wire shape cross-checks

- `TmuxSessionInfo` at `crates/seatloom-core/src/pty/mod.rs:113-119` has no `#[serde(rename_all)]` → snake_case fields. TS at `ui/src/lib/types-dto.ts:210-212` matches (`session_name` / `created_at` / `attached`). PASS.
- `AttachTmuxRequest` at `src-tauri/src/commands/session_cmds.rs:84` has `#[serde(rename_all = "camelCase")]`. TS envelope at `ui/src/lib/api.ts:146-153` passes `{ tmuxSessionName, rows, cols }` camelCase. PASS.
- `invoke('cmd_attach_tmux_session', { request })` at `api.ts:153` uses the `{ request }` outer-envelope pattern matching `cmd_launch_session`. PASS.

## 4. Known non-blockers (carry-forward)

Confirmed by Flux §5 and pre-marked in the verify packet §5 to prevent false-FAIL:

a. **Mirror leak on close-tab**: `closeTab` at `SessionsWorkspace.tsx:72-78` doesn't call `api.killSession`, so backend FIFO + tmux `pipe-pane` linger until app exit. This is A4-α legacy behavior — the `closeTab` wasn't changed by Mira and we explicitly scoped A4-β to the mock→real wire-up only. Not an R3 violation: A3 backend `kill_session` is R3-safe by design (verified at A3 Layer A.5), so the concern is resource-leak, not lifecycle-propagation. **Carry-forward to B1 or a follow-up cleanup packet.**

b. **Unused optional fields** in `attachTmuxSession` TS type (`seatId`, `sessionId`): reserved for future seat attribution. Current call-site omits them. No-op at runtime. Not a bug.

c. **Lockfile hygiene**: `pnpm install --frozen-lockfile` exit 0 at `ab672e5`. No drift. Yellow-flag never raised.

## 5. Downstream unlocks

SG-A chapter is now fully closed. The following now proceed:

1. **B1 "GO" dispatch to Nimbus**: B1 packet at `docs/coordination/tasks/nimbus/NIMBUS-2026-05-09-b1-pty-write-send-keys-v1.md` flips `status: draft` → `status: dispatched`. Nimbus may now begin implementation of the tmux `send-keys` / `paste-buffer` bidirectional write path.
2. **Joint supplement review unlocked** per Aegis 2026-05-09 ruling: Lyra product supplement (`3ccbd6d`) × Nimbus arch supplement (`dd8758c`). Parallel track starts now alongside B1.
3. **Nimbus-flagged B1 ↔ joint-review coupling**: Nimbus (in his reply earlier this evening) flagged a potential reshape risk — if the joint review elevates multi-project schema (006) scoping ahead of B1 landing, `PtySession` construction may need `project_id` threading and the buffer-path decision may need revisiting. I'm acknowledging this here; we'll watch for it during joint review and issue a B1-v2 if needed.

## 6. Acknowledgements

- **Mira**: A4-β is the cleanest UI delivery of this sprint. Three files, zero drift, `serde` standalone verification done before writing TS — that last one is the professional mark of this packet.
- **Flux**: Commit-pinned verify turned around in hours. §5 non-blocker pre-marking picked up and preserved in delivery. Three-layer gate protocol held.

---

*Accepted by Lyra · 2026-05-09 late-evening · SG-A closure unconditional · SG-B unlocked · B1 dispatched*
