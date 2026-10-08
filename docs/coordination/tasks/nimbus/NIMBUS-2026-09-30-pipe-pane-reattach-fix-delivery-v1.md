# Delivery: Fix the tmux mirror re-attach — `pipe-pane -o` is a toggle

| Field | Value |
|---|---|
| template | T3 |
| subtype | implementation_delivery |
| id | NIMBUS-2026-09-30-pipe-pane-reattach-fix-delivery-v1 |
| status | delivered |
| author | nimbus |
| date | 2026-09-30 |
| version | v1 |
| to | flux (Layer A verify), lyra (acceptance) |
| packet_ref | `docs/coordination/tasks/nimbus/NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1.md` |
| reported_by | Mira, from UM0 — `MIRA-2026-09-30-live-mirror-um0-delivery-v1.md` §0.1, without touching Rust |
| base branch | `track/infra-foundation` @ `11b5323` |
| delivery branch | `packet/nimbus/NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1` |
| delivery commit SHA (code) | **`abe4245`** — gate Layer A against this |
| build location | `/Users/jyxc-dz-0100609/Documents/GitHub/seatloom-worktrees/nimbus` |
| tags | nimbus, pty, tmux, live-mirror, um0, bugfix, delivery |

---

## 1. Scope — 2 files, +453/−8

```
$ git show --stat abe4245
commit abe4245911c15c125a8c4f56656aa2a4278b6bf9
Author: Xiaolong Zhang <zxlvip@gmail.com>
Date:   Wed Sep 30 23:46:15 2026 +0800

    fix(pty): drop pipe-pane -o so re-attach does not kill the mirror
    
    `-o` TOGGLES. The comment at pty/mod.rs:271 claimed it "makes the
    pipe-pane command invoke only once"; that reading was wrong. Measured:
    
        initial pane_pipe  = 0
        after 1st  -o call = 1
        after 2nd  -o call = 0      <- the pipe CLOSED
        after plain (no -o)= 1
        after 2nd plain    = 1      <- replaces, stays open
    
    So the first attach opened the mirror and the SECOND silently closed it:
    pane_pipe=0, zero bytes, and attach still returned a successful session.
    Any page reload, second viewer, or already-mirrored seat hit it. Found by
    Mira building UM0, reproduced by Lyra, reproduced again here before the
    change. Plain `pipe-pane <cmd>` replaces unconditionally — exactly
    re-attach semantics. The comment now carries the measurement so the flag
    is not restored.
    
    Also: attach no longer reports success for a mirror that never opened. A
    zero exit from pipe-pane does not mean the pane is piped, which is how
    this bug stayed invisible. attach_tmux now probes #{pane_pipe} and
    returns the new PtyError::MirrorNotOpen, cleaning up the FIFO and any
    half-open pipe. The decision logic is the pure fn verify_pipe_open with
    four unit tests, because the failure path cannot be induced end-to-end
    from outside attach_tmux.
    
    Stated, not overclaimed: this catches a pipe that NEVER OPENED. It does
    not catch a pipe whose command dies, because tmux holds pane_pipe=1 until
    it first WRITES to the dead pipe (measured; see the fn doc). Closing that
    gap would mean writing to a live seat's pane. Follow-up, not claimed.
    
    kill() assessed per packet §2.2: pipe-pane is per-PANE, not per-client, so
    one client detaching stops the mirror for every other client on that pane.
    That is tmux's model, not a bug here; documented on PtySession::kill and
    recommended as a follow-up packet rather than redesigned.
    
    99 lib tests. Proof harness runs on a throwaway tmux session it creates
    and kills; no seat touched, all 10 sessions left at pane_pipe=0.
    
    Packet: docs/coordination/tasks/nimbus/NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1.md
    
    Co-Authored-By: Claude Opus 5 (1M context) <noreply@anthropic.com>

 .../examples/pipe_pane_reattach_proof.rs           | 261 +++++++++++++++++++++
 crates/seatloom-core/src/pty/mod.rs                | 200 +++++++++++++++-
 2 files changed, 453 insertions(+), 8 deletions(-)
```

