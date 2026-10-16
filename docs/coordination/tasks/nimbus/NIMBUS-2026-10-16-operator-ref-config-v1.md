# Delivery: the operator identity can be configured without the environment — restoring a channel that was fixed on 10-12 and silently reverted on 10-13

[Nimbus -> Aegis] Mr Zhang clicked "accept" on 2026-10-16 and hit
`SEATLOOM_OPERATOR_REF is not set`. The refusal was **correct** and I did not
weaken it. What was broken is that it read from the environment only, and an app
launched from the Launchpad has no environment at all — so a correct guard had
become a wall he could never get past. This had been fixed once already and then
lost. That is the part worth your attention.

| Field | Value |
|---|---|
| template | T3 |
| subtype | delivery |
| id | NIMBUS-2026-10-16-operator-ref-config-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-10-16 |
| version | v1 |
| to | aegis |
| dispatch | `disp-SfHOAaw2hl0EfvAZdAxFv` |
| contract | `disp-SfHOAaw2hl0EfvAZdAxFv` payload; Aegis ruling 2026-10-11 (gate vs announcement) for the discipline around gates |
| depends_on | 2339223 (the original fix), 1e7b254 (the integration that reverted it) |
| branch | `packet/nimbus/gate-vs-announcement` |
| tags | acceptance, operator-identity, config, regression, silent-revert, launchpad |

---

## 1. The regression, measured

This is not a new gap. It is a fix that existed and was lost.

| Commit | Date | What happened |
|---|---|---|
| `b6751a1` | 2026-10-10 | `packet/nimbus/gate-vs-announcement` forks from `track/infra-foundation` here |
| `2339223` | 2026-10-12 | **The fix.** Mr Zhang adds a single-line file at `~/.seatloom/operator`, precedence env > file > refuse, plus two tests and a refusal that names both channels |
| `1e7b254` | 2026-10-13 | **The revert.** The intent integration onto track takes `acceptance_cmds.rs` wholesale from my packet branch. My branch forked at `b6751a1`, *before* the fix, so the file came back env-only |
| today | 2026-10-16 | Mr Zhang clicks accept. The wall is back |

Measured, not inferred:

```
$ git merge-base --is-ancestor 2339223 HEAD        # my branch
NO — not an ancestor
$ git merge-base --is-ancestor 2339223 track/infra-foundation
YES on track
$ git show track/infra-foundation:src-tauri/src/commands/acceptance_cmds.rs | grep -n 'fn operator_ref' -A 10
fn operator_ref() -> Result<String, String> {
    std::env::var(OPERATOR_REF_ENV)     # env only. The file is gone.
```

**The lesson worth writing down.** `1e7b254`'s own message says *"Integrated the
complete coherent changeset per crate (not file-picked — that is what produced the
add/add conflicts and the lying panel last time)."* Crate-level integration is what
stopped file-picking, and it is right. But it protects against **intra-branch**
conflicts, not against **track-side fixes made after the packet branch forked**.
The crate arrived coherent and still carried a one-day-old revert. Nothing in the
process caught it, because every gate is green on both sides of the revert — the
tests that pinned the fix lived in the file that was overwritten.

**What would have caught it:** a proof that measures the channel rather than a test
inside the file that can be overwritten with it. That is what this delivery adds
(`proof/operator_ref_config_proof.sh`), and it is the reason I did not just re-apply
the patch.

## 2. What was built

| File | What |
|---|---|
| `crates/seatloom-core/src/config.rs` | new — the one place the rule lives: the two channels, the precedence, the four distinct failures, the refusal message, 11 unit tests |
| `crates/seatloom-core/src/lib.rs` | `pub mod config;` |
| `crates/seatloom-core/examples/operator_ref_probe.rs` | new — drives the real reader from a process with a scrubbed environment, so the Launchpad case is measured rather than asserted |
| `crates/seatloom-mcp-emit/proof/operator_ref_config_proof.sh` | new — eight points against the real file and the real filesystem |
| `src-tauri/src/commands/acceptance_cmds.rs` | `operator_ref()` now calls the core reader and renders its typed refusal; the local `OPERATOR_REF_ENV` constant is gone so there is one home |

**One implementation.** `seatloom_core::config::read_operator_ref()` is the only
reader. The Tauri command holds no second copy — the proof asserts that
`std::env::var(OPERATOR_REF_ENV)` does not reappear in `acceptance_cmds.rs`, so a
second implementation cannot come back unnoticed.

## 3. Path and format, stated

- **Path:** `~/.seatloom/operator` — resolved from `$HOME`, **never** from the
  working directory. This is the whole point: an app opened from the Launchpad has
  CWD `/`, so a repo-relative path would not be found.
- **Format:** one line, the operator ref. `#` comments and blank lines are ignored,
  so the file can say what it is.
- **Value on this machine:** `human:supervisor` — the ref already present in the
  ledger's own spelling (`dispatches.issued_by`, `canonical_events.actor_ref`) and
  the same value the code injects into tool processes in `dispatch_cmds.rs`. It is
  **not** written as a default anywhere; it is what the file already contains.

## 4. Precedence, and the hard line

**File first, environment second** — the file is the persistent documented channel,
the environment is the per-launch override and the fallback when the file cannot be
read.

The hard line is intact and the proof measures it:

- **No default, no guess.** With both channels silent the only possible outcome is
  a refusal. The proof runs that case three times and greps for the word "default".
- **A blank file is not an identity.** Proven.
- **Two stated identities is a refusal, not a pick.** The file holds two non-empty
  lines → the refusal shows both and asks which was meant. Picking one would be the
  exact "never guess" failure the module exists to avoid.
- **There is no third channel.** One function, one constant, one read.

## 5. The four failures are four different messages

