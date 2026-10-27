# Flux verdict — the terminal-link fix (`cmd_open_external_url`)

| Field | Value |
|---|---|
| template | T3 |
| subtype | verification |
| id | FLUX-2026-10-27-verify-terminal-links-v1 |
| status | delivered |
| author | flux |
| date | 2026-10-27 |
| version | v1 |
| depends_on | AEGIS-2026-10-27-open-missing-acceptance-cards-v1 |
| tags | reverification, mutation-testing, tauri, links, seat-flux |

- **Verifying dispatch**: `disp-CE-64L-sSk44LGwSd8L6y` (Aegis → Flux, to open the missing acceptance card)
- **Subject dispatch**: `disp-H66BOKrYda4xltEgIwvuG` (Mira)
- **Subject commit**: `211b179` on `packet/mira/knowledge-panel` — **not merged to `track/infra-foundation`**
- **Verdict event**: **`ev-3qVHyZW597RSpMiK2zxby`** — the pre-declared id
- **Verdict**: **PASS**

## Verdict: PASS

**The bug is fixed on both surfaces, and I verified it by reading the handler off the real addon
instance rather than by trusting the note.** All ten of her mutations reproduce at exactly the
kill counts she claimed, and every gate figure she reported is accurate.

**Two findings she did not state:**

1. **`an_http_url_is_allowed_past_the_gate` is a tautology.** It asserts that a string literal
   starts with `https://`. I mutated the product so `http://` is **no longer accepted** and all
   **9 Rust tests stayed green** — so the `http` half of the scheme gate is completely untested,
   and the one test named after it proves nothing. See §4.
2. **Her "the bundle has no frontend assets" finding is correct and more consequential than she
   framed it.** It is not merely that a future build would be blank: **the installed app is a dev
   build**, and it only shows anything at all because it proxies `devUrl`. See §5.

---

## 1. The root cause, confirmed at the source

I did not take the `window.open`-is-blocked claim on faith. Her fourth test drives the **real**
`WebLinksAddon` constructed with no handler and spies on `window.open`, and it passes — so the
default really does reach `window.open`, and the default's return value is falsy. Her correction
to the dispatch's wording is right and worth recording: the default handler has **no return
statement at all**, so it returns `undefined`, not `null`. Same consequence (falsy, so the addon
also runs its own path), but the assertion now states what the code does.

Her `StatusBar` finding is also better than the dispatch's framing, and she is right to have
corrected it: the GitHub link had **two** independent defects, not one. `window.open` was blocked
*and* there was no `onClick` at all. "The same disease" understated it — the mouse path was
missing, not broken.

## 2. The fix, verified through the real component

**Terminal links.** Her test reads the handler off the **real addon instance** the component
loaded (spying `loadAddon` and picking the `WebLinksAddon` argument), then calls it. That is the
right shape — it tests the wiring, not a restatement of it. I verified:

| Check | Result |
|---|---|
| a handler is passed at all | yes — the regression this pins |
| the handler routes to `api.openExternalUrl` | yes, with the exact URL |
| the handler returns `true` | yes — so the addon does not also run its own default |
| a rejection is logged with the URL, not swallowed | yes |

**StatusBar.** Five tests, all driving the real component: mouse click opens, Enter opens, Space
opens, a rejection is logged with the URL, and `title` exposes the real URL. Her point about the
`title` is the subtle one and it is correct: the visible text is a local path
(`~/Documents/GitHub/seatloom`) and the click goes to GitHub, so without the title that mismatch
is invisible to the operator.

## 3. Mutations — all ten reproduce at her exact kill counts

| # | Mutation | Her claim | My measurement |
|---|---|---|---|
| M1 | `new WebLinksAddon()` with no handler | RED 2 | **RED 2** |
| M2 | handler returns `false` | RED 1 | **RED 1** |
| M3 | handler swallows the error | RED 1 | **RED 1** |
| M4 | StatusBar loses `onClick` | RED 2 | **RED 2** |
| M5 | StatusBar keeps only Enter | RED 1 | **RED 1** |
| M6 | StatusBar loses `title` | RED 1 | **RED 1** |
| M7 | StatusBar swallows the error | RED 1 | **RED 1** |
| RM1 | the scheme gate removed | RED 3 | **RED 3** |
| RM2 | the host gate removed | RED 2 | **RED 2** |
| RM3 | userinfo read at the first `@` | RED 1 | **RED 1** |

**Every count matches.** Two details worth noting because they show the tests are doing real work:

- Under M1, the test "the addon is loaded WITH a handler" **stays green** — because the xterm
  default is also a function. That is correct behaviour, and it means the discriminating
  assertions are the routing and the return value, not the existence check.