Zero `.tsx`, zero `.sql`, zero files under `ui/` or `infra/`. One source file changed
(`crates/seatloom-core/src/pty/mod.rs`) plus one new proof harness.

---

## 2. The bug — reproduced here before changing anything

I reproduced it independently rather than taking the packet's measurement secondhand, on a
throwaway tmux session created and killed for the purpose:

```
session=nimbus-throwaway-79894
initial pane_pipe      = 0
after 1st  -o call     = 1
after 2nd  -o call     = 0   <- Lyra measured 0 here
after plain (no -o)    = 1
after 2nd plain        = 1   <- must stay 1
after bare (detach)    = 0
```

Identical to Lyra's reproduction and to Mira's report. `-o` **toggles**. The first attach opened
the mirror; the **second closed it** — `pane_pipe=0`, zero bytes, and `cmd_attach_tmux_session`
still returned a successful `LiveSessionDto`. It looked live and was dead.

---

## 3. The three changes

### 3.1 `-o` removed from the attach path

`crates/seatloom-core/src/pty/mod.rs` — the only functional line of the fix:

```diff
-            .args(["pipe-pane", "-t", &tmux_target, "-o", &cat_cmd])
+            .args(["pipe-pane", "-t", &tmux_target, &cat_cmd])
```

The misleading comment is replaced with the measured table above and an explicit *"Do not
'restore' `-o`"*, so the flag is not reinstated by someone reading the man page the way all three
of us first did. The `attach_tmux` doc comment (which also documented `-o`) is corrected, and now
states that re-attach is safe and idempotent.

### 3.2 Detach path assessed (§2.2) — shared pipe confirmed, follow-up recommended

`PtySession::kill` issues `pipe-pane -t <target>` with no command and no `-o`. Confirmed: that
**unconditionally disables** copying, which is correct for detach and is *not* a toggle.

**The consequence, stated:** tmux keeps **one pipe per pane, not one per consumer**. If two
SeatLoom clients mirror the same pane, whichever detaches first **stops the mirror for both**. The
survivor keeps a `PtySession` whose reader simply goes quiet — the same silent-dead-mirror shape
§3.3 now catches at attach time, but arriving later, so attach-time verification cannot detect it.

Demonstrated incidentally in PROOF 3: three attaches to one pane shared a single pipe, and the
first `kill()` closed it for all three.

This is a property of tmux, not a defect in `kill()` — disabling is right for a single-client
detach, and there is no per-client pipe to disable instead. **Recommendation: yes, it needs a
follow-up packet.** Fixing it requires an ownership model (ref-counting attachers, or one pipe
fanned out to per-client FIFOs). Per §3 of the packet I did **not** build it. Documented on
`PtySession::kill` so the next reader finds it at the code.

### 3.3 A dead mirror is now an error, not a success

`attach_tmux` probes `#{pane_pipe}` after issuing `pipe-pane` and returns a new typed error:

```rust
    /// `tmux pipe-pane` exited 0 but the pane is not actually piped. …
    #[error(
        "tmux mirror did not open for {target}: pipe-pane exited 0 but pane_pipe={pane_pipe} \
         (expected 1). The mirror would have reported success and produced no output."
    )]
    MirrorNotOpen { target: String, pane_pipe: String },
```

On failure it stops any half-open pipe and unlinks the FIFO, so a failed attach leaks neither.

The decision is the pure function `verify_pipe_open(target, status_ok, pane_pipe_raw)`, extracted
**so the failure path is testable** — it cannot be induced end-to-end from outside `attach_tmux`,
which creates its own FIFO first. Four unit tests cover open, closed, empty/garbage, and a failed
probe.

