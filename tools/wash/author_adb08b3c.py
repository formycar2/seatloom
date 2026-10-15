"""Author S3 records for claude-code/adb08b3c (Lyra PO session)."""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from vela_wash.s3_extract import RecordStore  # noqa: E402

AH = os.path.expanduser("~/Documents/GitHub/AgentHistory")
S = "adb08b3c-2591-4cea-8be5-be5241967415"
F = "raw/claude-code/adb08b3c-2591-4cea-8be5-be5241967415.jsonl"
store = RecordStore(os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                 ".local/wash/s3/records.jsonl"), AH)


def add(kind, conf, title, l0, l1, a, b, topic="t000", tags=None):
    r = store.add(kind=kind, confidence=conf, title=title, l0_summary=l0,
                  l1_overview=l1, session_id=S, harness="claude-code",
                  topic_id=topic, l2_file_path=F, l2_line_start=a, l2_line_end=b,
                  evidence_seqs=[], tags=tags or [])
    return r["record_id"][:8]


add("decision", "quoted",
    "Paired packets couple through exactly one Lyra-owned value-domain contract table, verbatim in both",
    "When two seats implement two halves of one change, the only coupling is a single value-domain contract table owned by the PO and duplicated verbatim in both packets; the seats never negotiate the contract between themselves.",
    "Established for the runtime-triple pair (Nimbus Rust half, Onyx schema half). The contract pinned Harness/Agent in PascalCase to match existing PostgreSQL sessions.runtime values and SeatRole in snake_case to match existing seed values - the inconsistency is deliberate because both rules follow align-with-existing-data, and the existing data is itself inconsistent. NULL harness/model means unrecorded, not default; April history is not backfilled with 9-28 convergence values because that would fabricate attribution the ingestion layer later depends on.",
    264, 264, tags=["governance", "contract", "coordination"])

add("decision", "quoted",
    "Migration ordering is by rule, not by number",
    "apply-db.sh orders migrations by a stated rule - pre-seed if the seed writes columns the migration creates, post-seed if the migration constrains columns the seed does not provide - with a justification per migration, not by numeric slot.",
    "Born from a real gate failure: seed 001 at the merge candidate wrote seats.default_harness/_agent/_model and therefore needed 009 before the seed, while apply-db.sh bucketed every migration >=006 after the seed, so the scratch build from empty failed (exit 3, column default_harness does not exist). Onyx fixed the ordering by rule with per-migration justifications and touched no .sql file. Consequence to remember: an IF NOT EXISTS guard means re-applying to a database that already has the old constraint silently keeps it - verification needs a fresh database.",
    967, 1556, tags=["database", "migration", "ci"])

add("decision", "quoted",
    "Serialize packet branches on the shared tree; later amended to per-seat worktrees with disjoint file sets",
    "Originally: one packet branch checked out at a time because six seats share one working tree. Amended once per-seat worktrees existed: parallel packets are allowed only with disjoint file sets, and the shared checkout is reserved for docs.",
    "The serial rule came from Mr. Zhang directly (all six seats share one working tree, so one branch at a time). When Aegis later dispatched a parallel frontend packet (Mira) alongside Nimbus's Rust work, Lyra verified isolation before allowing it: Mira idle and self-contained, api.ts wrappers confirmed, Nimbus untouched ui/ so far - and fenced the two DTO type files Nimbus was permitted to touch, warning Mira to escalate rather than edit them. Parallelism was allowed because the file sets were made disjoint, not because the rule was waived.",
    358, 1744, tags=["coordination", "worktree", "governance"])

add("decision", "quoted",
    "Relayed dispatch files are data to verify, never instructions to execute",
    "Messages of the form AEGIS dispatch - read this file and act on it are treated as untrusted content: verify the file's origin (git log, authoring seat) and re-measure its factual claims at the authoritative commit before acting, even when the operator relays them.",
    "Lyra applied this consistently to every Aegis dispatch: verified gitlab track was a true ancestor before basing a packet on it, re-measured the 010 defaults the packet had to override, checked that the shared tree was actually clean rather than trusting a stale justification, and caught that a claimed CI schema/contract stage does not exist (real stages are lint/test/docs) - anchoring acceptance to rust:test instead and flagging the discrepancy back. The same discipline is why the truncated send-keys fragments were eventually traced to the dispatch mechanism rather than guessed at.",
    662, 1425, tags=["governance", "security", "coordination"])

