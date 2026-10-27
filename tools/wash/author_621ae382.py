"""Author S3 records for claude-code/621ae382 (Mira UM0 live-mirror session)."""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from vela_wash.s3_extract import RecordStore  # noqa: E402

AH = os.path.expanduser("~/Documents/GitHub/AgentHistory")
S = "621ae382-d6c1-4f0c-b3fa-7c1155a5ecb8"
F = "raw/claude-code/621ae382-d6c1-4f0c-b3fa-7c1155a5ecb8.jsonl"
store = RecordStore(os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                 ".local/wash/s3/records.jsonl"), AH)


def add(kind, conf, title, l0, l1, a, b, topic="t000", tags=None):
    r = store.add(kind=kind, confidence=conf, title=title, l0_summary=l0,
                  l1_overview=l1, session_id=S, harness="claude-code",
                  topic_id=topic, l2_file_path=F, l2_line_start=a, l2_line_end=b,
                  evidence_seqs=[], tags=tags or [])
    return r["record_id"][:8]


add("decision", "quoted",
    "Label data provenance in three states, and wrap every view in one provenance frame",
    "The UI distinguishes three data states - unhydrated fixture, hydrated-but-seed (a real query of fabricated rows), and genuinely live tmux mirror - and a single ProvenanceFrame wraps renderMainView() so no view can silently read as live; a list of real seats looks identical to a list of fake ones, so proof of life is movement (byte/chunk counters, last-output clock, pulsing dot), not the list.",
    "The central wrapper was chosen over editing eight views (each with early returns) - one place, no view can opt out. The banner is full-width amber for non-live states with a jump-to-live action, green for live. The default tab is live-seats so a new user lands on real data, and the StatusBar's hardcoded reconcile-complete label - the very fake sync timestamp that misled the operator - was relabeled as static sample.",
    117, 412, tags=["ui", "architecture", "provenance"])

add("failure", "quoted",
    "pipe-pane -o is a toggle, not an only-once guard (found by dogfooding UM0)",
    "The attach path passes tmux pipe-pane -o believing it means install only once; it actually toggles - so the second attach to the same pane turns the pipe OFF and the mirror silently dies. Symptoms: buffers frozen at their initial snapshot, pane_pipe=0, one early FIFO holding 285KB while later ones hold nothing.",
    "Mira diagnosed the whole chain (pane_pipe, FIFOs, transcript files, attach code), proved the semantics empirically (byte count froze after the second call), and - forbidden by packet from touching Rust - did not fix it: she reported the bug with the recommended fix, proved the UI side correct by working around it, and explicitly stated UM0 is not closed until the Rust fix lands. The React StrictMode double-invoke hypothesis was also tested and bounded rather than assumed.",
    568, 630, tags=["pty", "tmux", "bug"])

add("decision", "quoted",
    "Prove UI liveness without screenshots: drive the real UI and read its rendered buffer",
    "When screen capture is blocked, prove liveness by standing up a temporary dev-only probe that exposes the terminal instances, drives the real UI flow (clicking attach), and periodically dumps each xterm buffer - then plant a unique marker in your own real pane and find it in the UI buffer; revert the probe exactly before delivery.",
    "The marker test is what makes it decisive: the marker appeared in the real pane (capture-pane) and then in the UI's rendered xterm buffer at the real 50x200 geometry, and the buffer churned across 21 distinct content hashes. Buffer length churn alone was treated as necessary but not sufficient. The probe was reverted and verified exact, and the disclosure included the side effect that attaching toggled pipe state on panes the shared app may have been mirroring.",
    509, 641, tags=["ui", "testing", "verification"])

add("decision", "quoted",
    "Take liveness evidence on your own pane; verify what your probe actually attached",
    "For liveness evidence, target the pane you are running inside - your own tool output is genuine pane movement, so no other seat is disturbed - and verify what your automation actually operated on: a DOM selector matched an ancestor and attached the wrong seat (Lyra instead of Mira), which was caught only because the resulting session bytes did not match expectations.",
    "All five seats were live agent sessions, so injecting keystrokes into any of them would have interrupted real work; the own-pane choice removed that risk entirely. The selector mistake is recorded because the evidence still looked plausible (a real seat, real bytes) - only the attribution was wrong, and wrong attribution in a verification artifact is a defect.",
    200, 605, tags=["testing", "verification", "discipline"])

add("failure", "quoted",
    "A piped $? captures the wrong exit code - measure gate baselines before changing anything",
    "The packet claimed all four UI gates green at base; measured properly they were not (eslint 151 problems, prettier 96 files unformatted at base). The earlier green reading came from capturing a pipe's tail exit code instead of the command's - a measurement error, not a code change.",
    "Baselines were established before any edit so every later failure could be attributed honestly, and the delivery reported base-vs-delivery deltas (identical counts, zero regression; the 15 App.tsx eslint errors pre-existing and outside the author's hunks). The finding was reported once with the complete picture before the gate that depended on it, rather than fragmenting a busy seat's attention mid-turn.",
    243, 271, tags=["ci", "verification", "failure"])

add("failure", "quoted",
    "prettier --check is order-dependent because no .prettierignore exists",
    "With no .prettierignore, prettier --check . walks the generated ui/dist, so running a build before the check moves the count (96 to 103) and gate results depend on command order rather than on the code.",
    "Confirmed by cleaning dist and recounting - the source offender set was byte-identical. The durable mitigations: check before build (or rm -rf ui/dist first), and the real fix is adding .prettierignore, which is gate-tool debt owned outside the delivering seat. Recorded because a count that changes with ordering cannot be used as evidence.",
    394, 407, tags=["ci", "tooling", "failure"])

add("fact", "quoted",
    "APFS clone-copy makes a worktree cold build nearly free",
    "Cloning the shared checkout's 8GB target/ into a worktree with cp -c (APFS copy-on-write) takes ~18s, consumes ~zero extra disk, and recompiles only the path-dependent crates - measured 2 of 452, with the app up in ~36s.",
    "This is the measured answer to the shared-compile-cache problem (DV1 in the dev-velocity bucket): it beats sccache for the one-off worktree case and needs no daemon. Recorded in the dev-velocity memory and passed to the ROADMAP owner rather than edited into her document from a packet branch - the measurement belongs to the bucket, but the doc belongs to its owner.",
    436, 746, tags=["infrastructure", "velocity", "technique"])

print(json.dumps(store.stats(), indent=1))
