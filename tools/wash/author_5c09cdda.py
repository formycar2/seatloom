"""Author S3 records for claude-code/5c09cdda (Flux verification session)."""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from vela_wash.s3_extract import RecordStore  # noqa: E402

AH = os.path.expanduser("~/Documents/GitHub/AgentHistory")
S = "5c09cdda-ec7d-4881-89af-2a7c00b53c68"
F = "raw/claude-code/5c09cdda-ec7d-4881-89af-2a7c00b53c68.jsonl"
store = RecordStore(os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                 ".local/wash/s3/records.jsonl"), AH)


def add(kind, conf, title, l0, l1, a, b, topic="t000", tags=None):
    r = store.add(kind=kind, confidence=conf, title=title, l0_summary=l0,
                  l1_overview=l1, session_id=S, harness="claude-code",
                  topic_id=topic, l2_file_path=F, l2_line_start=a, l2_line_end=b,
                  evidence_seqs=[], tags=tags or [])
    return r["record_id"][:8]


add("failure", "quoted",
    "A live dev watcher on the shared tree breaks commit-pinned verification gates",
    "pnpm tauri dev running 30+ hours against the shared checkout regenerated a tracked file (src-tauri/gen/schemas/capabilities.json) the moment a gate checked out a different commit - the checkout changed src-tauri sources, the watcher rebuilt, and the tree went dirty mid-gate, continuously.",
    "Flux refused to kill the operator's dev server, revert the file, or run the destructive baseline verifier; it gathered the unaffected static evidence and returned HOLD with the cause named. The durable lesson recorded as memory: a live watcher on the shared tree makes the tree unpinnable, so commit-pinned gates must run in a worktree (or with the watcher stopped). Untracked machine-local dirs appearing after checkout are a branch-artifact of differing .gitignore, not dirtiness - distinguish tracked modifications from untracked artifacts before calling a HOLD.",
    121, 247, tags=["verification", "infrastructure", "failure"])

add("decision", "quoted",
    "Scratch-database substitution finds what live-DB observation hides",
    "Observing the live database shows a state; building from an empty scratch database proves the migration order works. The substitution exposed a real regression (apply-db.sh cannot build the merge candidate from scratch: the seed needs migration 009 pre-seed while the script buckets all >=006 post-seed) that live observation had hidden.",
    "The verdict flipped from HOLD to FAIL on this evidence, and the control experiment decided blame: apply-db.sh on un-merged track builds green (exit 0) while the merge candidate fails (exit 3, column default_harness does not exist) - a regression the merge would introduce, not a pre-existing condition. The remedy was then verified (009 pre-seed builds clean, exit 0) before the finding was written up.",
    353, 418, tags=["verification", "database", "migration"])

add("decision", "quoted",
    "Prove trap-avoidance from the result, not the procedure",
    "When a packet warns that a reused database would silently keep an old constraint (IF NOT EXISTS) and yield a false PASS, the decisive evidence is the observed result - the constraint read back from the fresh database carries exactly the new values - not the recitation of the fresh-build procedure.",
    "Applied to the 12-value event_type vocabulary: a reused database would have shown 13 values; reading back 12 (with the dropped value absent) proves the new constraint is in force. The negative test then used the dropped value itself as the probe, so its rejection simultaneously proves the new constraint is the one under test. Idempotency was likewise measured, not assumed: before/after snapshots of 43 schema facts plus row counts, identical across two further applies.",
    823, 950, tags=["verification", "database", "testing"])

add("failure", "quoted",
    "Flux-local PASS is not CI green: the frontend ratchet is file-granular",
    "CI's frontend:lint does not run eslint . - it runs scripts/lint-changed.sh, a ratchet over changed ui/src files. The ratchet is file-granular, so merely touching a pre-existing-dirty file pulls its debt into the gate: three edited files were already unformatted at base and all 15 App.tsx eslint errors sit outside the author's hunks, yet lint-changed.sh exits 1.",
    "Established by comparing error lines against the actual diff hunks and by counting any occurrences at base vs head (identical 15) - zero regression, real gate failure. The packet had asked for the repo-wide run; the project's actual standard is the ratchet, and running the wrong command produces the wrong verdict in both directions. Recorded as memory because the asymmetry (a delivery can be blameless and still unmergeable) recurs.",
    1291, 1315, tags=["ci", "verification", "failure"])

