"""Author S3 records for claude-code/01565715 (Nimbus runtime-triple + write API + pipe-pane fix)."""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from vela_wash.s3_extract import RecordStore  # noqa: E402

AH = os.path.expanduser("~/Documents/GitHub/AgentHistory")
S = "01565715-3e7f-48c8-9ff0-fdd8edbfdf9f"
F = "raw/claude-code/01565715-3e7f-48c8-9ff0-fdd8edbfdf9f.jsonl"
store = RecordStore(os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                 ".local/wash/s3/records.jsonl"), AH)


def add(kind, conf, title, l0, l1, a, b, topic="t000", tags=None):
    r = store.add(kind=kind, confidence=conf, title=title, l0_summary=l0,
                  l1_overview=l1, session_id=S, harness="claude-code",
                  topic_id=topic, l2_file_path=F, l2_line_start=a, l2_line_end=b,
                  evidence_seqs=[], tags=tags or [])
    return r["record_id"][:8]


add("failure", "quoted",
    "Packet fact tables measured against the dirty working tree, not the base commit",
    "The runtime-triple packet's fact table was verified against the uncommitted working tree instead of base commit 9599136: two rows were wrong - a line citation off by one, and a claim of six seed roles including data_engineer when the committed base has five roles and no data_engineer (it existed only in the uncommitted tree).",
    "Nimbus re-measured every row with git show against the true base and corrected the citations and the rationale that depended on them. Onyx's concurrent commit of the seed made the discrepancy visible - the commit message surfaced a fact that affected the packet. This is the same error class Lyra made in the same window (both seats recorded dirty-tree observations as committed facts), which is why the correction discipline is: re-measure the landed artifact with git show, never read the tree.",
    500, 769, tags=["verification", "governance", "failure"])

add("failure", "quoted",
    "Pre-existing clippy red blocked a green-gate criterion while scope rules forbade the fix",
    "The packet required clippy fully green, but four lints were already red at the base commit - three doc-lints in repositories.rs and one in pty/mod.rs, a file the packet explicitly forbade touching - so the criterion was unreachable without violating scope.",
    "Resolved two ways: definitive evidence that all four were pre-existing was produced on a throwaway worktree pinned at the base commit (rather than a stash comparison muddied by other seats' uncommitted work), and the conflict then dissolved when track itself merged the fixes. Recorded because the general lesson is to prove pre-existing lint state at the true base with citable evidence, and to surface an unreachable criterion rather than silently expanding scope to reach it.",
    407, 557, tags=["ci", "governance", "failure"])

add("decision", "quoted",
    "Isolate your commit in a shared tree via a temporary index, never by switching branches",
    "When another seat is actively working in the same shared checkout on a different branch, land your own commit by writing a temporary index (or a worktree) for exactly your files - never by checking out a branch, which would yank HEAD out from under them.",
    "Nimbus did this while Onyx worked in the same tree: a temp index committed exactly his 10 files to the delivery branch, leaving Onyx's HEAD, index and worktree untouched; Onyx's next commit swept only the seed file. Afterwards he removed his files from the shared tree so they could not be swept into anyone else's commit, verifying afterwards with git show --stat (which reads commit objects, not the tree).",
    419, 581, tags=["coordination", "git", "worktree"])

add("failure", "quoted",
    "A proof whose error message says only 'db error' does not prove which failure occurred",
    "Two behavioural proofs initially failed with only postgres error: db error, and in one of them the claimed root cause (a lossy payload) was simply wrong - the actual blocker was a foreign key on handoffs. A vague error is not evidence, and a wrong narrative on top of it is worse.",
    "Fixed by rendering full Postgres error detail (SQLSTATE, constraint, table) into the proof output and rewriting the proof honestly: 6 of 6 seeded events unreplayable with the exact serde error, and the FK named explicitly in the second case. Recorded because the failure mode is subtle - the proof passed for the wrong stated reason, and only the reviewer's demand for the specific cause exposed it.",
    1120, 1154, tags=["testing", "verification", "failure"])

add("failure", "quoted",
    "tmux clears pane_pipe only when it first writes to a dead pipe - an earlier probe was misleading",
    "A standalone probe showed pane_pipe dropping to 0 after the piped process died, but the same condition inside the harness read 1: tmux clears the flag only when it first attempts a write to the dead pipe, and a quiet pane never attempts one.",
    "The contradiction was chased down rather than averaged over: the fresh session's startup output had triggered the write in the standalone probe. Every claim built on the wrong reading (doc comment, unit-test comment, proof 4a) was corrected, and the honest bound was stated - the check catches pipes that never opened, not pipes that die on first write. The probe that found the real trigger: a pipe-pane whose redirect target is unwritable exits 0 with pane_pipe=0, the genuine silent-dead-mirror condition.",
    1448, 1491, tags=["pty", "tmux", "testing"])

