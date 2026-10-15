---
template: T5
subtype: ops_plan
doc_id: LYRA-2026-10-15-model-proxy-domain-migration-v1
title: Migration plan — model-proxy.stepfun-inc.com → mp.z01.normandyfunc.com (stepgo / stepcode toolchain)
status: active
author: lyra
doc_date: 2026-10-15
version: v1
depends_on: scripts/host-move-model-proxy-domain.sh
tags: ops, stepgo, stepcode, domain-migration, model-proxy, host-move
---

# Migration plan — model proxy domain rename

**Scope:** the stepgo toolchain on this machine (`~/.stepgo`, the live install; `~/.stepcode`
is a dead legacy tree). **Driver:** the operator's request to replace
`models-proxy.stepfun-inc.com` with `mp.z01.normandyfunc.com`, mainly for the claude code
configs that stepgo launches.

**Script:** `scripts/host-move-model-proxy-domain.sh` — idempotent, `--dry-run`, and the
bundle patch is **gated behind an explicit `--include-bundle`** so it cannot happen by accident:

```
./scripts/host-move-model-proxy-domain.sh                    # 只改活动配置(默认)
./scripts/host-move-model-proxy-domain.sh --include-bundle   # 同时改装机程序的白名单/默认值
./scripts/host-move-model-proxy-domain.sh --dry-run          # 只报告,什么都不动
./scripts/host-move-model-proxy-domain.sh --rollback         # 回滚
```

The gate exists because a first version of the script patched the bundle by default, and a
test run executed it before the decision in §8.1 had been made. It was reverted immediately
(§9). A tool that can modify installed third-party code must make that an explicit act.

---

## 1. Decision

The rename is **done for every live config surface and verified at the program level**
(`stepgo config get baseUrl` returns the new host). Three things remain open, in descending
order of danger:

1. **A silent bypass exists** (§3.2): activating either of the two configured profiles makes
   stepgo *strip the root `baseUrl`* and route to a **different service**
   (`api.stepfun.com/step_plan`, different keys). This must be settled before anything else,
   because it can undo the migration without any error.
2. **Two host allow-lists inside the installed program** (§3.1) still name the old host. I
   measured the consequence: not fatal, but claude code's model picker grows from 3 entries
   to 384.
3. **A bundle patch does not survive a manual upgrade** (§5). Auto-update is off, so the
   patch persists until someone updates by hand — at which point it must be replayed.

## 2. What was changed, and what was deliberately not

### 2.1 Changed — 12 files, each backed up as `.bak-hostmove-20261015-193530`

| File | Occurrences | Field |
|---|---|---|
| `~/.stepgo/config/config.json` | 1 | top-level `baseUrl` |
| `~/.stepgo/config/agent/managed/config.json` | 3 | two `baseUrl` (with `/v1`) + one bare entry in an array |
| `~/.stepgo/agents/cursor/config.json` | 1 | `target_api` |
| `~/.stepgo/agents/claude/launch/*/settings.json` | 7 files | **`ANTHROPIC_BASE_URL`** — the one claude code actually reads |
| `~/.pi/agent/models.json` | 2 | pi's own provider `baseUrl` |

`/v1` suffixes were preserved (only the host substring was replaced). All JSON/JSONC files
re-validated after the edit.

### 2.2 Deliberately not changed

| Target | Why |
|---|---|
| `~/.stepgo/logs/**`, `~/.stepgo/sessions/**`, `~/.stepcode/shadow-eval/**` | Logs and session transcripts are evidence. Rewriting them destroys the record for zero benefit. |
| All `*.bak-*`, `*.backup-*`, `*.legacy-disabled-*` | Historical snapshots kept for rollback. |
| `~/.stepgo/cache/models-list-*.json` | Derived cache, keyed by a hash that changes with the host. Editing it is pointless; it orphans itself. |
| `~/.stepcode/**` | Verified dead: `PATH`'s `stepcode` resolves to `~/.stepgo/bin/stepcode`, `STEPCODE_HOME` is unset, `~/.stepcode/current` still points at `versions/1.2.79` while the live runtime is `~/.stepgo/runtime/host`, no process references it, and its claude launch directory is empty. |
| `~/.stepgo/runtime/host/dist/*.js` | **Not yet** — see §3.1. This is the one place a patch is technically possible but should be a conscious decision. |

## 3. The three open items

### 3.1 The two host allow-lists in the installed program

`~/.stepgo/runtime/host/dist/` contains four occurrences of the old host:

| File:line | What it is | Consequence of leaving it |
|---|---|---|
| `VJN2XLCC.js:34` | `FULL_CHANNEL_PURPOSE_HOSTS` — gates whether `/v1/models` gets `?purpose=code` | **Measured:** the catalog goes from 5 curated code models to all 384. `claude-opus-4-8` is present in both, and the model is pinned in `settings.json`, so nothing breaks — but claude code's picker lists 384 instead of 3. |
| `SUL6KV6A.js:16` | `MODEL_PLATFORM_HOSTS` — gates whether the host is added to `NO_PROXY` | **Latent only:** no proxy env var is set today, so nothing happens. If a proxy is ever configured, the new host would not bypass it. |
| `QZGQPGY3.js:15` | `DEFAULT_BASE_URL_FALLBACK` | Only reached when config has no `baseUrl`. Ours has one. |
| `QZGQPGY3.js:60` | Build-time-inlined default in `getDefaultBaseUrl()` | Same — only reached when config has no `baseUrl`. |

**Measured evidence that the two hosts are the same backend:** `/v1/models` returns identical
model lists on both (384 without `purpose`, 5 with `purpose=code`), and `/v1/messages`
returns a byte-identical billing error on both — including `Used 2528.70 of 2500.00`. Same
quota accounting means same backend, so this is a DNS rename, not a service swap.

