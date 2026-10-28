"""Author S3 records for claude-code/ca5f59be (Onyx schema/migration session)."""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from vela_wash.s3_extract import RecordStore  # noqa: E402

AH = os.path.expanduser("~/Documents/GitHub/AgentHistory")
S = "ca5f59be-32fc-493e-ac44-3731062d4c75"
F = "raw/claude-code/ca5f59be-32fc-493e-ac44-3731062d4c75.jsonl"
store = RecordStore(os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                 ".local/wash/s3/records.jsonl"), AH)


def add(kind, conf, title, l0, l1, a, b, topic="t000", tags=None):
    r = store.add(kind=kind, confidence=conf, title=title, l0_summary=l0,
                  l1_overview=l1, session_id=S, harness="claude-code",
                  topic_id=topic, l2_file_path=F, l2_line_start=a, l2_line_end=b,
                  evidence_seqs=[], tags=tags or [])
    return r["record_id"][:8]


add("failure", "quoted",
    "Packet facts described an uncommitted working tree, not the base commit",
    "The runtime-triple packet's fact table claimed six seats all on ClaudeCode and a data_engineer role binding measured at base commit 9599136 - but the committed base has five seats with mixed runtimes and no data_engineer; the six-seat state was an uncommitted working-tree change (the 2026-09-28 convergence) that had never been committed.",
    "Onyx caught it by reading the seed at the base commit with git show, reconstructed the pre-existing uncommitted change exactly (convergence only, no triple columns), and committed it separately as its own commit so the packet's diff stayed auditable - pre-existing work is never silently absorbed into a delivery. This is the third independent instance of the dirty-tree-as-committed-fact error in the same window (Lyra and Nimbus hit their own versions).",
    358, 386, tags=["verification", "governance", "failure"])

add("failure", "quoted",
    "Schema 008 and seed 001 are mutually inapplicable (D-3, P0)",
    "Migration 008 makes sessions.project_id NOT NULL and drops its default, but seed 001 never supplies project_id - so the seed cannot be applied after 008, and every second apply-db.sh run fails on the same row. This is why the old verifier stops at schema 005.",
    "Proven pre-existing by control experiment: track's own script is exit 0 on run 1 and exit 3 on run 2 with the identical project_id error, independent of any later change. ON CONFLICT cannot help because the NOT NULL check fires before conflict arbitration. The minimal fix requires editing the seed SQL, which packets forbade - so it was recorded as a tracked defect rather than fixed in-flight. The ordering rule that emerged (migrations that add columns the seed writes run pre-seed; migrations that constrain columns the seed does not provide run post-seed) exists precisely because of this class of conflict.",
    340, 802, tags=["database", "migration", "failure"])

add("failure", "quoted",
    "verify-postgres-baseline.sh is stale and unrunnable",
    "The verification script applies only schema 001-005 (it never applies 008, so its database lacks project_id) and hardcodes docker - on this machine there is no docker and no compose provider at all, so the script cannot run as written.",
    "Captured as a base-commit blocker independent of any change (docker: command not found). bootstrap.sh is the podman-aware path and applies 008. The D-5 drift finding recommended porting container detection; the detection fix alone is necessary but not sufficient because podman has no compose provider either - the compose-to-run/rm port is a separate scoped packet. Recorded because a verification tool that cannot run is worse than a missing one: it silently stops being the authority.",
    179, 247, tags=["database", "tooling", "failure"])

add("fact", "quoted",
    "SeatRegistry has zero production callers; the file store is an unimplemented export path",
    "Every save_* call site in the seat registry lives inside its own test module (test module starts at line 176; production code is lines 1-174 with definitions only) - so the empty .seatloom/seats/ and delegations/ directories are not a failing writer but an export path that was never wired.",
    "Established while writing the drift report: PG holds 6 seats, 6 role bindings and 1 delegation while the file store holds nothing. The distinction matters because the two diagnoses have different owners - a silent failure needs a bug fix, an unimplemented path needs a decision (wire it up or declare PG-only, since AD-011 already makes PG canonical and .seatloom a cache).",
    537, 544, tags=["database", "architecture", "fact"])

add("decision", "quoted",
    "A closed CHECK vocabulary must be built from what the running system can insert, not only from seed rows",
    "Before shipping a closed event_type vocabulary, enumerate every value the live code path can insert: the Supervisor IM path emits SupervisorMessage and SeatResponse (supervisor_cmds.rs:45), neither of which appears in any seed row, and the frontend passes event_type through unvalidated. A vocabulary measured from seed data alone would have broken the top-level product path at runtime.",
    "The seed carries 10 distinct event types; live Rust adds 2 more that have never appeared in a row. Combined with the ref_type finding (the contract domain omitted delegation, which exists in committed seed data - a CHECK built from the contract as written would fail on existing history), the rule is: measure seed + live code + the contract, reconcile all three, and flag every discrepancy rather than silently working around it.",
    912, 951, tags=["schema", "database", "governance"])

add("decision", "quoted",
    "Encode migration ordering as explicit justified lists with a completeness guard",
    "apply-db.sh now orders migrations by rule with explicit ordered lists (001-005 plus 009 pre-seed, seed 001-004, 008 post-seed), a one-line justification printed per entry, and a completeness guard that fails loudly (exit 2) on any on-disk migration not classified - the silent skip is exactly what let the ordering bug hide.",
    "The fix was proven by reproducing the FAIL first (exit 3, same error), then the after (exit 0 from empty), then proving the guard actually fires with a deliberately unclassified file, then the control experiment establishing the second-run failure is pre-existing on track. The header also states what to do with a migration that needs both orderings (split it; none does today).",
    743, 777, tags=["database", "migration", "ci"])

add("decision", "quoted",
    "When another worktree holds your branch: detach, work, tag, and update the ref last",
    "If another seat's worktree has your packet branch checked out, do all work on a detached HEAD at your delivery SHA, protect the resulting commits with a tag you own (so nothing is garbage-collected and no ref is contended), and treat the branch-ref update as the single remaining contention point to coordinate.",
    "Used twice: when Flux's worktree held the packet branch, and again when the ref update was blocked by worktree protection. Also in this session: commits are strictly path-scoped in a shared tree so another seat's uncommitted work is never included, and a status that raced a concurrent revert was reported honestly (dirty, then clean, with the race named) rather than assumed either way.",
    672, 841, tags=["git", "coordination", "worktree"])

add("decision", "quoted",
    "Verify every packet fact with git show against the authoritative base before reading anything else",
    "Packet facts are verified with git show <authoritative-base>:<path> before any implementation reading, and vocabulary measurements separate their grep patterns - conflating event_type with ref_type produced a wrong count that had to be re-measured.",
    "Applied to the P0 schema packet: facts 1-7 confirmed exactly (including that sessions.project_id already exists from 008, so the G5 obligation is verify-only), the base was established as the GitLab authoritative tip (verified an ancestor of local track), and the one claim that did not check out (nonexistent CI schema/contract stages) was flagged back rather than propagated. Comments inside a bash array literal were verified by printing the parsed array at runtime, not just bash -n.",
    884, 1023, tags=["verification", "governance", "discipline"])

print(json.dumps(store.stats(), indent=1))