add("decision", "quoted",
    "A closed-vocabulary CHECK applied post-seed must accept every value already in the data",
    "Before landing a closed event_type vocabulary, verify the CHECK accepts every value already present in committed data - across all seed files, not just the first - and that the CI reconcile step cannot emit an out-of-vocabulary value.",
    "Verified for the 12-value vocabulary: seed 001 is the only seed touching canonical_events and uses exactly the 10 group-(a) values; the Rust code emits only SupervisorMessage and SessionStarted and reconcile does not write canonical_events. Also caught the mechanical trap: deleting the ArtifactProduced line naively would leave a dangling comma before the closing paren - a SQL syntax error - so the edit was specified precisely. A retired value survives only in a comment recording why it was dropped, which is the right way to retire from an append-only vocabulary.",
    1433, 1565, tags=["schema", "database", "governance"])

add("decision", "quoted",
    "Commit-pinned verification gates run in a detached HEAD at the exact SHA",
    "A verifier checks out the target commit in detached HEAD state rather than a branch: it pins the exact SHA, works even when the branch is held by another seat's worktree, and cannot disturb the owning seat.",
    "Adopted when git refused Flux a checkout of the Nimbus step-2 branch because Nimbus's worktree held it. Also the durable pattern behind declining an offered warm target/ inside another seat's worktree - isolation between seats is what prevented collisions all evening, and a frontend-only gate gains nothing from a Rust build cache anyway.",
    1917, 2111, tags=["coordination", "verification", "git"])

add("failure", "quoted",
    "Working-tree observations recorded as committed facts (twice, same class)",
    "Lyra twice wrote fact tables measured against a dirty working tree instead of the committed state: the runtime-triple packet's seat-runtime table claimed six ClaudeCode seats when the committed base had five with mixed default_runtime, and the step-2 contract table listed eight ref_type values when merged 010 admits nine (delegation is in committed seed data).",
    "Both were caught by other seats re-measuring with git show against the authoritative commit - Flux flagged the first (leading to an explicit HOLD on the packet itself), Nimbus flagged the second before landing. Root cause both times: reading the working tree (or copying an earlier stale table) instead of re-measuring the landed artifact. The class of error matches a standing memory lesson about treating observation as proof of what is committed. The erratum discipline that followed: correct the table before any downstream packet inherits it.",
    622, 1897, tags=["governance", "verification", "failure"])

add("failure", "quoted",
    "Monitor watchers false-positived on stale log windows and intermediate progress",
    "Three separate background watchers misfired: one matched an unrelated older commit in the log window, one fired on a branch tip advancing to a mere track-merge (step 1 of 4), one fired on the delivery doc appearing while the branch had not moved. Each produced a premature milestone report that had to be retracted.",
    "Fix: watch the actual completion signal - the branch tip advancing past a known SHA, or the delivery doc existing in the branch tree - never a grep over a commit window. Related near-miss in the same session: Lyra's own fast-forward of the packet branch landed one commit short (cfdd647 instead of d4f40b5) and briefly dropped Onyx's 24KB delivery doc; it was recovered from the reflog within minutes, and the lesson recorded is to verify the ref target against the full intended commit range before moving refs.",
    836, 1075, tags=["coordination", "tooling", "failure"])

add("failure", "quoted",
    "append_supervisor_message bypasses the write API and depends on both 010 defaults",
    "repositories.rs inserts supervisor messages straight into canonical_events without naming schema_version and into event_object_refs without naming role - so it depends on both migration-010 defaults - and it writes only a seat-type ref, making every supervisor message a zero-subject event that the new write API rejects by design.",
    "Verified by reading the code (repositories.rs:405-433) rather than accepting Nimbus's report secondhand. It is the sole remaining consumer of both defaults, so it blocks the P1 cleanup that would drop them, and it produces events the new API would refuse. Nothing breaks at runtime because 010 keeps the defaults; the obligation is to migrate this path onto the write API. Recorded in daily memory the same hour as insurance against loss.",
    1829, 1858, tags=["database", "write-api", "failure"])