- Under RM3 the only test that dies is the userinfo one, which is exactly the input designed to
  separate the two readings (`https://@github.com/...` — first-`@` gives an empty host and a
  spurious "no host" refusal). That is a well-aimed test.

## 4. Finding: one Rust test is a tautology, and the `http` allowance is untested

`an_http_url_is_allowed_past_the_gate` reads:

```rust
let url = "https://github.com/seatloom/pull/1".to_string();
assert!(url.starts_with("http://") || url.starts_with("https://"));
```

It asserts a property of a **string literal**, using the **same predicate** the product uses. It
cannot fail, and it says nothing about the command. I proved that by mutating the product so
`http://` is no longer accepted:

```
gate accepts https:// only   ->   9 passed, 0 failed
```

**So the `http` half of the scheme gate has no coverage at all**, and the test named after it is
the one that looks like it covers it. This is the same failure shape she herself caught twice and
wrote up ("a test that re-implements the logic under test proves nothing") — it is just that this
one survived her own review.

**Why it is not simply a missing test:** the allowed path spawns a real browser
(`std::process::Command::new("open").arg(url).spawn()`), and there is no injection point — no
trait, no env override, nothing to stub. So a test that calls the command with an `http://` URL
and asserts it is *accepted* would open a browser tab on the developer's machine. Her instinct to
skip that is right; the fix is not to spawn, it is to **extract the gate decision into a pure
function** and test that:

```rust
fn url_is_openable(url: &str) -> Result<(), String> { /* the two prefix tests + host */ }

#[test] fn an_http_url_is_openable() {
    assert!(url_is_openable("http://github.com/seatloom").is_ok());  // real assertion
}
#[test] fn only_http_and_https_are_openable() {
    assert!(url_is_openable("file:///etc/passwd").is_err());
    assert!(url_is_openable("mailto:a@b.c").is_err());
}
```

That would make the `http` allowance load-bearing and would let the tautological test be deleted.
**Suggested, not done** — it is her file, and the dispatch said to report rather than fix.

Note that this does **not** change the verdict: the defect being fixed was "no handler, nothing
opens", and that is genuinely fixed and genuinely pinned. The gap is in the *permission* half of
the new command, not in the fix for the reported bug.

## 5. Her build finding — confirmed, and it explains the recurring "the running binary has no fix"

I verified her claim independently rather than accepting it:

```
/Applications/SeatLoom.app/Contents/Resources/   ->  icon.icns only. No assets/, no index.html.
strings <binary> | grep cmd_open_external_url    ->  0 occurrences
tauri.conf.json build.frontendDist              ->  "../ui/dist"   (correctly set)
ui/dist/index.html                              ->  exists
```

And the part that makes it click:

```
strings <binary> | grep devUrl markers          ->  http://localhost:1420, "index.html", tauri://localhost
```

**The installed app is a dev build.** It renders only because it proxies `devUrl` at
`localhost:1420`. So:

- her finding is **correct** — `cargo tauri build` produces a `.app` whose `Resources` holds only
  the icon, so `frontendDist` is not being copied;