add("decision", "quoted",
    "Prove liveness additively: a second instance on a spare port, never disturbing the running app",
    "To prove a live mirror without disturbing the operator's running app, launch an additive second Tauri instance from your own worktree against a spare vite port (1421, since strictPort holds 1420), then prove the chain: pane_pipe flips 0 to 1 only for panes your UI attached, transcript files accrue real bytes, and the mirrored stream contains your own session's verbatim output.",
    "The decisive evidence was reading the mirrored FIFO back and finding this very session's text with ANSI positioning intact, growing 59k to 60k to 66k bytes - proof that the UI drove the attach and bytes genuinely flowed. Honest bounds were stated: the render layer could not be screenshotted (no patchright on the host), and Bash-tool output does not render into a pane's TTY, which is why a planted marker never appeared - a property of the execution environment, not of the mirror.",
    1398, 1510, tags=["verification", "ui", "testing"])

add("decision", "quoted",
    "Verify database state independently instead of trusting the harness's printout",
    "Running someone else's proof harness and reading its HELD lines is still trusting the harness; re-check the resulting state with your own queries, and prove derivability by reading the raw events yourself (create event supplies title/priority/owner, the later status event supplies status=done, updated_at tracks the latest).",
    "Also: respect other seats' scratch databases (Nimbus's seatloom_s2_nimbus was neither reused nor dropped), and prove the live database was never written by showing it lacks the migration's columns entirely (the missing project_id error is itself evidence that 008/010 never landed there). The structural check was hardened too: single-line grep could miss multi-line SQL, so every mention of each projection table was enumerated.",
    1055, 1141, tags=["verification", "database", "discipline"])

add("failure", "quoted",
    "A guard test that compares Rust against a hardcoded transcription is one-directional",
    "serialized_sets_match_schema_010_checks asserts the Rust enums against a hardcoded vec literal transcribed from migration 010 - it catches the enum drifting from the transcription, but nothing reads the .sql, so both could drift from the real schema together.",
    "Disclosed as a precision on a PASS that had already been earned, then closed for the day by comparing the Rust lists against the live database constraints directly (12 / 3 / 9, exact match including order). The durable fix recorded as follow-up: derive the expected sets from the shipped .sql or assert against pg_constraint. Recorded because a guard believed bidirectional that is not is worse than no guard - it converts a real check into a false sense of one.",
    1100, 1124, tags=["testing", "ci", "failure"])

add("fact", "quoted",
    "append_supervisor_message is also non-atomic and swallows ref-insert failures",
    "Beyond the known defaults dependency, the legacy supervisor-message writer performs two inserts outside any transaction and discards the ref insert's result (let _ =), so an event can land without its ref - extra facts found while verifying the known finding.",
    "Established while confirming the zero-subject finding in code (repositories.rs:405-433). The two inserts not being wrapped means append_supervisor_message is not merely unmigrated but structurally unable to satisfy the atomicity the new write API guarantees. Recorded because verifying a known finding often surfaces adjacent facts, and those are cheap to capture while the code is open.",
    1130, 1131, tags=["database", "write-api", "fact"])

add("decision", "quoted",
    "Truncated instructions get a stated interpretation, never a silent guess",
    "When a dispatch arrives truncated (ur run., k out packet/nimbus...), the verifier states the interpretation it is acting on, proceeds only with the low-risk reading, and explicitly invites correction.",
    "Applied twice in this session. The positive instruction was recovered from the dispatch document itself (read the full doc rather than inferring from the fragment), and the exclusion it implied (do not touch packet/nimbus) was honored. The same fragments were the first visible symptom of the send-keys truncation defect that later forced file-based dispatch.",
    33, 135, tags=["coordination", "verification", "discipline"])

print(json.dumps(store.stats(), indent=1))