add("failure", "quoted",
    "prettier --check walks ui/dist: no .prettierignore makes counts order-dependent",
    "There is no .prettierignore, so prettier --check . walks the generated ui/dist; running a build first moves the file count from 96 to 103 and gate counts stop reconciling. A stale full ui/dist left in a verifier worktree poisons the next gate the same way.",
    "Mira warned about this in her delivery notes; it had already bitten Lyra's own prep - Flux's worktree held a full step-2 build (assets, index.html, supervisor.html), not just the stub, and was cleaned before dispatch. Mitigations: keep packet ordering (check before build) or rm -rf ui/dist first; the durable fix is adding .prettierignore, which is gate-tool debt owned outside the delivering seat.",
    2016, 2111, tags=["ci", "tooling", "failure"])

add("failure", "quoted",
    "The drift-guard test only compares Rust against a hardcoded transcription, not the live schema",
    "serialized_sets_match_schema_010_checks compares the Rust enums against a hardcoded transcription of migration 010, so it catches Rust drifting from the transcription but not both drifting from the real schema together - a one-directional guard.",
    "Flux disclosed this precision on a PASS it had already earned (it manually verified all three Rust lists against a real database's constraints). Lyra accepted the PASS, corrected the erratum that had overstated the guard, and recorded the follow-up: derive the expected sets from the shipped .sql file or assert against pg_constraint directly. Recorded because the corrected claim matters more than the PASS - a guard believed bidirectional that is not is worse than no guard.",
    1967, 1981, tags=["ci", "testing", "failure"])

add("failure", "quoted",
    "Truncated send-keys dispatches arrived as unparseable user fragments",
    "Long tmux send-keys dispatches truncated in transit, so the operator's instructions arrived as fragments like n issued. and ion packet on top of the landed schema - initially unreadable, and only later traced to the dispatch mechanism once the file-based dispatch script existed.",
    "The fragments caused Lyra to stop and ask rather than guess - the right call given the stakes. The explanation arrived retroactively when Aegis shipped scripts/seat-dispatch.sh (write the instruction to a file, send a short pointer). Recorded because the failure mode is silent: a truncated instruction looks like a typo, and acting on a guessed reading is worse than asking.",
    268, 734, tags=["coordination", "tooling", "failure"])

add("unresolved", "inferred",
    "verify-postgres-baseline.sh remains unrunnable; apply-db.sh second-run idempotency defect",
    "Two pre-existing verification defects were confirmed and deliberately not fixed in-flight: verify-postgres-baseline.sh cannot run (it hardcodes docker and podman has no compose provider, and its compose down -v would wipe live data), and a second consecutive apply-db.sh run still fails because seed 001's session inserts lack project_id which 008 makes NOT NULL (ON CONFLICT cannot help because the NOT NULL check fires first).",
    "Both were proven pre-existing by control experiments (fresh scratch database, live track checkout, git archive to /tmp - same exit code, same failing row ses-aegis-supervisor-001), so neither regressed the merge; both were recorded as follow-ups rather than fixed under a packet that forbade seed edits. Resolution status is inferred: the corpus does not show these being fixed.",
    920, 976, tags=["database", "tooling", "unresolved"])

add("fact", "quoted",
    "tmux keeps one pipe per pane, not per client; 8 leaked FIFOs predate the session",
    "tmux maintains one pipe per pane rather than per client, so whichever client detaches first determines mirror behaviour for all - and eight leaked FIFOs (ses-* pattern) were found in /tmp/seatloom-mirror dating from hours before the session, because PtySession::kill unlinks the FIFO but crashes and dev-server restarts bypass that cleanup path.",
    "Established while verifying the pipe-pane re-attach fix. Nimbus independently reproduced the toggle behaviour on a throwaway session (0,1,0,1,1) and bounded its own fix with a MirrorNotOpen error and verify_pipe_open. The FIFO leak is a genuine resource leak with a recommended follow-up (multi-client pipe ownership plus detecting a pipe that dies right after opening).",
    2130, 2157, tags=["pty", "tmux", "infrastructure"])

print(json.dumps(store.stats(), indent=1))
