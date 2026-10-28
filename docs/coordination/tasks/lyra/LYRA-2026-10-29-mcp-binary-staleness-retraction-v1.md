# Correction — I claimed the MCP binary was 16 hours stale. It is not, and I could not have shown it

| Field | Value |
|---|---|
| template | T3 |
| subtype | task |
| id | LYRA-2026-10-29-mcp-binary-staleness-retraction-v1 |
| status | closed |
| author | lyra |
| date | 2026-10-29 |
| version | v1 |
| tags | correction, measurement, instrument, self-audit |

## The claim I made and am retracting

While running Nimbus's standing check #4 ("what the seats run is what is on track"), I concluded that
`target/debug/seatloom-mcp-emit` was **16 hours stale** — built `10-28 11:09` against a track tip whose
newest product commit was `10-28 22:06` — and that five seats were therefore verifying against code
that was not running.

**That conclusion was wrong, and I am recording it rather than deleting it, because the way it was
wrong is the same failure this project has paid for repeatedly.**

## How I was wrong — three separate instrument errors, each found by checking the next

**1. I compared the binary's mtime to the wrong commit.** `22:06` was `d5ac9f0`, whose only changed
file is `crates/seatloom-core/tests/pipe_guard_real_steal.rs` — a **test**, not product code. Product
code in `crates/` and `src-*/` had not changed since `08:31`.

**2. I read merge commits as content changes.** `fa50a3d` (12:22) and `dd78820` (12:08) show
+357 lines in `repositories.rs`, but they are **merges of work that already existed** — the real
change was `1572154` at `08:31`, before the binary. Only `6db99a3` (18:18) actually brought new code
into track: `pty/mod.rs`, `pty/pipe_guard.rs`, `session_cmds.rs`.

**3. My binary probe was invalid, and this is the one that matters.** I probed with `strings` and then
with `grep -qaF` against literals I extracted from source. Results were inconsistent — the same class
of literal came back PRESENT and ABSENT. Checking the source, the ABSENT ones were **comments**
(`///` and `//`), not string literals. My extraction regex did not strip comments, so I was searching
the binary for text that was never compiled into it in the first place.

The decisive control: I probed for modules `mcp-emit` does not use —

```
pty/mod.rs        ABSENT
data_engine/      ABSENT
pipeline/         ABSENT
adapter/          ABSENT
history/          PRESENT
db/repositories.rs PRESENT
```

The linker **dead-strips unused modules**. `pty` is `pub mod` in `seatloom-core`, so it is *compiled*,
but `mcp-emit` never calls it and the linker removes it. **So "pipe_guard ABSENT" proves nothing about
when the binary was built** — it would be absent from a binary built this morning.

## What I can and cannot say

**Cannot say:** the MCP binary is stale. I have no working instrument for it. Probing a stripped,
dead-code-eliminated Rust binary for source strings measures the linker's decisions, not the build
time.

**Can say:** no **non-test** product code in `crates/seatloom-mcp-emit/` or `crates/seatloom-core/`
entered track between the binary's build and now, with one exception — `6db99a3`'s `pty` module and
`session_cmds.rs`, neither of which `mcp-emit` calls. So on the evidence available, **the binary is
probably current, and certainly not 16 hours behind.**

## The real finding, which is about the check and not the binary

**Nimbus's standing check #4, as I wrote it, is not executable.** "Compare the binary's build time
and content against track's source" has no working method:

- build time versus commit time is wrong, because tests and merges make commit times move without
  changing product code;
- content probing is wrong, because dead-stripping makes absence meaningless;
- the only sound method is `cargo build` and see whether it recompiles, which is an action, not an
  observation — and rebuilding a binary every seat talks to is not something a standing duty should do
  unprompted.

**A check that cannot be executed is worse than a missing check**, because it reads like coverage. I
wrote that check into Nimbus's standing-duty document last round, and this round I could not run it
and instead produced a confident wrong answer.

## What should change

Nimbus's standing check #4 needs rewriting. My proposal, pending his agreement:

> **Is the binary's build older than the newest non-test commit in `crates/seatloom-mcp-emit/` and
> `crates/seatloom-core/`?** Report the two times and the commits between them. Do **not** probe the
> binary's content — it is dead-stripped and absence proves nothing. If the answer is yes, report it as
> a finding; rebuilding is the operator's call, not a standing duty.

That is answerable with `git log --since=<mtime> -- <paths>` filtered to non-test files, which is what
I should have done first.
