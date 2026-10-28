# Delivery: a permanent in-window identifier for sandbox builds

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | MIRA-2026-10-21-sandbox-build-banner-v1 |
| status | delivered |
| author | mira |
| date | 2026-10-21 |
| version | v1 |
| dispatch | disp-ZqvAuQ1GANGLx8Y8FLz2T |
| depends_on | AEGIS-2026-10-21-sandbox-build-banner-v1 |
| tags | sandbox, runtime-identity, banner, delivery, mira |

---

## 0. One paragraph

The icon is the first layer of "which build am I looking at" and it only works in the Dock
and Cmd-Tab. With a window open, the sandbox and the main build look identical, so this
adds the second layer: a permanent strip at the top of the window that names the branch,
the commit and the build time. The sandbox renders it; the main build renders nothing at
all. Everything it shows comes from a new backend command that reads four environment
variables Aegis's installer already injects, so it does not depend on localStorage's
three un-namespaced keys, which the two builds fight over when both are open.

## 1. What I measured before writing anything

The packet's mechanism was "read a process environment variable", which depends on the
installer's launch actually reaching the app. I did not take that on trust, because if it
were wrong the banner would never light and the whole dispatch would be a green build
with no visible result.

I built a throwaway `.app` bundle that printed its own environment, launched it the way
`prepare-sandbox-build.sh` does, and read what it saw:

| launch method | did the app see the variable |
|---|---|
| `SEATLOOM_SANDBOX=1 ... open -a "$APP"` (what the script does) | **yes** |
| `launchctl setenv ... open -a "$APP"` | no |

So the installer's existing mechanism works and nothing there needed changing — the
frontend could simply read the env vars. (Had it been the `launchctl` case, the fix would
have had to reach Aegis's script.) The measurements and their conclusion are recorded in
both new files, so the next person does not have to re-run the probe.

## 2. The backend command

`src-tauri/src/commands/runtime_cmds.rs` → `cmd_get_runtime_identity`. It reads four
environment variables and nothing else: no database, no process spawn, no new
permission, no new capability. That is the whole point — the frontend must not be given a
new way to do things.

```rust
pub struct RuntimeIdentityDto {
    pub is_sandbox: bool,      // SEATLOOM_SANDBOX == "1"
    pub branch: Option<String>,// SEATLOOM_SANDBOX_BRANCH
    pub commit: Option<String>,// SEATLOOM_SANDBOX_COMMIT
    pub built_at: Option<String>, // SEATLOOM_SANDBOX_BUILT
}
```

Two design decisions worth stating:

- **`is_sandbox` is exactly `"1"`.** Not present-and-truthy: an empty string, `"0"`,
  `"true"`, `"01"`, or a value with stray whitespace all read as *not* the sandbox. The
  failure direction is deliberate — if the installer ever writes something other than
  `"1"`, the operator sees "not recorded" and notices, rather than trusting a strip that
  claims something it cannot prove.
- **Absent means `None`, all the way to the screen.** An exported-but-empty variable is
  not information; `env_value` drops it. That is what lets the UI print "not recorded" for one
  field while still naming the others.

The command cannot fail — there is no I/O it performs. That is intentional: a banner that
errored out would be indistinguishable from a missing one, and the caller could not tell
"the command is broken" from "this is the main build".

Five Rust tests, run twice to prove they are order-independent: exactly-`"1"` lights it,
the main build reports no sandbox and no details, eight near-misses do not light it, empty
and whitespace values arrive absent, and a sandbox whose installer recorded nothing still
states that it is a sandbox. The tests needed a real snapshot/restore guard around the
process-global env state — and the first version of that guard was written as a comment
rather than as code, which is the sort of thing that fails loudly here, so the test suite
caught it.

## 3. The banner

`ui/src/components/SandboxBanner.tsx`, mounted once inside `AppShell`'s children, above
the `ProvenanceFrame`, so it is present on every tab and is not re-created by navigation.

```
SANDBOX · branch packet/mira/knowledge-panel · commit abc1234 · built 10-21 13:20
```

Missing values print as "not recorded". No dismiss button, no way to close it — it is an
identity, not a notice, and a banner that can be turned off stops being one.

**The main build renders nothing.** The packet rules out a symmetric "I am the main
build" strip, and the CSS has no rule for one.

**Failure is said out loud.** If the command throws, the strip turns red and states that
this build's identity cannot be confirmed — it does *not* fall back to looking like a
clean main build. A banner missing because the backend failed is indistinguishable from
a main build that correctly shows nothing, which is exactly the confusion this exists to
remove. The same applies to a payload the code does not recognise (an older or renamed
struct): that is reported as an identity failure, not silently read as "main build".

The API call is one seam, not a second implementation: `lib/api.ts` is Nimbus's fenced
file and does not yet carry `getRuntimeIdentity`, so the component calls the same Tauri
command directly and names itself as a seam in the comment, exactly like
`KnowledgeSection.tsx` does for the knowledge tree. When the additive edit lands, replace
the block with `api.getRuntimeIdentity()` — one call site.

Styling lives in `styles/globals.css` next to the other `.sl-*` component rules, in the
orange family to match the icon, so both layers read as the same fact.

## 4. Tests

`ui/src/components/SandboxBanner.test.tsx`, 9 assertions, both directions:

- sandbox: the strip appears and names the branch, the commit and the build time, and it
  names the command it read so a wrong build can be traced
- main build: `container.innerHTML === ''` — literally nothing, and specifically no
  "I am the main build" strip
- missing values: "not recorded" counted per field, with no `null`, no `undefined`, no invented
  word such as `unknown` or `main`
- command failure: the error strip appears and says sandbox-or-main cannot be
  determined; the sandbox strip does not appear
- unrecognised payload: treated as a failure, not as the main build
- no dismiss control exists at all

## 5. Gates

| Gate | Result |
|---|---|
| `cargo test` | 168 passed (12 new, in `seatloom-tauri`) |
| `cargo clippy --all-targets` | no new warnings |
| `cargo fmt --all -- --check` | clean |
| `tsc --noEmit` | clean |
| `vitest run` | **227/227**, 16 files (9 new) |
| `vite build` | green (the >500 kB chunk advisory is pre-existing) |
| lint | `SandboxBanner.tsx` 0, `SandboxBanner.test.tsx` 0, `globals.css` 0; `App.tsx` 15 → 15 against a `track` worktree (all 15 pre-existing) |
| `verify-docs.sh` | this note clean |

## 6. Not verified, and boundaries

No app was launched, so the strip has not been seen on screen: no real sandbox build was
installed from this seat (the scripts and `/Applications` are the operator's). The
end-to-end wiring is verified by unit tests on both sides of the IPC boundary plus the
measured `open -a` propagation probe; what has not been observed is a real window with a
real strip in it.

No database write, no acceptance card, no merge, no mount of `AppV2`, and
`tauri.conf.json`'s CSP untouched. `lib/api.ts` and `lib/types-dto.ts` were not edited —
the new command is reached through one named seam in the component, as the fence requires
for a missing wrapper.

## 7. Artifact index

- Branch: `packet/mira/knowledge-panel`.
- Ledger: see `emit_event(ArtifactCreated)`.
- This delivery note.