- and it is **the likely root cause of the thing I have now reported five times** ("the running
  app does not contain the fix"). That was not (only) a stale binary: a release bundle built this
  way would have **no frontend at all**, so no frontend fix could ever reach the operator through
  it. She is right that this is more urgent than the link bug.

I did **not** reproduce the release build myself (it is minutes-long and her evidence plus the
installed artefact is conclusive). **Not fixed** — changing `frontendDist` or the build pipeline
affects every seat's artefact and is the build owner's call, exactly as she said.

## 6. The plugin-shell decision — her reasoning checked

She chose `std::process::Command` + `open`/`xdg-open` over `@tauri-apps/plugin-shell`'s JS
`open()`. I verified each premise:

- `@tauri-apps/plugin-shell` **is** declared in `ui/package.json` (line 22) — and there is **no**
  `.plugin(...)` registration anywhere in `src-tauri/src/`. Her claim is correct.
- `cmd_open_local_path` really does use `std::process::Command` + `open`, so this is one
  mechanism, already compiled in. Confirmed at `history_cmds.rs:238-245`.
- No `tauri.conf.json` or `capabilities/default.json` change is needed, because this path does not
  go through the shell plugin's permission model. Confirmed: the commit touches neither file.

**Her decision is right and her reasoning is complete.** Adding a Cargo dependency, a capability
permission, and a second way to open things — to fix a bug in a link — would have been the worse
trade, and she checked rather than assumed.

**Her scheme decision is also right.** Only `http`/`https`; `file://`, `mailto:`, `javascript:`
refused with the scheme named in the error. The threat model is the correct one: the URL comes out
of a wrapped CLI's output, so it is chosen by whoever could write to the operator's terminal. Her
refusal to add a domain allowlist is right too — it would turn "click an unregistered link" into a
silent failure, and scheme is all this layer can responsibly judge.

## 7. Gates — every figure she reported is accurate

- Her tests: `SessionTerminal.links.test.tsx` **4 passed**, `StatusBar.test.tsx` **5 passed**.
- Full UI suite: **330 passed / 28 files / 0 errors** (baseline 314 + her 9 + 7 from the rotation
  dispatch — the arithmetic matches).
- `tsc --noEmit` clean.
- **eslint: 143 problems before AND after**, diffed against `211b179~1` on the whole `src` tree.
  Her "identical to baseline, 0 on the files I touched" claim holds.
- **prettier**: her two new files are clean; `StatusBar.tsx` and `api.ts` are listed as different
  — and were **already** different before the commit, which I confirmed by checking out the parent.
  So she added no new formatting debt, exactly as she said.
- Rust: **21 tests pass** (9 in `history_cmds`), `cargo clippy --all-targets` clean,
  `cargo fmt --check` clean, `seatloom-core` **168 pass** (unchanged).
- `vite build` succeeds and the built `SupervisorPanel` chunk contains `cmd_open_external_url`.

## 8. What I did not do

- Did **not** extract the gate into a pure function or delete the tautological test. Suggested in
  §4; it is her file and the dispatch said report rather than fix.
- Did **not** touch `tauri.conf.json`, the build pipeline, or the installed app. Reported.
- Did **not** merge. `211b179` is not on `track`; the running binary is 14:03 and contains zero
  occurrences of `cmd_open_external_url`.
- Did **not** touch Mira's worktree beyond a detached worktree at `211b179`, removed afterwards;
  her `git status` is empty.
- Did **not** click anything in a GUI. Criterion 5 (real-machine click) remains open and needs a
  human — she said so plainly rather than claiming it.

## Reproduction

```bash
git -C <mira> worktree add --detach /private/tmp/links 211b179
ln -s <mira>/ui/node_modules /private/tmp/links/ui/node_modules
cd /private/tmp/links/ui

./node_modules/.bin/vitest run src/app-v2/panel/SessionTerminal.links.test.tsx   # 4 passed
./node_modules/.bin/vitest run src/layouts/StatusBar.test.tsx                    # 5 passed
./node_modules/.bin/vitest run                                                   # 330 / 28 files
./node_modules/.bin/tsc --noEmit                                                 # clean
./node_modules/.bin/eslint src                                                   # 143 (same as 211b179~1)
./node_modules/.bin/prettier --check src/app-v2/panel/SessionTerminal.links.test.tsx \
    src/layouts/StatusBar.test.tsx                                               # clean
./node_modules/.bin/vite build                                                   # ok
cd .. && cargo test -p seatloom-tauri                                            # 21 passed
        cargo clippy -p seatloom-tauri --all-targets && cargo fmt -- --check     # clean

# THE DEFAULT HANDLER, MEASURED (not described):
#   new WebLinksAddon() with no handler -> _handler is a function, it reaches
#   window.open, and it returns FALSY (undefined, not null — no return statement).

# MUTATIONS (each reverted, tree verified identical afterwards) — all match her claims:
#   no handler                    -> RED 2
#   handler returns false         -> RED 1
#   handler swallows the error    -> RED 1
#   StatusBar loses onClick       -> RED 2
#   StatusBar keeps only Enter    -> RED 1
#   StatusBar loses title         -> RED 1
#   StatusBar swallows the error  -> RED 1
#   scheme gate removed           -> RED 3
#   host gate removed             -> RED 2
#   userinfo at the first '@'     -> RED 1

# THE GAP (my finding, §4):
#   gate accepts https:// ONLY    -> 9 passed, 0 failed   <- http is untested
#   because an_http_url_is_allowed_past_the_gate asserts a STRING LITERAL with the
#   product's own predicate. Fix: extract the gate decision into a pure fn and test it.

# HER BUILD FINDING, INDEPENDENTLY CONFIRMED:
#   ls /Applications/SeatLoom.app/Contents/Resources/   -> icon.icns only
#   strings <binary> | grep -c cmd_open_external_url     -> 0
#   strings <binary> | grep localhost:1420               -> present: it is a DEV build
#   tauri.conf.json frontendDist = "../ui/dist"          -> correctly set, still not copied
```

## Verdict event

`ev-3qVHyZW597RSpMiK2zxby`, `ReviewVerdictIssued`, `verdict: pass`,
`dispatch_id: disp-H66BOKrYda4xltEgIwvuG` in the payload.

## Files I did not change

No source file. Ten mutations were applied and each reverted, with the tree confirmed
byte-identical to `211b179` after each. One detached worktree was created at Mira's packet branch
and removed afterwards. My probe files were deleted.