> **What this catches, and what it does not — measured, not assumed.**
> It catches a pipe that **never opened**. It does **not** catch a pipe whose command starts and
> then dies, because tmux holds `pane_pipe=1` until it first *writes* to the dead pipe (PROOF 4a).
> The attach-time probe runs immediately after `pipe-pane`, so it is too early to see that. Closing
> the gap would mean writing to the user's pane to force a flush — unacceptable on a live seat. A
> dying pipe therefore still surfaces only as "no bytes arriving", which belongs with the §3.2
> follow-up. This is a strict improvement over reporting success unconditionally, **not a complete
> one**, and I would rather say so than let the error type imply more than it delivers.

---

## 4. Layer A — verbatim

### 4.1 `cargo check -p seatloom-core`

```
$ cargo check -p seatloom-core
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.46s
[exit 0]
```

### 4.2 `cargo test -p seatloom-core --lib` — 99 passed

```
$ cargo test -p seatloom-core --lib
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.34s
     Running unittests src/lib.rs (target/debug/deps/seatloom_core-767388bf53ce3b11)

running 99 tests
test db::document_parser::tests::ignores_header_examples_inside_code_fences ... ok
test db::document_parser::tests::header_subtype_valid_for_real_doc ... ok
test db::document_parser::tests::parses_doc_id_and_status ... ok
test db::document_parser::tests::extracts_title_from_h1 ... ok
test db::document_parser::tests::parses_author_date_version ... ok
test db::document_parser::tests::normalizes_short_template_aliases ... ok
test db::document_parser::tests::parses_depends_on_list ... ok
test db::document_parser::tests::parses_supersedes_and_tags ... ok
test db::document_parser::tests::parses_template_and_subtype ... ok
test db::document_parser::tests::slugify_produces_lowercase_dashes ... ok
test db::document_parser::tests::slugify_strips_leading_trailing_dashes ... ok
test db::document_parser::tests::validates_all_template_families ... ok
test db::document_parser::tests::validates_subtype_t1_prd ... ok
test db::document_parser::tests::returns_none_for_doc_without_header ... ok
test db::document_parser::tests::validates_subtype_t3_task ... ok
test db::event_domain::tests::all_domains_round_trip ... ok
test db::event_domain::tests::artifact_produced_is_not_in_the_vocabulary ... ok
test db::event_domain::tests::casing_is_significant_and_domain_specific ... ok
test db::event_domain::tests::serialized_sets_match_schema_010_checks ... ok
test db::document_parser::tests::extracts_sections_with_levels ... ok
test db::event_domain::tests::unknown_values_are_typed_errors ... ok
test db::document_parser::tests::sections_have_deterministic_ordinals ... ok
test db::document_parser::tests::extracts_section_bodies ... ok
test db::reconcile::tests::changed_digest_detects_change ... ok
test db::reconcile::tests::is_allowed_ingest_path_accepts_docs_coordination ... ok
test db::reconcile::tests::is_allowed_ingest_path_accepts_docs_root ... ok
test db::reconcile::tests::is_allowed_ingest_path_rejects_non_docs ... ok
test db::reconcile::tests::is_allowed_ingest_path_rejects_non_md ... ok
test db::reconcile::tests::sha256_hex_changes_on_body_change ... ok
test db::reconcile::tests::sha256_hex_is_deterministic ... ok
test db::reconcile::tests::unchanged_digest_detects_no_change ... ok
test db::reconcile::tests::path_to_doc_id_slugifies_correctly ... ok
test db::repositories::tests::null_harness_and_agent_are_accepted ... ok
test db::repositories::tests::out_of_domain_harness_fails_loudly ... ok
test db::repositories::tests::out_of_domain_agent_fails_loudly_naming_the_row ... ok
test db::repositories::tests::out_of_domain_seat_default_agent_fails_loudly ... ok
test db::repositories::tests::rendered_error_names_table_row_column_and_value ... ok
test db::repositories::tests::seeded_values_hydrate ... ok
test db::write_api::tests::blank_ref_id_is_rejected ... ok
test db::write_api::tests::inserts_name_the_columns_010_defaults_would_otherwise_fill ... ok
test db::write_api::tests::append_with_no_refs_at_all_is_rejected ... ok
test db::write_api::tests::blank_required_fields_are_rejected ... ok
test db::write_api::tests::exactly_one_subject_is_accepted_and_returned ... ok
test db::write_api::tests::schema_version_below_one_is_rejected ... ok
test db::write_api::tests::payload_must_be_a_json_object ... ok
test db::write_api::tests::seeded_historical_payload_is_provably_unreplayable ... ok
test db::write_api::tests::turn_seq_domain_is_enforced_without_conflating_none_and_zero ... ok
test db::write_api::tests::two_subjects_are_rejected_in_rust_before_the_index_fires ... ok
test db::write_api::tests::workitem_created_v1_payload_round_trips ... ok
test db::write_api::tests::workitem_status_changed_v1_payload_round_trips ... ok
test db::write_api::tests::zero_subject_append_is_rejected ... ok
test objects::seat::tests::every_seeded_role_value_parses ... ok
test objects::seat::tests::custom_role_round_trips ... ok
test objects::seat::tests::unknown_role_is_a_typed_error_not_a_custom_fallback ... ok
test objects::seat::tests::serde_shape_is_snake_case ... ok
test objects::session::tests::agent_domain_round_trips ... ok
test objects::session::tests::dimensions_are_not_interchangeable ... ok
test objects::session::tests::harness_domain_round_trips ... ok
test objects::session::tests::model_id_is_opaque_free_form_text ... ok
test objects::session::tests::serde_shape_is_pascal_case_and_externally_tagged ... ok
test objects::session::tests::unknown_agent_is_a_typed_error_not_a_custom_fallback ... ok
test ledger::index::tests::rebuilds_lookup_maps_from_seeded_events ... ok
test objects::session::tests::unknown_harness_is_a_typed_error ... ok
test pty::tests::default_transcripts_dir_nests_under_repo_root ... ok
test pty::tests::mirror_dir_is_tmp_seatloom_mirror ... ok
test pty::tests::shell_quote_handles_spaces_and_quotes ... ok
test pty::tests::verify_pipe_open_accepts_only_an_open_pipe ... ok
test pty::tests::verify_pipe_open_rejects_a_failed_probe_even_if_stdout_says_one ... ok
test pty::tests::verify_pipe_open_rejects_exit_zero_with_closed_pipe ... ok
test pty::tests::verify_pipe_open_rejects_unreadable_pane_pipe ... ok
test pty::tests::write_noop_on_empty_bytes ... ok
test pty::tests::write_noop_on_shutdown ... ok
test storage::artifact_store::tests::subtype_validation_follows_allowlist ... ok
test storage::artifact_store::tests::resolves_artifact_paths_deterministically ... ok
test storage::handoff_store::tests::resolves_handoff_paths_deterministically ... ok
test storage::handoff_store::tests::missing_handoffs_dir_returns_empty_vec ... ok
test storage::artifact_store::tests::missing_artifacts_dir_returns_empty_vec ... ok
test storage::seat_registry::tests::delegation_validation_rejects_required_boundary_gaps ... ok
test db::reconcile::tests::collect_markdown_files_returns_sorted_paths ... ok
test pty::tests::list_tmux_sessions_returns_vec_without_panic ... ok
test storage::seat_registry::tests::missing_delegations_dir_returns_empty_vec ... ok
test storage::seat_registry::tests::missing_seats_dir_returns_empty_vec ... ok
test storage::seat_registry::tests::resolves_seat_layer_paths_deterministically ... ok
test storage::jsonl_io::tests::jsonl_append_and_read_round_trip ... ok
test storage::seat_registry::tests::delegation_round_trips ... ok
test storage::seat_registry::tests::save_delegation_returns_typed_validation_error ... ok
test storage::session_store::tests::resolves_session_paths_deterministically ... ok
test storage::session_store::tests::missing_sessions_dir_returns_empty_vec ... ok
test storage::handoff_store::tests::handoff_round_trip_and_list ... ok
test storage::workitem_store::tests::resolves_workitem_paths_deterministically ... ok
test storage::artifact_store::tests::artifact_round_trip_and_list ... ok
test storage::workitem_store::tests::missing_workitems_dir_returns_empty_vec ... ok
test storage::seat_registry::tests::list_seat_identities_sorted_by_name ... ok
test storage::seat_registry::tests::list_delegations_sorted_by_issued_at_descending ... ok
test storage::seat_registry::tests::role_binding_round_trips ... ok
test storage::seat_registry::tests::seat_identity_round_trips ... ok
test storage::yaml_io::tests::project_config_round_trips_through_yaml ... ok
test storage::session_store::tests::session_round_trip_and_list ... ok
test storage::workitem_store::tests::workitem_round_trip_and_list ... ok

test result: ok. 99 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
[exit 0]
```

