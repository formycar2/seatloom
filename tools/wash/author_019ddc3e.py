"""Author S3 records for codex/019ddc3e (April Flux bounded-verification session)."""
import json, os, sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from vela_wash.s3_extract import RecordStore

AH = os.path.expanduser("~/Documents/GitHub/AgentHistory")
S = "019ddc3e-06d0-73c3-a80e-0d49f38ffe80"
F = "raw/codex/rollout-2026-04-30T10-35-45-019ddc3e-06d0-73c3-a80e-0d49f38ffe80.jsonl"
store = RecordStore(os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                 ".local/wash/s3/records.jsonl"), AH)

store.add(kind="preference", confidence="quoted",
    title="Bounded verification packets: own only your files, pin the exact commit, no scope widening",
    l0_summary="A verification packet bounds the verifier to named files, forbids reverting others' work, forbids widening scope or parallel subtasks, and pins verification to an exact commit even when the branch has moved on.",
    l1_overview="The April-era Flux packet states the ownership boundary explicitly (only the delivery doc and the evidence directory), the shared-tree etiquette (you are not alone in the codebase; do not revert anyone else's work), the anti-parallelism rule, and commit pinning (verify the exact target SHA even though the branch now contains later commits). These four constraints are the ancestor of the later commit-pinned gate discipline: detached HEAD at the exact SHA, scope asserted by diff stat, and other seats' scratch state left untouched.",
    session_id=S, harness="codex", topic_id="t000",
    l2_file_path=F, l2_line_start=6, l2_line_end=6, evidence_seqs=[],
    tags=["verification", "governance", "coordination"])

print(json.dumps(store.stats(), indent=1))