The first cut made a blank file indistinguishable from an absent one and fell
through to the generic refusal. They are now separate, because they call for
different actions:

| State | Message |
|---|---|
| Neither channel states one | "SeatLoom is not told who you are yet — there is no identity file and no `SEATLOOM_OPERATOR_REF`." + the absolute path + the exact command + "no restart needed" |
| File present but empty | "Your identity file is there but it is empty." |
| File present, two lines | "your identity file states more than one: […]. Keep exactly one line in it." |
| File present, unreadable | "…exists but cannot be read (permission denied)." |
| Env set but blank | "`SEATLOOM_OPERATOR_REF` is set but blank, which is not an identity" |

**A bug the unit test caught in my own first cut:** the "not configured" message
read *"this app cannot find your home directory"* when `$HOME` was set and the file
was simply absent — because the message re-read the live `$HOME` while the error
carried the path that was actually tried. Both now come from the error, and the
test pins that the message never claims `$HOME` is missing when it is not.

## 6. When a change takes effect

**The next click. No restart.** The file is read inside `cmd_accept_delivery` /
`cmd_reject_delivery`, not cached at startup, so there is nothing stale to reload.
An app that is already running picks the change up on its next decision.

## 7. Gates run

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -p seatloom-core --all-targets -- -D warnings` | clean |
| `cargo clippy -p seatloom-mcp-emit --all-targets -- -D warnings` | clean |
| `cargo test -p seatloom-core` | 172 passed / 0 failed (2 ignored, need Postgres) — 11 of them this slice |
| `cargo test -p seatloom-mcp-emit` | 85 passed / 0 failed (19 ignored) |
| `cargo test -p seatloom-mcp-emit --bin seatloom-waker` | 9 passed / 0 failed |
| `cargo check -p seatloom-tauri` | clean |
| `cargo check --workspace` | clean |
| `proof/operator_ref_config_proof.sh` | ALL POINTS PASS |
| `proof/lineage_ref_proof.sh` | ALL POINTS PASS (unchanged, re-run) |
| `scripts/verify-docs.sh` | this note is clean |

## 8. Acceptance — what I measured, and what is yours to click

**Measured by me, against the real file and the real filesystem:**

| Criterion | Result |
|---|---|
| The real `~/.seatloom/operator` resolves | `RESOLVED value=human:supervisor source=File("…/.seatloom/operator")` — with the environment scrubbed, i.e. the Launchpad case |
| Delete the file, click accept | Refusal naming the absolute path, `echo 'human:supervisor' > <path>`, and "no restart needed" |
| A blank file | Refused — "exists but is empty" |
| Two lines | Refused — both lines shown |
| The env channel still works | `RESOLVED … source=Env` |
| The read writes nothing | File byte size identical before and after |
| No default exists | Refusal three times over, and no occurrence of "default" in the message |

**Yours to click, which I cannot run and am not claiming:**

1. Open SeatLoom from the Launchpad (no script, no terminal), click accept on a
   pending card. **Expected: it merges.** The file is already in place with the
   right value, so no setup step is needed first.
2. Then, to see the refusal: `rm ~/.seatloom/operator`, click accept again.
   **Expected: the message in §5, naming the file and the command.**
3. Then `echo 'human:supervisor' > ~/.seatloom/operator` and click accept again
   **without restarting the app**. **Expected: it works** — that is the "read at
   decision time" claim, and it is the one step that proves the no-restart
   behaviour rather than trusting my reading of the code.

**Why the running app will not show this yet:** the process currently running
(`/Applications/SeatLoom.app`, pid from the earlier census) is the previously
installed binary. It has to be rebuilt and reinstalled to carry this change. I did
**not** rebuild, reinstall or restart it — stopping the operator's running app is
not mine to do, and the acceptance card's build
(`scripts/prepare-verification-build.sh`) is the sanctioned path that installs and
launches it. **The card and the build must arrive together**; a card whose build
does not carry the fix is a card he cannot pass.

## 9. Known limits, stated rather than hidden

- **`$HOME` must be set.** If it is not, the file cannot be located and the refusal
  says so rather than inventing a path. The environment remains a channel.
- **The file is one line.** A ref containing a newline is not expressible; a ref
  with an interior space is carried verbatim (trimmed at the edges only), because
  silently altering a ref would be worse than carrying an odd one.
- **Precedence is file > env, which is the dispatch's wording.** If Aegis rules the
  other way, it is a two-line change in one place — but note the consequence:
  `scripts/launch-seatloom.sh` sets the env var, and under file-first that becomes
  the fallback rather than the primary. That is the intended end state ("this is not
  the terminal fix"), and the script keeps working either way.
- **The click test in §8 is the operator's.** I measured the resolution logic
  against the real file; I did not launch the GUI, and I am not claiming the app
  was exercised.

## 10. Out of scope, unchanged

The acceptance card's state machine, the merge logic itself, the live database, and
every other command. The only behavioural change is *where the operator identity
comes from*, and the refusal message when it comes from nowhere.

## 11. Forward recording (D5 Phase 2 Lane B)

One memory, `source_kind=canonical_event`, `source_ref` = the delivery
announcement, `confidence=quoted`, topic `topic-crate-integration-reverts-track-fixes`:

| Memory | Kind | Why it passes the three-way test |
|---|---|---|
| `mem-1DrHWyFxWYQy9dOwlPliv` | decision | Crate-level integration protects against intra-branch conflicts but **not** against track-side fixes made after the packet branch forked. Changes what a future integrator checks before taking a crate wholesale from a packet branch; exists nowhere else; evidenced by `2339223` fixed, `1e7b254` reverted, both dated. |

Nothing else was recorded. The four refusal states and the `$HOME`-message bug are
consequences of this one decision rather than new facts, and no preference was
touched — Mr Zhang's corrections are Aegis's to record.