### 4.3 `cargo clippy -p seatloom-core -- -D warnings`

```
$ cargo clippy -p seatloom-core -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.26s
[exit 0]
```

Also run at the ENGINEERING_STANDARDS scope, which is stricter than the packet's list:

```
$ cargo clippy -p seatloom-core --all-targets -- -D warnings
[exit 0]
```

### 4.4 `cargo fmt --all --check`

```
$ cargo fmt --all --check
[exit 0]
```

### 4.5 `cargo check -p seatloom-tauri`, `tsc --noEmit`, `pnpm build`

Not in the packet's §4 list; run anyway, since the attach path is reached from `src-tauri`.

```
$ cargo check -p seatloom-tauri
[exit 0]     (3 pre-existing dead_code warnings on *Dto structs, untouched here)

$ cd ui && pnpm exec tsc --noEmit
[exit 0]

$ cd ui && pnpm build
[exit 0]
```

---

## 5. Behavioural proof

`crates/seatloom-core/examples/pipe_pane_reattach_proof.rs`. It exercises the **shipped Rust attach
path** (`PtySession::attach_tmux`), not raw tmux commands, on a throwaway session it creates and
kills. It refuses to start if that session name already exists, and kills it on every exit path.

```
$ cargo run -q -p seatloom-core --example pipe_pane_reattach_proof
throwaway tmux session = nimbus-pipeproof-95706
(this harness creates and kills ONLY this session; no seat is touched)

initial pane_pipe = 0

########## PROOF 1 — first attach: pane_pipe=1 and bytes flow ##########
  bytes delivered = 249, marker "NIMBUS_MARK_ONE" seen = true
  pane_pipe after 1st attach = 1
HELD

########## PROOF 2 — SECOND attach: pane_pipe STAYS 1 and bytes STILL flow ##########
  (with `-o` this step left pane_pipe=0 and the mirror silently dead)
  bytes delivered = 277, marker "NIMBUS_MARK_TWO" seen = true
  pane_pipe after 2nd attach = 1
  bytes delivered = 281, marker "NIMBUS_MARK_THREE" seen = true
  pane_pipe after 3rd attach = 1
HELD: re-attach is idempotent, not a toggle.

########## PROOF 3 — detach: pipe closes, FIFO unlinked, tmux session still ALIVE ##########
  fifo (3rd attach) = /tmp/seatloom-mirror/proof-attach-3.fifo
  exists before kill = true
  pane_pipe after kill  = 0   (expect 0)
  fifo exists after kill = false
  tmux session still alive = true
  fifo(1) exists = false, fifo(2) exists = false
HELD. NOTE (packet §2.2): the pipe is per-PANE, so these three attaches shared ONE
     pipe and the first kill stopped the mirror for all of them. Documented on
     PtySession::kill; a follow-up packet is recommended, not built here.

########## PROOF 4 — a mirror that cannot open is an Err, never a silent dead session ##########
  4a. Measuring what tmux actually does, because my first reading was wrong.
      A pipe command that dies does NOT immediately clear pane_pipe — tmux keeps
      the flag set until it first tries to WRITE to the dead pipe:
      tmux pipe-pane <unwritable path>  -> exit=0
        +300ms pane_pipe=1
        +600ms pane_pipe=1
        +900ms pane_pipe=1
      now force pane output, which makes tmux write to the dead pipe:
        +300ms pane_pipe=0
        +600ms pane_pipe=0
      CONSEQUENCE, stated plainly: the attach-time probe runs immediately after
      pipe-pane, so it CANNOT see a pipe that dies on first write. It catches a pipe
      that never opened. Closing the remaining gap would mean writing to the user's
      pane to force a flush, which is unacceptable on a live seat. Recorded as a
      follow-up, not silently claimed as fixed.

  4b. verify_pipe_open() rejects exactly that combination. Unit tests:
      cargo test -p seatloom-core --lib -- pty::tests::verify_pipe_open
      (exit 0 + pane_pipe=0, empty, garbage, and a failed probe are all Err)

  4c. End-to-end: attach to a target that cannot be piped at all.
      tmux error: tmux pipe-pane nimbus-pipeproof-nonexistent-session:0: can't find session: nimbus-pipeproof-nonexistent-session

      NOTE, stated rather than glossed: this case is caught by the PRE-EXISTING
      non-zero-exit guard, so it does NOT exercise MirrorNotOpen. Driving the new
      check end-to-end needs a pane where pipe-pane exits 0 and the pipe still
      fails to open — 4a shows that state is reachable, but attach_tmux creates
      its own FIFO first, so it cannot be induced from outside without a test
      seam. That is why the decision logic is a pure function with its own
      tests (4b) instead of being asserted only through the happy path.
HELD: every unopenable mirror is an Err; none is reported as a live session.

cleanup: throwaway session killed; still exists = false

########## ALL PROOFS HELD ##########
[exit 0]
```