**Recommendation: patch all four.** Rationale: a domain migration that leaves the vendor
default pointing at the old name is exactly the half-migration that bites six months later,
and the picker regression is user-visible. The cost is that a manual upgrade wipes it (§5).

### 3.2 The silent bypass — settle this first

`applySelectedProfileConfig()` calls `stripRootRuntimeFieldsForProfile()`, which destructures
**`baseUrl` out of the root config** whenever a profile is active. So:

- **Now (no `activeProfile` in the config, no env override):** the root `baseUrl` wins → the
  new host. Verified.
- **If `code-plan` or `stepplan-claude` is ever activated:** the root `baseUrl` is stripped
  and the profile's own `baseUrl` takes over — which is
  `https://api.stepfun.com/step_plan`, **a different service with two different API keys**
  (`2E7vWs…oAZO`, `1B1X0S…eWIn` — not the model-proxy key `ak-d…er4f`).

**Do not "fix" this by rewriting the profiles' `baseUrl` to the new host.** They point at a
different service; that would silently repoint the step_plan integration at the model proxy.

**What is actually needed is a decision from the operator:** are those two profiles still in
use? If yes, the migration must state explicitly that profile-mode traffic does **not** go
through the model proxy, so nobody later concludes the rename failed. If no, deactivate them
and say so. Either way, the script's post-check already warns when the effective `baseUrl`
is not the new host, which is what would catch this.

### 3.3 Other hosts in the same toolchain (out of scope, recorded so they are not surprises)

- `api.stepfun.com/step_plan` — the two profiles' endpoint (§3.2).
- `sado-platform.basemind.com` — hardcoded fallback for the eval client (`VU4KBRZN.js`).
- `stepcode.basemind.com` — sits alongside the old host in both allow-lists.

None of these is the domain being renamed.

## 4. Verification

| Level | How | Status |
|---|---|---|
| File | `grep` the live config for the old host | ✅ zero remaining |
| Syntax | JSON/JSONC re-parse every changed file | ✅ all valid, `/v1` preserved |
| **Program** | `stepgo config get baseUrl` | ✅ returns the new host |
| **Program** | `stepgo config show` (merged config from all sources) | ✅ `baseUrl` = new host; profiles carry no model-proxy URL |
| Backend identity | identical `/v1/models` lists and identical `/v1/messages` quota error on both hosts | ✅ same backend |
| End-to-end | launch a real session and confirm `ANTHROPIC_BASE_URL` + that traffic reaches the new host | ⚠️ **cannot fully pass — the API key's quota is exhausted** (`2528.70 / 2500.00`), so no model call can succeed on either host. Routing can be verified; content cannot. |

The script runs the file, program and backend-identity checks on every invocation, so a
re-run after any upgrade is a full re-verification, not just a re-patch.

## 5. Durability across upgrades

- `updateCheckEnabled = false` in the config, and `~/.stepgo/runtime/` holds no
  `.old` / `.rollback` / `.new` staging directories — so nothing updates spontaneously and a
  bundle patch persists until a **manual** update.
- A manual update replaces `~/.stepgo/runtime/host/dist/`, which wipes §3.1's patch. The
  runtime does **not** rewrite `~/.stepgo/config/config.json` on its own (only
  `updateConfig()` does, from commands like `stepgo config model`), so the config half of the
  migration survives.
- **Two side effects of `updateConfig()` worth knowing:** it rewrites the whole file with
  `writeJSON(..., {spaces: 2})`, which **strips the `//` comments** — including the one that
  documents the disabled previous API key — and if a profile is active it writes the change
  *into the profile* instead of the root. Neither loses `baseUrl`, but the comment loss is
  real.
- Therefore: **after any manual stepgo update, re-run the script.** It is idempotent and
  re-verifies.

## 6. Rollback

```
scripts/host-move-model-proxy-domain.sh --rollback
```

restores every `.bak-hostmove-*` to its original name. Note that a running claude code
session holds its environment from startup, so it must be restarted to pick up either
direction of the change.

## 7. Live-session caveat

One claude code session is running right now against
`~/.stepgo/agents/claude/launch/def8a5e5-0678-44ff-a74e-086d4bf39f2d/settings.json` — a file
this migration changed. Its process loaded the old value at startup, but that settings file
registers a `ConfigChange` hook, so claude code watches it and may reload mid-session. Both
hosts are the same backend, so this is functionally harmless, but in-flight requests could
stutter. Restarting the session removes the ambiguity.

## 9. Process note — the bundle was patched once, by accident

While testing the script, its first version patched the bundle by default, so the test run
executed the patch that §3.1 says needs a decision. It was reverted immediately and
precisely (only the three `dist/*.js` files, from their `.bak-hostmove-20261015-194540`
backups); the authorized config change was untouched. Verified after the revert: the bundle
holds its original four old-host occurrences, and `stepgo config get baseUrl` still returns
the new host. The script now requires `--include-bundle` for that path and supports
`--dry-run`, so testing can no longer mutate anything.

The lesson is the one this project keeps paying for: a tool that can modify installed
third-party code should not do it as a side effect of being run.

## 10. Open decisions for the operator

1. **Patch the bundle's four occurrences?** (Recommendation: yes — §3.1.)
2. **Are the `code-plan` / `stepplan-claude` profiles still in use?** They bypass the model
   proxy entirely (§3.2). This one gates how the migration should be documented, and it is
   the only item that can silently undo the rename.
3. **The API key is out of quota** (`2528.70 / 2500.00`). Independent of this migration, no
   model call will succeed until it is raised or replaced.
