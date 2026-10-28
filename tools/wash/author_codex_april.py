"""Author S3 records for the two April-era codex sessions (019dda18, 019dce05).

These are the Lyra PO foundation sessions (PRD review, frontend refactor, seat
coordination). Extraction is topic-sampled (documented in the delivery doc):
the highest-substance topic windows were read in full.
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from vela_wash.s3_extract import RecordStore  # noqa: E402

AH = os.path.expanduser("~/Documents/GitHub/AgentHistory")
store = RecordStore(os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                 ".local/wash/s3/records.jsonl"), AH)


def add(sid, f, kind, conf, title, l0, l1, a, b, topic, tags=None):
    r = store.add(kind=kind, confidence=conf, title=title, l0_summary=l0,
                  l1_overview=l1, session_id=sid, harness="codex",
                  topic_id=topic, l2_file_path=f, l2_line_start=a, l2_line_end=b,
                  evidence_seqs=[], tags=tags or [])
    return r["record_id"][:8]


S1 = "019dda18-07f6-79e0-8008-e4eb16a5448d"
F1 = "raw/codex/rollout-2026-04-30T00-35-01-019dda18-07f6-79e0-8008-e4eb16a5448d.jsonl"
S2 = "019dce05-fb23-7081-aaa6-40d60b4b33d2"
F2 = "raw/codex/rollout-2026-04-27T16-19-51-019dce05-fb23-7081-aaa6-40d60b4b33d2.jsonl"

add(S1, F1, "decision", "quoted",
    "Working rule: approved docs override draft docs where they conflict",
    "Where documents conflict, approved artifacts win over drafts - a rule inferred from status fields (mvp-scenarios and architecture-decisions are approved; the PRD, validation, UX spec and architecture design are still drafts), which means product truth is currently fragmented by design.",
    "Stated explicitly as an inference in the product review packet, with the consequence spelled out: the approved docs are executable for surface layout but not sufficient for implementation because unresolved contracts remain. The review produced a prioritized decision list (P0 entry path, seatloom run, pipeline triggers, technical gaps, LaunchPack fallback), a two-week milestone plan, and a stage-gate proposal with entry/exit criteria and go/hold rules - the pattern that later became the packet-plus-acceptance rhythm.",
    352, 353, "t001", tags=["governance", "product", "decision"])

add(S1, F1, "decision", "quoted",
    "Frontend refactor direction: dashboard-first, 48px icon rail, three-layer disclosure, light canvas",
    "The frontend refactor direction set the default first screen to a supervision dashboard (data-driven dashboard plus workflow panorama plus action queue), replaced the full sidebar with a 48px icon rail, adopted three-layer progressive information disclosure (L1 scan / L2 decide / L3 deep), and moved to a light canvas with scalpel-style color accents.",
    "Dispatched from the ui-design-direction review with a baseline commit first (chore: baseline commit before frontend refactoring) so the refactor had a clean diff base. Executed as phases P1-P7 over the following day; the light-theme direction and the nav-rail structure are what the later V1 AppShell still carries.",
    15506, 15507, "t155", tags=["ui", "architecture", "decision"])

add(S1, F1, "preference", "quoted",
    "Seat-to-seat messages in English; multi-line via load-buffer/paste-buffer",
    "Messages between seats are written in English, and multi-line instructions are delivered by writing a temp file and using tmux load-buffer plus paste-buffer rather than fighting send-keys escaping.",
    "The dispatch convention from the April era: concise messages, one active packet per seat, artifact paths referenced rather than inlined. The load-buffer trick predates - and partially motivates - the later file-based seat-dispatch.sh that fixed the truncation problem for good.",
    21985, 21992, "t221", tags=["coordination", "tmux", "technique"])

add(S2, F2, "failure", "quoted",
    "Seat continuity failed repeatedly: Mira went offline mid-work and her work was absorbed",
    "The Mira seat went offline repeatedly (context crashes, CLI failures) and Lyra absorbed her outstanding work each time - the concrete, recurring failure that the whole continuity platform exists to eliminate.",
    "The April sessions show the pattern over and over: Mira offline, her left work reassigned, a re-entry packet prepared, results collected later. This is the empirical basis for the later architecture claim that seat context must not live inside the harness - a seat that can be continued from docs plus state survives its harness session dying.",
    11865, 11966, "t128", tags=["coordination", "continuity", "failure"])

add(S2, F2, "fact", "quoted",
    "Nimbus delivered the storage plus ledger foundation (canonical events, content-addressed documents)",
    "Nimbus's storage and ledger foundation delivered the canonical event ledger and the documents table with content addressing (body digest, revision, run_id lineage) - the write-side substrate everything later builds on.",
    "Reported to Lyra as a completed packet in the April window. The relevant nuance discovered later: the ledger existed but had no live writers (all events were seed rows), which is precisely the gap the P0 write API (append(event) to projections) closed in September.",
    12538, 12653, "t132", tags=["architecture", "database", "fact"])

print(json.dumps(store.stats(), indent=1))