**Reading it:**

- **PROOF 1** — first attach: `pane_pipe=1`, 277 bytes delivered, marker seen.
- **PROOF 2 — the regression that must not come back.** Second attach keeps `pane_pipe=1` **and
  bytes still flow**. A third attach too: the old bug was an alternating toggle, so the *even*
  attach was the failing case, and stopping at two would have been a weaker test than it looked.
- **PROOF 3** — detach closes the pipe (`pane_pipe=0`), unlinks the FIFO, and **leaves the tmux
  session alive** (R3 failure isolation). Also shows the §3.2 shared-pipe behaviour.
- **PROOF 4** — see §3.3. 4a is the measurement that corrected my own first claim; 4c states
  plainly that it is the *pre-existing* non-zero-exit guard firing, **not** `MirrorNotOpen`, so
  nobody reads it as end-to-end coverage of the new check.

### 5.1 A correction to my own evidence

My first version of PROOF 4 asserted that `tmux pipe-pane "cat >> /nonexistent-dir/x.fifo"` gives
`exit=0, pane_pipe=0`, because a standalone probe showed exactly that. Re-run inside the harness it
showed `pane_pipe=1`, contradicting it.

The reason is timing, and it matters: tmux clears `pane_pipe` only when it first **writes** to the
dead pipe. My standalone probe used a freshly created session that was still emitting shell startup
output, which triggered the write; the harness's pane was idle, so nothing did.

