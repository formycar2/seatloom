# Heartbeat — 2026-10-28 (fourth pass): you accepted three fixes you cannot see, and the reason they are invisible is now gone

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-28-heartbeat-pass4-v1 |
| status | issued |
| author | lyra |
| date | 2026-10-28 |
| version | v1 |
| tags | heartbeat, landing, running-app, accepted-not-visible, prepared |

> Fourth heartbeat pass. No dispatch waiting (`get_next_dispatch` → nothing ready), queue empty.
> facts / findings / decisions separated. Corrects the precision of pass 2 and pass 3 without
> retracting either: pass 2's instrument was wrong (retracted in place), pass 3's per-card
> conclusion was right but was about a *different* card.

## 1. Facts — measured this turn

| # | What | Measurement |
|---|---|---|
| F1 | The operator cleared 30 cards at 18:19 | 30 `AcceptanceGranted` between 10:19:08 and 10:19:26, one merge `8e56e44` (18:18:59) onto track. `accepted` 26 → 56, `pending` 31 → 1. |
| F2 | Queue and ledger | 316 done / 0 ready / 0 in_flight; ledger last event **11 minutes** before the query. |
| F3 | **Three user-visible fixes are on track, accepted, and absent from the app he runs** | measured by decompressing every brotli blob embedded in `/Applications/SeatLoom.app` (15 blobs, 1,036,421 bytes) and content-matching against a fresh `ui/dist`: |
| F4 | | The two mirror strings in `ui/src/components/MirrorDetached*` (quoted in §3 below) — **0** in app, **1** in fresh dist. Landed `9a67ea5` (10-27 20:26), card `acc-disp-jZg89gwy8YzVOOjdAsxTL` **accepted 18:18:57**. |
| F5 | | The session-work section header in `ui/src/app-v2/panel/*SessionDetail*` (quoted in §3 below) — **0** in app, **1** in fresh dist. Landed `252c5aa` (10-28 13:18). |
| F6 | What IS in the app | The two supervisor-sync strings in `ui/src/components/SupervisorSyncLost.tsx` are **present in both** — they landed 10-27 05:10/07:06, before the 14:03 build. So the app is not uniformly stale; it is a clean snapshot of 10-27 14:03. |
| F7 | Why the gap exists, exactly | app binary mtime **2026-10-27 14:03:22**; `ui/dist` mtime **10-27 21:48**; track tip **10-28 18:18**. Three different times, none equal. |
| F8 | **The blocker on landing them is gone** | `scripts/land-the-mirror-fix.sh` dry-run now passes step 1 — *"yes - 9a67ea5 is an ancestor of track/infra-foundation"* — because it was written to refuse until the card was accepted, and the card was accepted at 18:18:57. Earlier today it exited 1 with *"the fix is not on track yet … Nothing to install."* |
| F9 | A landing is cheap and proven | `vite build` from the existing `node_modules`, **offline**, **2.0 seconds**, and the output contains all three missing strings. `cargo check -p seatloom-core --offline` also passes. |
| F10 | What `--apply` would do | back up the running binary as `.bak-<timestamp>`, install the new one, then print the three steps for the operator — it deliberately does **not** restart, because that closes his windows. |
| F11 | What I did not do | run `--apply`. Installing over `/Applications/SeatLoom.app` replaces the app the operator is using right now. That is his call, not mine, and `LYRA-HEARTBEAT.md` says not to make his decisions for him. |
| F12 | What I cannot measure | whether his open window currently renders — no assistive access (`System Events` → `-25211`), no screenshot. Unmeasured, not guessed. |

## 2. Finding

**The accepted work is real and reachable; the only thing between it and him is one command plus one restart.** F3–F6 mean this is not "the app is a mockup" and not "nothing is usable" — it is a precise, enumerable three-item list, each with a commit, a card, and a proof that a fresh build contains it. F8 means the guard that was stopping the landing has been satisfied by his own 18:19 clicks.

That is worth stating because the previous framing ("40 commits stale") invited the conclusion that the fix was a big rebuild. It is not. It is: `pnpm build`, `pnpm tauri build`, `verify-tauri-bundle.sh`, install, restart. Aegis already wrote all of it and proved the verification logic.

## 3. Decision — what I am handing the operator, in his three-part shape

**What changed:** you accepted 30 cards at 18:19, including the mirror pipe-guard fix, the
turn-aware dual render and the session-work bridge view. All three are on track.

**What you will see, once the app is rebuilt:** a seat whose pipe gets stolen by another process stops
appearing and offers a re-attach button (instead of silently showing stale output); the live-seats pane
gets the turn-aware smooth window; and the session detail gains a section listing what that session
distilled, with the three states kept visually distinct. **You will not see any of the three right now** —
measured: 0 occurrences of their strings in the app you have open.

**Reproducing that measurement** (the claim is only as good as its instrument — see §5):

```bash
# 1. the three strings, as they appear on track
for s in 'mirror-lost' 'sessionWork'; do
  echo "$s: track=$(git grep -l "$s" track/infra-foundation -- ui/src | wc -l)"
done
# 2. decompress the app's embedded blobs and content-match (NOT strings, NOT ls Resources)
python3 - <<'PY'
import glob,os,subprocess
exe=open('/Applications/SeatLoom.app/Contents/MacOS/seatloom-tauri','rb').read()
out=b''
for p in glob.glob('target/**/*',recursive=True):
    if not os.path.isfile(p) or os.path.getsize(p)<50: continue
    if open(p,'rb').read() in exe:
        try: out+=subprocess.run(['brotli','-d','-c',p],capture_output=True,timeout=20).stdout
        except Exception: pass
for s in [b'mirror-lost', b'sessionWork']:
    print(s.decode(), out.count(s))
PY
# 3. and a fresh frontend build, for the contrast
cd ui && ./node_modules/.bin/vite build --outDir /tmp/dist-check
```

The instrument matters: `strings` on the app finds **none** of these, because Tauri v2 embeds the
frontend brotli-compressed. A `strings` check therefore reports a healthy app as broken and a broken
one as healthy depending on which way you squint — which is the mistake I made in pass 2 and retracted.

## 5. The instrument, stated so the next person does not repeat my error

**What you need to do:** decide whether to land it now. If yes, one command in the main checkout:

```
bash scripts/land-the-mirror-fix.sh --apply
```

It backs up the current binary, installs the new one, proves the bundle really contains the frontend by
decompressing and content-matching (never by listing `Resources`, never by grepping the executable),
and then tells you the three steps to restart — it will not restart for you, because that closes your
windows. I have not run it: it replaces the app you are using.

## 4. Escalations

- **To Aegis:** the three-step landing chain is yours to own, and its first guard has now flipped to
  green. If you would rather it be a standing heartbeat check — "is the installed app's embedded
  content behind track, and by which strings?" — I have the instrument working and can hand it over.
- **To whoever owns the sandbox:** `/Applications/SeatLoom-Sandbox.app` is from Oct 21, 282 commits and
  36 cards behind. If a side-by-side is wanted for future cards, `prepare-sandbox-build.sh` dry-runs clean.
- **Nothing dispatched.** The queue is empty and there is no seat work here: the remaining step is a
  decision only the operator can make. Inventing a dispatch would burn a seat's attention for nothing.
