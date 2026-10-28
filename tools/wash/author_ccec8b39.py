"""Author S3 records for claude-code/ccec8b39 (Flux v1-v3 Onyx gate; unique material only —
the overlapping gate arc is already recorded from session 5c09cdda)."""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from vela_wash.s3_extract import RecordStore  # noqa: E402

AH = os.path.expanduser("~/Documents/GitHub/AgentHistory")
S = "ccec8b39-b1b9-4445-bb7a-336ec823f8f2"
F = "raw/claude-code/ccec8b39-b1b9-4445-bb7a-336ec823f8f2.jsonl"
store = RecordStore(os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                 ".local/wash/s3/records.jsonl"), AH)


def add(kind, conf, title, l0, l1, a, b, topic="t000", tags=None):
    r = store.add(kind=kind, confidence=conf, title=title, l0_summary=l0,
                  l1_overview=l1, session_id=S, harness="claude-code",
                  topic_id=topic, l2_file_path=F, l2_line_start=a, l2_line_end=b,
                  evidence_seqs=[], tags=tags or [])
    return r["record_id"][:8]


add("decision", "quoted",
    "A bounded fix mirrors the existing reference implementation, not the packet's suggestion",
    "When a packet authorizes a bounded fix and suggests a variable name, port the reference implementation's actual convention instead: bootstrap.sh already solves container detection with $CONTAINER and docker-first-then-podman precedence, so the verifier adopts that naming and order rather than the packet's proposed $CTR.",
    "Consistency with the in-repo reference beat the packet's wording. The fix was also proven sufficient-or-not honestly: detection resolves (auto -> podman, explicit docker fails closed with exit 1), but podman has no compose provider, so the authorized detection fix is necessary and not sufficient - the compose-to-run/rm port was recommended as a separate scoped packet rather than silently expanded into. Detection was proven with a truncated copy of the script that stops before the destructive reset, never by running the real thing.",
    353, 405, tags=["verification", "infrastructure", "discipline"])

add("decision", "quoted",
    "Commit the delivery doc locally even when not asked: an untracked doc hands the next seat a dirty tree",
    "A verification delivery doc is committed locally (not pushed) even when the dispatch does not ask, because leaving it untracked would leave the shared tree dirty and trigger for the next seat exactly the dirty-workspace HOLD being reported in that same doc.",
    "The same consistency check applies to the watcher-regenerated file: before discarding anything, the generated content was proven byte-identical to what track already has committed (hash cc4097f8...), so the discard is lossless - and no stash was used because the packet forbids it. The durable insight was additionally written to cross-session memory even though it appears in the delivery doc, because its value is as a workflow constraint for future gates, not as a one-off report.",
    252, 292, tags=["governance", "verification", "discipline"])

add("fact", "quoted",
    "Environmental gaps a packet cannot anticipate are worked around transparently, not treated as defects",
    "Two environmental gaps surfaced mid-gate that were nobody's defect: scripts/apply-db.sh does not exist at the pinned commit (it landed in a later build-standards packet), and no psql exists on the host at all. Both were worked around with disclosed, removable artifacts (the script copied in from track as an untracked tool; a PATH shim forwarding psql into the container) and removed afterward.",
    "The distinction that matters: an environmental gap is engineered around transparently and recorded, while a from-scratch build failure caused by the change under test is a FAIL with a transcript. The same discipline decided the v1 HOLD: the assertions that could not be honestly verified (double-apply without writing to the shared DB; the unrunnable baseline verifier) were listed as unverified rather than assumed, and the verdict was HOLD with no defect alleged against the delivered work.",
    337, 338, tags=["verification", "database", "discipline"])

print(json.dumps(store.stats(), indent=1))