```
  tmux pipe-pane <unwritable path>  -> exit=0
    +300ms pane_pipe=1       <- child already dead, flag still 1
    +900ms pane_pipe=1
  send-keys "echo forcing_a_write"  (forces a write)
    +300ms pane_pipe=0       <- only NOW does tmux clear it
```

I had written the wrong explanation into the code comment, the unit-test comment and the proof. All
three are corrected. This is also what bounds the new check (§3.3) — had I not re-measured, the
delivery would have claimed a guarantee the code does not provide.

---

## 6. tmux left exactly as found

**Baseline, before any work** (10 sessions, every one `pane_pipe=0`):

```
Flux-Quality&Ops-seatloom    pane_pipe=0
Lyra-po-seatloom             pane_pipe=0
Mira-UX/UED-seatloom         pane_pipe=0
Nimbus-TechArchi-seatloom    pane_pipe=0
Onyx-data-seatloom           pane_pipe=0
aegis-feishu                 pane_pipe=0
cp-alpha-ws                  pane_pipe=0
ebook_guardian_web           pane_pipe=0
hwfs-test                    pane_pipe=0
ss-ws-test                   pane_pipe=0
session count = 10
```

**After all work — identical:**

```
Flux-Quality&Ops-seatloom    pane_pipe=0
Lyra-po-seatloom             pane_pipe=0
Mira-UX/UED-seatloom         pane_pipe=0
Nimbus-TechArchi-seatloom    pane_pipe=0
Onyx-data-seatloom           pane_pipe=0
aegis-feishu                 pane_pipe=0
cp-alpha-ws                  pane_pipe=0
ebook_guardian_web           pane_pipe=0
hwfs-test                    pane_pipe=0
ss-ws-test                   pane_pipe=0
session count = 10
```