add("decision", "quoted",
    "Two event vocabularies coexist deliberately: file-ledger EventType stays, a PG-boundary enum is new",
    "The ~27-variant file-ledger EventType (JSONL index) is left untouched; the Postgres canonical_events boundary gets its own closed 12-value enum with hand-written FromStr/Display, because the standards forbid serde derive across the Rust-PG TEXT boundary and the two vocabularies serve different stores.",
    "The file ledger's broad vocabulary and the PG CHECK's closed vocabulary are not the same domain and must not be conflated; reusing one for the other would either lose variants or invent values the database rejects. The parse error type was reused from the runtime-triple work (ParseValueError), which is why that refactor had to land first.",
    990, 1046, tags=["schema", "rust", "architecture"])

add("fact", "quoted",
    "34 of 36 ledger events had zero subject refs; only the two written by the new API comply",
    "Measured in the scratch database: 34 of 36 canonical_events rows carry zero subject refs. The only two that satisfy the exactly-one-subject invariant are the ones the new write API itself wrote - the entire seeded history predates the invariant.",
    "This quantified the append_supervisor_message finding (it writes only a seat-type ref, so every supervisor message is a zero-subject event the new API rejects, and it depends on both 010 transitional defaults). The finding was anchored at both commits because the author's own helper shifted the line numbers by +7 - both anchor sets are correct for their own commit, and the delivery doc said so explicitly so the verifier could locate them.",
    1199, 1267, tags=["schema", "write-api", "database"])

add("decision", "quoted",
    "Contract deviations are notified to the PO before landing, even when the implementation is already committed",
    "When the merged schema admits a value the packet's contract table omits (ref_type has nine values including delegation, the packet said eight), the implementing seat adds the value, flags the deviation to the PO before merging, and records it in the delivery doc - committed-but-unmerged still counts as before landing.",
    "Nimbus caught that his packet's table was stale (copied from an earlier draft instead of re-measured against the landed migration), implemented the correct nine-value domain, and notified Lyra before landing. Onyx had added delegation during 010 and flagged it; the error was in the PO's copied table. Recorded because the discipline is what makes the verbatim-contract pattern safe: deviations are surfaced, never silently absorbed.",
    963, 1223, tags=["governance", "contract", "coordination"])

add("decision", "quoted",
    "Gate against the last commit touching code, never the delivery-doc tip; never stamp the tip SHA into the doc",
    "Verification gates pin the last commit that touched code (not the delivery-doc commit), and a delivery doc references the code SHA explicitly rather than the branch tip - because stamping the tip into the doc changes the tip, a self-reference loop.",
    "Hit directly: amending the delivery doc changed the SHA it had just stamped (7738905 to 1d1affa), making the doc's own reference stale. Fixed by pinning the code SHA and noting the delivery commit separately. The same commit-pinning discipline is why Flux later verifies in a detached HEAD at an exact SHA.",
    853, 887, tags=["governance", "verification", "git"])

add("decision", "quoted",
    "Record the baseline of every tmux pane before touching tmux state, and leave no FIFO behind",
    "Before changing anything that touches tmux, capture the current pane_pipe state of every session so you can prove you left it as found; proof-harness FIFOs get distinct names and are verified cleaned up, and pre-existing leaked FIFOs are left untouched and reported.",
    "Applied in the pipe-pane fix: baseline captured (10 sessions, all pane_pipe=0), state verified identical afterwards, proof FIFOs named proof-attach-* and confirmed gone, and eight pre-existing ses-* FIFOs (from the running app or another seat, timestamped hours earlier) reported rather than cleaned. The distinction between your artifacts and pre-existing leaks is what makes the claim credible.",
    1343, 1514, tags=["pty", "tmux", "discipline"])

add("fact", "quoted",
    "Packet files vanish on branch checkout; recover them with git show against the owning branch",
    "A packet committed to track/infra-foundation disappears from the working tree when a seat checks out its own packet branch; the recovery path is git show <branch>:<path>, which reads the commit object rather than the tree.",
    "Hit repeatedly by both Nimbus and Flux. The durable lesson Lyra added: proactively tell a seat the recovery path before it stalls on a vanished packet, since the failure looks like a lost file rather than a branch-scoped view.",
    914, 936, tags=["coordination", "git", "tooling"])

print(json.dumps(store.stats(), indent=1))