No seat session was killed, restarted, or attached to. Every tmux command I ran targeted a
throwaway session named `nimbus-throwaway-*`, `nimbus-probe-*`, `nimbus-race-*` or
`nimbus-pipeproof-*`, each killed immediately after use.

**Note on the five-seat count.** The packet says five seats; I found **five seat sessions**
(Flux, Lyra, Mira, Nimbus, Onyx) among ten tmux sessions total. There is no
`Aegis-Supervisor-seatloom` session — `aegis-feishu` exists instead. Not touched, and not this
packet's business, but recorded since the packet's "all five seats" check is what I was asked to
confirm and the session list is what I actually saw.

**Leftover FIFOs in `/tmp/seatloom-mirror/` are not mine.** Seven `ses-*.fifo` entries dated
23:08–23:20 pre-date this work and belong to the running app / Mira's UM0 session. None of my
`proof-attach-*` FIFOs remain — PROOF 3 shows all three unlinked. I left the `ses-*` ones alone.

---

## 7. Done-definition (§5)

| # | Criterion | Status |
|---|---|---|
| 1 | `-o` removed; comment corrected with measured semantics | **PASS** — §3.1 |
| 2 | Detach assessed; shared-pipe consequence stated; follow-up recommended | **PASS** — §3.2, follow-up **recommended** |
| 3 | Attach returns a typed error when the pipe is not open | **PASS** — §3.3, `PtyError::MirrorNotOpen`, with its limits stated |
| 4 | Re-attach proven: second attach keeps `pane_pipe=1`, bytes flow | **PASS** — PROOF 2, with a third attach too |
| 5 | Tmux left clean; no seat killed; all seats back at `pane_pipe=0` | **PASS** — §6 |
| 6 | Layer A green, verbatim | **PASS** — §4 |
| 7 | Zero `.tsx`, zero `.sql` | **PASS** — §1 |
| 8 | Flux commit-pinned verify PASSes | **PENDING** — gate `abe4245` |

---

## 8. Follow-ups recommended (not built here)

1. **Multi-client pipe ownership** (§3.2) — one pane, one tmux pipe, so any client's detach stops
   every client's mirror. Needs ref-counting or per-client FIFO fan-out. Explicitly out of scope
   per packet §3.
2. **Liveness detection for a pipe that dies after opening** (§3.3) — not reachable by the
   attach-time probe. Natural home is the same packet as (1), since both are "the mirror stopped
   and nothing told anyone".

Neither blocks UM0: the reported bug — second attach kills the mirror — is fixed and proven.

---

## 9. Constraints honoured

- Built only in the Nimbus worktree, on the packet branch. Shared checkout untouched.
- Packet file absent at `11b5323`; read via
  `git show track/infra-foundation:docs/coordination/tasks/nimbus/NIMBUS-2026-09-30-pipe-pane-reattach-fix-v1.md`.
- **No seat tmux session killed, restarted or attached.** Throwaway sessions only.
- Zero `.tsx` — Mira owns `ui/src`. No multi-client ownership model. No `.sql`. No write-API change.

---

*Nimbus · 2026-09-30 · reported by Mira from UM0 without touching Rust · reproduced by Lyra and
again here before the fix · gates reliable UM0*
